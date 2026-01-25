use gtk::{glib, prelude::*};
use std::{cell::RefCell, collections::HashMap, process::Command};
use zbus::{Connection, Result as ZbusResult};
use zbus::zvariant::OwnedValue;
use futures::stream::StreamExt;

#[derive(Debug, Clone)]
pub enum ConnectionType {
  Wifi,
  Ethernet,
  Disconnected,
}

#[derive(Debug, Clone)]
pub struct WifiInfo {
  pub ssid: String,
  pub signal: u8,  // 0-100
  pub security: String,
  pub connected: bool,
}

#[derive(Debug, Clone)]
pub struct EthernetInfo {
  pub name: String,
  pub device: String,
  pub connected: bool,
}

#[derive(Debug, Clone)]
pub struct NetworkMetrics {
  pub is_networking_enabled: bool,
  pub is_wifi_enabled: bool,
  pub connection_type: ConnectionType,
  pub connection_name: String,
  pub signal_strength: u8,  // 0-100, only relevant for WiFi
  pub available_wifi_networks: Vec<WifiInfo>,
  pub ethernet_connections: Vec<EthernetInfo>,
}

type NetworkCallback = Box<dyn Fn(NetworkMetrics)>;

struct NetworkServiceState {
  metrics: NetworkMetrics,
  subscribers: Vec<NetworkCallback>,
  running: bool,
}

pub struct NetworkService;

thread_local! {
  static NETWORK_SERVICE: RefCell<Option<NetworkServiceState>> = RefCell::new(None);
}

impl NetworkService {
  pub fn start(update_interval: u32) -> NetworkMetrics {
    NETWORK_SERVICE.with(|service: &RefCell<Option<NetworkServiceState>>| {
      if let Some(state) = service.borrow().as_ref() {
        return state.metrics.clone();
      }

      trigger_wifi_scan();

      let initial_metrics = collect_metrics();

      let state = NetworkServiceState {
        metrics: initial_metrics.clone(),
        subscribers: Vec::new(),
        running: true,
      };

      *service.borrow_mut() = Some(state);

      // Start D-Bus monitoring in background thread
      std::thread::spawn(|| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
          if let Err(e) = monitor_dbus_signals().await {
            eprintln!("D-Bus monitoring error: {}", e);
          }
        });
      });

      // Periodic polling at the configured interval - only updates network lists
      glib::timeout_add_seconds_local(update_interval, move || {
        std::thread::spawn(move || {
          // Trigger wifi scan before collecting network lists
          trigger_wifi_scan();
          let wifi_networks = get_wifi_connections();
          let ethernet = get_ethernet_connections();
          Self::update_network_lists(wifi_networks, ethernet);
        });

        NETWORK_SERVICE.with(|service| {
          let service_opt = service.borrow();
          if let Some(ref state) = *service_opt {
            if !state.running {
              return glib::ControlFlow::Break;
            }
          }
          glib::ControlFlow::Continue
        })
      });

      initial_metrics
    })
  }

  pub fn stop() {
    NETWORK_SERVICE.with(|service| {
      if let Some(ref mut state) = *service.borrow_mut() {
        state.running = false;
      }
      *service.borrow_mut() = None;
    });
  }

  pub fn subscribe<F>(callback: F)
  where
    F: Fn(NetworkMetrics) + 'static
  {
    NETWORK_SERVICE.with(|service| {
      if let Some(ref mut state) = *service.borrow_mut() {
        state.subscribers.push(Box::new(callback));
      }
    });
  }

  pub fn toggle_networking(enable: bool) {
    std::thread::spawn(move || {
      // Use rfkill to enable/disable WiFi and WWAN (not Bluetooth)
      let action = if enable { "unblock" } else { "block" };
      let _ = Command::new("rfkill")
        .args(&[action, "wifi"])
        .output();
      let _ = Command::new("rfkill")
        .args(&[action, "wwan"])
        .output();

      // After rfkill unblock, also tell iwd to power the device on
      if enable {
        let _ = Command::new("iwctl")
          .args(&["device", "wlan0", "set-property", "Powered", "on"])
          .output();
      }

      Self::refresh();
    });
  }

  pub fn toggle_wifi(enable: bool) {
    std::thread::spawn(move || {
      let state = if enable { "on" } else { "off" };
      let _ = Command::new("iwctl")
        .args(&["device", "wlan0", "set-property", "Powered", state])
        .output();

      // Refresh metrics after toggle
      Self::refresh();
    });
  }

  pub fn connect_to_wifi(ssid: &str) {
    let ssid = ssid.to_string();
    eprintln!("[NetworkService] connect_to_wifi called with ssid: {}", ssid);

    // First check if the network needs a password
    let ssid_clone = ssid.clone();
    std::thread::spawn(move || {
      match check_network_needs_password(&ssid_clone) {
        Ok(NeedsPassword::No) => {
          // Network is open or known - connect directly
          // Don't call refresh() here - D-Bus signals will update state when connection completes
          let result = connect_to_wifi_dbus(&ssid_clone);
          match result {
            Ok(_) => eprintln!("[NetworkService] connect_to_wifi_dbus succeeded"),
            Err(e) => {
              eprintln!("[NetworkService] connect_to_wifi_dbus error: {}", e);
              Self::refresh(); // Only refresh on error
            }
          }
        }
        Ok(NeedsPassword::Yes) => {
          // Network needs a password - show dialog on main thread
          eprintln!("[NetworkService] Network {} needs password, showing dialog", ssid_clone);
          let ssid_for_dialog = ssid_clone.clone();
          glib::idle_add_once(move || {
            show_password_dialog(&ssid_for_dialog);
          });
        }
        Err(e) => {
          eprintln!("[NetworkService] Error checking network: {}", e);
          Self::refresh();
        }
      }
    });
  }

  /// Disconnect from the current WiFi network
  pub fn disconnect_wifi() {
    eprintln!("[NetworkService] disconnect_wifi called");
    std::thread::spawn(move || {
      let result = Command::new("iwctl")
        .args(&["station", "wlan0", "disconnect"])
        .output();

      match result {
        Ok(output) => {
          if output.status.success() {
            eprintln!("[NetworkService] iwctl disconnect succeeded");
          } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("[NetworkService] iwctl disconnect failed: {}", stderr);
          }
        }
        Err(e) => eprintln!("[NetworkService] iwctl disconnect command failed: {}", e),
      }

      Self::refresh();
    });
  }

  /// Show a confirmation dialog and disconnect if confirmed
  pub fn confirm_disconnect_wifi(ssid: &str) {
    let ssid = ssid.to_string();
    eprintln!("[NetworkService] confirm_disconnect_wifi called for: {}", ssid);
    glib::idle_add_once(move || {
      show_disconnect_confirmation_dialog(&ssid);
    });
  }

  /// Connect to a WiFi network with a password using iwctl
  pub fn connect_to_wifi_with_password(ssid: &str, password: &str) {
    let ssid = ssid.to_string();
    let password = password.to_string();
    eprintln!("[NetworkService] connect_to_wifi_with_password called with ssid: {}", ssid);

    std::thread::spawn(move || {
      // Don't call refresh() on success - D-Bus signals will update state when connection completes
      let result = Command::new("iwctl")
        .args(&["--passphrase", &password, "station", "wlan0", "connect", &ssid])
        .output();

      match result {
        Ok(output) => {
          if output.status.success() {
            eprintln!("[NetworkService] iwctl connect succeeded for {}", ssid);
            // D-Bus signals will handle state update
          } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("[NetworkService] iwctl connect failed for {}: {}", ssid, stderr);
            Self::refresh(); // Only refresh on failure
          }
        }
        Err(e) => {
          eprintln!("[NetworkService] iwctl command failed: {}", e);
          Self::refresh(); // Only refresh on error
        }
      }
    });
  }

  /// Immediately refresh all metrics and notify subscribers
  fn refresh() {
    let metrics = collect_metrics();

    glib::idle_add_once(move || {
      NETWORK_SERVICE.with(|service| {
        let mut service_opt = service.borrow_mut();
        if let Some(ref mut state) = *service_opt {
          eprintln!("[NetworkService] refresh() - signal_strength: {} -> {}", state.metrics.signal_strength, metrics.signal_strength);
          state.metrics = metrics.clone();

          for subscriber in &state.subscribers {
            subscriber(metrics.clone());
          }
        }
      });
    });
  }

  /// Update only the network lists (wifi networks + ethernet) - used by periodic polling
  fn update_network_lists(wifi_networks: Vec<WifiInfo>, ethernet: Vec<EthernetInfo>) {
    eprintln!("[NetworkService] update_network_lists: {} wifi networks, {} ethernet", wifi_networks.len(), ethernet.len());
    glib::idle_add_once(move || {
      NETWORK_SERVICE.with(|service| {
        let mut service_opt = service.borrow_mut();
        if let Some(ref mut state) = *service_opt {
          state.metrics.available_wifi_networks = wifi_networks;
          state.metrics.ethernet_connections = ethernet;

          let metrics = state.metrics.clone();
          for subscriber in &state.subscribers {
            subscriber(metrics.clone());
          }
        }
      });
    });
  }

  /// Update only connection state - used by D-Bus signal handler
  fn update_connection_state(
    is_networking_enabled: Option<bool>,
    is_wifi_enabled: Option<bool>,
    connection_type: Option<ConnectionType>,
    connection_name: Option<String>,
    signal_strength: Option<u8>,
  ) {
    glib::idle_add_once(move || {
      NETWORK_SERVICE.with(|service| {
        let mut service_opt = service.borrow_mut();
        if let Some(ref mut state) = *service_opt {
          let mut changed = false;

          if let Some(enabled) = is_networking_enabled {
            if state.metrics.is_networking_enabled != enabled {
              state.metrics.is_networking_enabled = enabled;
              changed = true;
            }
          }

          if let Some(enabled) = is_wifi_enabled {
            if state.metrics.is_wifi_enabled != enabled {
              state.metrics.is_wifi_enabled = enabled;
              changed = true;
            }
          }

          if let Some(conn_type) = connection_type {
            state.metrics.connection_type = conn_type;
            changed = true;
          }

          if let Some(name) = connection_name {
            if state.metrics.connection_name != name {
              state.metrics.connection_name = name;
              changed = true;
            }
          }

          if let Some(signal) = signal_strength {
            if state.metrics.signal_strength != signal {
              eprintln!("[NetworkService] signal_strength updated: {} -> {}", state.metrics.signal_strength, signal);
              state.metrics.signal_strength = signal;
              changed = true;
            }
          }

          // Only notify if something actually changed
          if changed {
            let metrics = state.metrics.clone();
            for subscriber in &state.subscribers {
              subscriber(metrics.clone());
            }
          }
        }
      });
    });
  }
}

fn trigger_wifi_scan() {
  if let Ok(connection) = zbus::blocking::Connection::system() {
    if let Ok(station_path) = find_iwd_station_path(&connection) {
      if let Ok(proxy) = zbus::blocking::Proxy::new(
        &connection,
        "net.connman.iwd",
        station_path.as_str(),
        "net.connman.iwd.Station",
      ) {
        let _: Result<(), _> = proxy.call("Scan", &());
      }
    }
  }
}

fn collect_metrics() -> NetworkMetrics {
  let is_networking_enabled = is_networking_enabled();
  let is_wifi_enabled = is_wifi_enabled();
  let (connection_type, connection_name, signal_strength) = get_primary_connection();
  let ethernet_connections = get_ethernet_connections();
  let available_wifi_networks = get_wifi_connections();

  NetworkMetrics {
    
    is_networking_enabled,
    is_wifi_enabled,
    connection_type,
    connection_name,
    signal_strength,
    available_wifi_networks,
    ethernet_connections,
  }
}


/// Check if networking is enabled by checking rfkill state for wifi
fn is_networking_enabled() -> bool {
  let output = Command::new("rfkill")
    .args(&["list", "wifi"])
    .output();

  if let Ok(output) = output {
    if output.status.success() {
      let stdout = String::from_utf8_lossy(&output.stdout);
      // Check for "Soft blocked: yes" or "Hard blocked: yes"
      // If either is blocked, networking is disabled
      let soft_blocked = stdout.lines().any(|line| {
        line.contains("Soft blocked:") && line.contains("yes")
      });
      let hard_blocked = stdout.lines().any(|line| {
        line.contains("Hard blocked:") && line.contains("yes")
      });
      return !soft_blocked && !hard_blocked;
    }
  }

  // Default to true if we can't determine the state
  true
}

fn is_wifi_enabled() -> bool {
  let output = Command::new("iwctl")
    .args(&["device", "list"])
    .output();

  if let Ok(output) = output {
    if output.status.success() {
      let stdout = String::from_utf8_lossy(&output.stdout);
      // Look for wlan0 line and check if Powered column is "on"
      for line in stdout.lines() {
        if line.contains("wlan0") {
          let cleaned = strip_ansi_codes(line);
          let parts: Vec<&str> = cleaned.split_whitespace().collect();
          // Format: Name Address Powered Adapter Mode
          // Index:  0    1       2       3       4
          if parts.len() >= 3 {
            return parts[2] == "on";
          }
        }
      }
    }
  }

  false
}

fn get_primary_connection() -> (ConnectionType, String, u8) {
  let primary_device = get_default_route_device();

  if let Some(device) = primary_device {
    // Check if it's wifi (wlan*)
    if device.starts_with("wlan") {
      // Get wifi connection info
      let output = Command::new("iwctl")
        .args(&["station", &device, "show"])
        .output();

      if let Ok(output) = output {
        if output.status.success() {
          let stdout = String::from_utf8_lossy(&output.stdout);

          let mut connected_network = None;
          let mut rssi = None;

          for line in stdout.lines() {
            let cleaned = strip_ansi_codes(line);
            if cleaned.contains("Connected network") {
              if let Some(network) = cleaned.split_whitespace().last() {
                connected_network = Some(network.to_string());
              }
            } else if cleaned.contains("RSSI") {
              // Parse: "RSSI                  -59 dBm"
              let parts: Vec<&str> = cleaned.split_whitespace().collect();
              if let Some(pos) = parts.iter().position(|&p| p == "RSSI") {
                if pos + 1 < parts.len() {
                  if let Ok(dbm) = parts[pos + 1].parse::<i32>() {
                    rssi = Some(dbm);
                  }
                }
              }
            }
          }

          if let Some(network) = connected_network {
            let signal = rssi.map(|r| dbm_to_percentage(r)).unwrap_or(0);
            return (ConnectionType::Wifi, network, signal);
          }
        }
      }
    } else if device.starts_with("en") || device.starts_with("eth") {
      // It's ethernet
      return (ConnectionType::Ethernet, device, 0);
    }
  }

  (ConnectionType::Disconnected, String::from("Not connected"), 0)
}

fn dbm_to_percentage(dbm: i32) -> u8 {
  // Convert dBm to percentage (0-100)
  // -30 dBm = 100% (excellent)
  // -67 dBm = 50% (good)
  // -90 dBm = 0% (unusable)
  if dbm >= -30 {
    100
  } else if dbm <= -90 {
    0
  } else {
    // Linear interpolation
    (((dbm + 90) * 100) / 60).max(0).min(100) as u8
  }
}

fn get_default_route_device() -> Option<String> {
  let output = Command::new("ip")
    .args(&["route", "show", "default"])
    .output();

  if let Ok(output) = output {
    if output.status.success() {
      let stdout = String::from_utf8_lossy(&output.stdout);

      // Parse: "default via 192.168.1.1 dev wlp2s0 proto dhcp metric 600"
      for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();

        // Find "dev" keyword and get the next word (the device name)
        for i in 0..parts.len() {
          if parts[i] == "dev" && i + 1 < parts.len() {
            return Some(parts[i + 1].to_string());
          }
        }
      }
    }
  }

  None
}


fn get_ethernet_connections() -> Vec<EthernetInfo> {
  let output = Command::new("networkctl")
    .args(&["list"])
    .output();

  if let Ok(output) = output {
    if output.status.success() {
      let stdout = String::from_utf8_lossy(&output.stdout);
      return stdout.lines()
        .skip(1)  // Skip header
        .filter_map(|line| {
          let parts: Vec<&str> = line.split_whitespace().collect();

          // Format: IDX LINK TYPE OPERATIONAL SETUP
          if parts.len() >= 4 && parts[2] == "ether" {
            let device = parts[1].to_string();
            let operational = parts[3];

            // Connected if operational is "routable" or "carrier"
            let connected = operational == "routable" || operational == "carrier";

            Some(EthernetInfo {
              name: device.clone(),
              device,
              connected,
            })
          } else {
            None
          }
        })
        .collect();
    }
  }

  Vec::new()
}

fn get_wifi_connections() -> Vec<WifiInfo> {
  // Use D-Bus to query iwd for precise signal strength values
  get_wifi_connections_dbus().unwrap_or_else(|e| {
    eprintln!("D-Bus wifi query failed: {}, falling back to iwctl", e);
    get_wifi_connections_iwctl_fallback()
  })
}

fn get_wifi_connections_dbus() -> Result<Vec<WifiInfo>, Box<dyn std::error::Error>> {
  // Use blocking D-Bus connection
  let connection = zbus::blocking::Connection::system()?;

  // Find the station path by looking for objects that implement net.connman.iwd.Station
  let station_path = find_iwd_station_path(&connection)?;

  let proxy = zbus::blocking::Proxy::new(
    &connection,
    "net.connman.iwd",
    station_path.as_str(),
    "net.connman.iwd.Station",
  )?;

  // Call GetOrderedNetworks on the station to get networks with signal strength
  // Returns array of (object_path, signal_strength_dbm_times_100)
  let ordered_networks: Vec<(zbus::zvariant::OwnedObjectPath, i16)> =
    proxy.call("GetOrderedNetworks", &())?;

  let mut networks = Vec::new();

  for (network_path, signal_dbm100) in ordered_networks {
    // Get network properties
    let network_proxy = zbus::blocking::Proxy::new(
      &connection,
      "net.connman.iwd",
      network_path.as_str(),
      "net.connman.iwd.Network",
    )?;

    let name: String = network_proxy.get_property("Name")?;
    let network_type: String = network_proxy.get_property("Type")?;
    let connected: bool = network_proxy.get_property("Connected")?;

    // Convert signal from dBm*100 to percentage
    // signal_dbm100 is like -5900 for -59 dBm
    let dbm = signal_dbm100 / 100;
    let signal = dbm_to_percentage(dbm as i32);

    networks.push(WifiInfo {
      ssid: name,
      signal,
      security: network_type,
      connected,
    });
  }

  // Deduplicate by SSID, keeping the strongest signal
  let mut dedup_map: HashMap<String, WifiInfo> = HashMap::new();

  for network in networks {
    let ssid = network.ssid.clone();
    dedup_map.entry(ssid)
      .and_modify(|existing| {
        let should_replace = match (network.connected, existing.connected) {
          (true, false) => true,
          (false, true) => false,
          _ => network.signal > existing.signal,
        };
        if should_replace {
          *existing = network.clone();
        }
      })
      .or_insert(network);
  }

  let mut result: Vec<WifiInfo> = dedup_map.into_values().collect();

  // Sort with connected network first, then by signal strength
  result.sort_by(|a, b| {
    match (a.connected, b.connected) {
      (true, false) => std::cmp::Ordering::Less,
      (false, true) => std::cmp::Ordering::Greater,
      _ => b.signal.cmp(&a.signal),
    }
  });

  Ok(result)
}

/// Find the iwd station object path by querying the ObjectManager
fn find_iwd_station_path(connection: &zbus::blocking::Connection) -> Result<String, Box<dyn std::error::Error>> {

  // Query ObjectManager to get all iwd objects
  let proxy = zbus::blocking::Proxy::new(
    connection,
    "net.connman.iwd",
    "/",
    "org.freedesktop.DBus.ObjectManager",
  )?;

  let objects: HashMap<zbus::zvariant::OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>> =
    proxy.call("GetManagedObjects", &())?;

  // Find the first object that implements net.connman.iwd.Station
  for (path, interfaces) in objects {
    if interfaces.contains_key("net.connman.iwd.Station") {
      return Ok(path.to_string());
    }
  }

  Err("No iwd station found".into())
}

enum NeedsPassword {
  Yes,
  No,
}

/// Check if a network needs a password to connect
fn check_network_needs_password(ssid: &str) -> Result<NeedsPassword, Box<dyn std::error::Error>> {
  let connection = zbus::blocking::Connection::system()?;

  let proxy = zbus::blocking::Proxy::new(
    &connection,
    "net.connman.iwd",
    "/",
    "org.freedesktop.DBus.ObjectManager",
  )?;

  let objects: HashMap<zbus::zvariant::OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>> =
    proxy.call("GetManagedObjects", &())?;

  for (_path, interfaces) in objects {
    if let Some(network_props) = interfaces.get("net.connman.iwd.Network") {
      if let Some(name_value) = network_props.get("Name") {
        if let Ok(name) = TryInto::<String>::try_into(name_value.clone()) {
          if name == ssid {
            let network_type = network_props.get("Type")
              .and_then(|v| TryInto::<String>::try_into(v.clone()).ok())
              .unwrap_or_default();

            let known_network = network_props.get("KnownNetwork")
              .and_then(|v| TryInto::<zbus::zvariant::OwnedObjectPath>::try_into(v.clone()).ok());

            let is_known = known_network.is_some();
            let is_open = network_type == "open";

            eprintln!("[NetworkService] check_network_needs_password: type={}, is_known={}, is_open={}",
              network_type, is_known, is_open);

            if is_open || is_known {
              return Ok(NeedsPassword::No);
            } else {
              return Ok(NeedsPassword::Yes);
            }
          }
        }
      }
    }
  }

  Err(format!("Network '{}' not found", ssid).into())
}

/// Show a confirmation dialog for disconnecting from a WiFi network
fn show_disconnect_confirmation_dialog(ssid: &str) {
  let ssid = ssid.to_string();

  let dialog = gtk::Window::builder()
    .title("Disconnect from WiFi")
    .modal(true)
    .default_width(350)
    .default_height(120)
    .resizable(false)
    .build();

  let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
  content.set_margin_top(20);
  content.set_margin_bottom(20);
  content.set_margin_start(20);
  content.set_margin_end(20);

  let label = gtk::Label::new(Some(&format!("Disconnect from \"{}\"?", ssid)));
  label.set_halign(gtk::Align::Start);
  content.append(&label);

  let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  button_box.set_halign(gtk::Align::End);
  button_box.set_margin_top(12);

  let cancel_button = gtk::Button::with_label("Cancel");
  let disconnect_button = gtk::Button::with_label("Disconnect");
  disconnect_button.add_css_class("destructive-action");

  button_box.append(&cancel_button);
  button_box.append(&disconnect_button);
  content.append(&button_box);

  dialog.set_child(Some(&content));

  // Connect signals
  let dialog_weak = dialog.downgrade();
  cancel_button.connect_clicked(move |_| {
    if let Some(d) = dialog_weak.upgrade() {
      d.close();
    }
  });

  let dialog_weak = dialog.downgrade();
  disconnect_button.connect_clicked(move |_| {
    NetworkService::disconnect_wifi();
    if let Some(d) = dialog_weak.upgrade() {
      d.close();
    }
  });

  dialog.present();
}

/// Show a password dialog for connecting to a WiFi network
fn show_password_dialog(ssid: &str) {
  let ssid = ssid.to_string();

  let dialog = gtk::Window::builder()
    .title(format!("Connect to {}", ssid))
    .modal(true)
    .default_width(350)
    .default_height(150)
    .resizable(false)
    .build();

  let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
  content.set_margin_top(20);
  content.set_margin_bottom(20);
  content.set_margin_start(20);
  content.set_margin_end(20);

  let label = gtk::Label::new(Some(&format!("Enter password for \"{}\":", ssid)));
  label.set_halign(gtk::Align::Start);
  content.append(&label);

  let password_entry = gtk::PasswordEntry::new();
  password_entry.set_show_peek_icon(true);
  password_entry.set_hexpand(true);
  content.append(&password_entry);

  let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  button_box.set_halign(gtk::Align::End);
  button_box.set_margin_top(8);

  let cancel_button = gtk::Button::with_label("Cancel");
  let connect_button = gtk::Button::with_label("Connect");
  connect_button.add_css_class("suggested-action");

  button_box.append(&cancel_button);
  button_box.append(&connect_button);
  content.append(&button_box);

  dialog.set_child(Some(&content));

  // Connect signals
  let dialog_weak = dialog.downgrade();
  cancel_button.connect_clicked(move |_| {
    if let Some(d) = dialog_weak.upgrade() {
      d.close();
    }
  });

  let dialog_weak = dialog.downgrade();
  let ssid_clone = ssid.clone();
  let entry_clone = password_entry.clone();
  connect_button.connect_clicked(move |_| {
    let password = entry_clone.text().to_string();
    if !password.is_empty() {
      NetworkService::connect_to_wifi_with_password(&ssid_clone, &password);
      if let Some(d) = dialog_weak.upgrade() {
        d.close();
      }
    }
  });

  // Allow Enter key to submit
  let dialog_weak = dialog.downgrade();
  let ssid_clone = ssid.clone();
  password_entry.connect_activate(move |entry| {
    let password = entry.text().to_string();
    if !password.is_empty() {
      NetworkService::connect_to_wifi_with_password(&ssid_clone, &password);
      if let Some(d) = dialog_weak.upgrade() {
        d.close();
      }
    }
  });

  dialog.present();
}

/// Connect to a WiFi network via D-Bus by finding the network and calling Connect
/// This should only be called for networks that don't need a password (open or known)
fn connect_to_wifi_dbus(ssid: &str) -> Result<(), Box<dyn std::error::Error>> {
  let connection = zbus::blocking::Connection::system()?;

  // Query ObjectManager to find the Network object with matching SSID
  let proxy = zbus::blocking::Proxy::new(
    &connection,
    "net.connman.iwd",
    "/",
    "org.freedesktop.DBus.ObjectManager",
  )?;

  let objects: HashMap<zbus::zvariant::OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>> =
    proxy.call("GetManagedObjects", &())?;

  // Find the Network object with the matching SSID
  for (path, interfaces) in objects {
    if let Some(network_props) = interfaces.get("net.connman.iwd.Network") {
      if let Some(name_value) = network_props.get("Name") {
        if let Ok(name) = TryInto::<String>::try_into(name_value.clone()) {
          if name == ssid {
            eprintln!("[NetworkService] Found network {} at path {}", ssid, path);

            // Call Connect on this Network
            let network_proxy = zbus::blocking::Proxy::new(
              &connection,
              "net.connman.iwd",
              path.as_str(),
              "net.connman.iwd.Network",
            )?;

            let result: Result<(), zbus::Error> = network_proxy.call("Connect", &());
            match result {
              Ok(_) => {
                eprintln!("[NetworkService] Connect call succeeded for {}", ssid);
                return Ok(());
              }
              Err(e) => {
                eprintln!("[NetworkService] Connect call failed for {}: {}", ssid, e);
                return Err(e.into());
              }
            }
          }
        }
      }
    }
  }

  Err(format!("Network '{}' not found", ssid).into())
}

/// Fallback to iwctl parsing if D-Bus fails
fn get_wifi_connections_iwctl_fallback() -> Vec<WifiInfo> {
  let output = Command::new("iwctl")
    .args(&["station", "wlan0", "get-networks"])
    .output();

  if let Ok(output) = output {
    if output.status.success() {
      let stdout = String::from_utf8_lossy(&output.stdout);

      let networks: Vec<WifiInfo> = stdout.lines()
        .skip(4)
        .filter_map(|line| {
          if line.trim().is_empty() || line.contains("----") {
            return None;
          }

          let cleaned = strip_ansi_codes(line);
          let connected = cleaned.trim_start().starts_with('>');
          let parts: Vec<&str> = cleaned.split_whitespace().collect();

          if parts.len() < 3 {
            return None;
          }

          let mut ssid_parts = Vec::new();
          let mut security = String::new();

          for part in parts.iter() {
            if part == &">" {
              continue;
            }
            if *part == "psk" || *part == "open" || *part == "8021x" {
              security = part.to_string();
              break;
            }
            ssid_parts.push(*part);
          }

          if ssid_parts.is_empty() {
            return None;
          }

          let ssid = ssid_parts.join(" ");
          // Fallback: use asterisk counting
          let bar_count = count_bright_asterisks(line);
          let signal = ((bar_count * 25).min(100)) as u8;

          Some(WifiInfo { ssid, signal, security, connected })
        })
        .collect();

      return networks;
    }
  }

  Vec::new()
}

fn strip_ansi_codes(s: &str) -> String {
  let mut result = String::new();
  let mut in_escape = false;

  for c in s.chars() {
    if c == '\x1b' {
      in_escape = true;
    } else if in_escape {
      if c == 'm' {
        in_escape = false;
      }
    } else {
      result.push(c);
    }
  }

  result
}

/// Count bright (non-dimmed) asterisks in iwctl signal output.
/// iwctl shows signal as `****` where dimmed asterisks use `\x1b[1;90m`
/// Example: `*\x1b[1;90m***\x1b[0m` = 1 bright + 3 dimmed = 1 bar
fn count_bright_asterisks(s: &str) -> usize {
  let mut count = 0;
  let mut in_escape = false;
  let mut is_dimmed = false;

  let chars: Vec<char> = s.chars().collect();
  let mut i = 0;

  while i < chars.len() {
    let c = chars[i];

    if c == '\x1b' {
      in_escape = true;
      // Check if this is the dim sequence [1;90m
      let rest: String = chars[i..].iter().collect();
      if rest.starts_with("\x1b[1;90m") {
        is_dimmed = true;
      } else if rest.starts_with("\x1b[0m") {
        is_dimmed = false;
      }
    } else if in_escape {
      if c == 'm' {
        in_escape = false;
      }
    } else if c == '*' && !is_dimmed {
      count += 1;
    }

    i += 1;
  }

  count
}

async fn monitor_dbus_signals() -> ZbusResult<()> {
  let connection = Connection::system().await?;

  use zbus::MatchRule;
  use futures::future::select;
  use futures::pin_mut;

  // Monitor iwd PropertiesChanged signals
  let iwd_rule = MatchRule::builder()
    .msg_type(zbus::message::Type::Signal)
    .sender("net.connman.iwd")?
    .interface("org.freedesktop.DBus.Properties")?
    .member("PropertiesChanged")?
    .build();

  // Monitor systemd-networkd PropertiesChanged signals
  let networkd_rule = MatchRule::builder()
    .msg_type(zbus::message::Type::Signal)
    .sender("org.freedesktop.network1")?
    .interface("org.freedesktop.DBus.Properties")?
    .member("PropertiesChanged")?
    .build();

  let mut iwd_stream = zbus::MessageStream::for_match_rule(
    iwd_rule,
    &connection,
    None,
  ).await?;

  let mut networkd_stream = zbus::MessageStream::for_match_rule(
    networkd_rule,
    &connection,
    None,
  ).await?;

  loop {
    let iwd_next = iwd_stream.next();
    let networkd_next = networkd_stream.next();
    pin_mut!(iwd_next, networkd_next);

    match select(iwd_next, networkd_next).await {
      futures::future::Either::Left((Some(msg), _)) => {
        if let Ok(msg) = msg {
          if let Ok((interface, changed_props, _invalidated)) = msg.body().deserialize::<(
            String,
            HashMap<String, OwnedValue>,
            Vec<String>,
          )>() {
            handle_iwd_property_change(&interface, &changed_props);
          }
        }
      }
      futures::future::Either::Right((Some(msg), _)) => {
        if let Ok(msg) = msg {
          if let Ok((interface, changed_props, _invalidated)) = msg.body().deserialize::<(
            String,
            HashMap<String, OwnedValue>,
            Vec<String>,
          )>() {
            handle_networkd_property_change(&interface, &changed_props);
          }
        }
      }
      _ => {}
    }
  }
}

/// Handle property changes from iwd and update connection state accordingly
fn handle_iwd_property_change(
  interface: &str,
  changed_props: &HashMap<String, OwnedValue>,
) {
  match interface {
    "net.connman.iwd.Device" => {
      // Device Powered property -> is_wifi_enabled
      if let Some(powered_value) = changed_props.get("Powered") {
        if let Ok(powered) = TryInto::<bool>::try_into(powered_value.clone()) {
          NetworkService::update_connection_state(None, Some(powered), None, None, None);
        }
      }
    }

    "net.connman.iwd.Station" => {
      // Station State property -> connection state
      if let Some(state_value) = changed_props.get("State") {
        if let Ok(state) = TryInto::<String>::try_into(state_value.clone()) {
          match state.as_str() {
            "disconnected" => {
              NetworkService::update_connection_state(
                None,
                None,
                Some(ConnectionType::Disconnected),
                Some("Not connected".to_string()),
                Some(0),
              );
            }
            "connected" => {
              // When connected, fetch the connection details
              // Also set is_networking_enabled and is_wifi_enabled to true since we're connected
              let (conn_type, conn_name, signal) = get_primary_connection();
              NetworkService::update_connection_state(
                Some(true),  // networking must be enabled if we're connected
                Some(true),  // wifi must be enabled if we're connected
                Some(conn_type),
                Some(conn_name),
                Some(signal),
              );
            }
            _ => {
              // "connecting", "disconnecting", "roaming" - ignore intermediate states
            }
          }
        }
      }

      // ConnectedNetwork property changed
      if changed_props.contains_key("ConnectedNetwork") {
        // Fetch updated connection info
        let (conn_type, conn_name, signal) = get_primary_connection();
        NetworkService::update_connection_state(
          None,
          None,
          Some(conn_type),
          Some(conn_name),
          Some(signal),
        );
      }
    }

    "net.connman.iwd.Network" => {
      // Network Connected property changed
      if let Some(connected_value) = changed_props.get("Connected") {
        if let Ok(connected) = TryInto::<bool>::try_into(connected_value.clone()) {
          if connected {
            // This network became connected - fetch connection details
            let (conn_type, conn_name, signal) = get_primary_connection();
            NetworkService::update_connection_state(
              Some(true),  // networking must be enabled if connected
              Some(true),  // wifi must be enabled if connected
              Some(conn_type),
              Some(conn_name),
              Some(signal),
            );
          }
        }
      }
    }

    _ => {
      // Ignore other interfaces
    }
  }
}

/// Handle property changes from systemd-networkd and update connection state
fn handle_networkd_property_change(
  interface: &str,
  changed_props: &HashMap<String, OwnedValue>,
) {
  if interface == "org.freedesktop.network1.Manager" {
    if changed_props.contains_key("OperationalState") {
      // When operational state changes, refresh connection info
      // but check rfkill for networking enabled state
      let networking_enabled = is_networking_enabled();
      let wifi_enabled = is_wifi_enabled();
      let (conn_type, conn_name, signal) = get_primary_connection();

      NetworkService::update_connection_state(
        Some(networking_enabled),
        Some(wifi_enabled),
        Some(conn_type),
        Some(conn_name),
        Some(signal),
      );
    }
  }
}
