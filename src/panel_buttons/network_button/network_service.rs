use gtk::glib;
use std::{cell::RefCell, process::Command};

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
  pub is_airplane_mode: bool,
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

      let initial_metrics = collect_metrics();

      let state = NetworkServiceState {
        metrics: initial_metrics.clone(),
        subscribers: Vec::new(),
        running: true,
      };

      *service.borrow_mut() = Some(state);

      glib::timeout_add_seconds_local(update_interval, move || {
        // Fetch metrics asynchronously on a background thread
        std::thread::spawn(move || {
          let metrics = collect_metrics();

          // Send result back to main thread using idle_add_once (not _local)
          // since we're calling from a background thread
          glib::idle_add_once(move || {
            NETWORK_SERVICE.with(|service| {
              let mut service_opt = service.borrow_mut();
              if let Some(ref mut state) = *service_opt {
                state.metrics = metrics.clone();

                // Notify all subscribers
                for subscriber in &state.subscribers {
                  subscriber(metrics.clone());
                }
              }
            });
          });
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
      let state = if enable { "on" } else { "off" };
      let _ = Command::new("nmcli")
        .args(&["networking", state])
        .output();

      // Refresh metrics after toggle
      Self::refresh();
    });
  }

  pub fn toggle_wifi(enable: bool) {
    std::thread::spawn(move || {
      let state = if enable { "on" } else { "off" };
      let _ = Command::new("nmcli")
        .args(&["radio", "wifi", state])
        .output();

      // Refresh metrics after toggle
      Self::refresh();
    });
  }

  pub fn toggle_airplane_mode(enable: bool) {
    std::thread::spawn(move || {
      // Use rfkill to toggle all radios (wifi, bluetooth, wwan)
      let state = if enable { "block" } else { "unblock" };
      let _ = Command::new("rfkill")
        .args(&[state, "all"])
        .output();

      // Refresh metrics after toggle
      Self::refresh();
    });
  }

  pub fn connect_to_wifi(ssid: &str) {
    let ssid = ssid.to_string();
    std::thread::spawn(move || {
      let _ = Command::new("nmcli")
        .args(&["device", "wifi", "connect", &ssid])
        .output();

      // Refresh metrics after connection attempt
      Self::refresh();
    });
  }

  /// Immediately refresh metrics and notify subscribers
  fn refresh() {
    let metrics = collect_metrics();

    glib::idle_add_once(move || {
      NETWORK_SERVICE.with(|service| {
        let mut service_opt = service.borrow_mut();
        if let Some(ref mut state) = *service_opt {
          state.metrics = metrics.clone();

          for subscriber in &state.subscribers {
            subscriber(metrics.clone());
          }
        }
      });
    });
  }


}

fn collect_metrics() -> NetworkMetrics {
  let is_networking_enabled = is_networking_enabled();
  let is_airplane_mode = is_airplane_mode();
  let is_wifi_enabled = is_wifi_enabled();
  let (connection_type, connection_name, signal_strength) = get_primary_connection();
  let ethernet_connections = get_ethernet_connections();
  let available_wifi_networks = get_wifi_connections();
 
  NetworkMetrics {
    is_networking_enabled,
    is_airplane_mode,
    is_wifi_enabled,
    connection_type,
    connection_name,
    signal_strength,
    available_wifi_networks,
    ethernet_connections,
  }
}

pub fn is_airplane_mode() -> bool {
  let output = Command::new("rfkill")
    .args(&["--json"])
    .output();

  if let Ok(output) = output {
    if output.status.success() {
      let stdout = String::from_utf8_lossy(&output.stdout);

      // Parse JSON output to check if all wlan and bluetooth devices are soft-blocked
      // Example: {"rfkilldevices":[{"type":"bluetooth","soft":"blocked"},{"type":"wlan","soft":"blocked"}]}
      if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
        if let Some(devices) = json.get("rfkilldevices").and_then(|d| d.as_array()) {
          let radio_devices: Vec<_> = devices.iter()
            .filter(|d| {
              let device_type = d.get("type").and_then(|t| t.as_str()).unwrap_or("");
              device_type == "wlan" || device_type == "bluetooth"
            })
            .collect();

          // If there are no radio devices, not in airplane mode
          if radio_devices.is_empty() {
            return false;
          }

          // Airplane mode = all wlan and bluetooth devices are soft-blocked
          return radio_devices.iter().all(|d| {
            d.get("soft").and_then(|s| s.as_str()) == Some("blocked")
          });
        }
      }
    }
  }
  false
}


fn is_wifi_enabled() -> bool {
  let output = Command::new("nmcli")
    .args(&["radio", "wifi"])
    .output();

  if let Ok(output) = output {
    if output.status.success() {
      let stdout = String::from_utf8_lossy(&output.stdout);
      return stdout.trim() == "enabled";
    }
  }

  false
}

fn is_networking_enabled() -> bool {
  let output = Command::new("nmcli")
    .args(&["networking"])
    .output();

  if let Ok(output) = output {
    if output.status.success() {
      let stdout = String::from_utf8_lossy(&output.stdout);
      return stdout.trim() == "enabled";
    }
  }

  false
}

fn get_primary_connection() -> (ConnectionType, String, u8) {
  let primary_device = get_default_route_device();

  let output = Command::new("nmcli")
    .args(&["-t", "-f", "TYPE,NAME,DEVICE", "connection", "show", "--active"])
    .output();

  if let Ok(output) = output {
    if output.status.success() {
      let stdout = String::from_utf8_lossy(&output.stdout);

      for line in stdout.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 3 {
          let conn_type = parts[0];
          let name = parts[1].to_string();
          let conn_device = parts[2];

          // If we have a primary device, match it. Otherwise, return the first connection.
          let is_match = if let Some(ref device) = primary_device {
            conn_device == device
          } else {
            true  // No primary device found, use first connection as fallback
          };

          if is_match {
            if conn_type == "802-11-wireless" {
              let signal = get_wifi_signal_strength(&name);
              return (ConnectionType::Wifi, name, signal);
            } else if conn_type == "802-3-ethernet" {
              return (ConnectionType::Ethernet, name, 0);
            }
          }
        }
      }
    }
  }

  (ConnectionType::Disconnected, String::from("Not connected"), 0)
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

fn get_wifi_signal_strength(ssid: &str) -> u8 {
  let output = Command::new("nmcli")
    .args(&["-t", "-f", "SSID,SIGNAL,IN-USE", "device", "wifi", "list", "--rescan", "no"])
    .output();

  if let Ok(output) = output {
    if output.status.success() {
      let stdout = String::from_utf8_lossy(&output.stdout);

      let mut fallback_signal = None;

      // Find the line matching the SSID, prioritizing the connected one
      for line in stdout.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 3 && parts[0] == ssid {
          if let Ok(signal) = parts[1].trim().parse::<u8>() {
            let is_connected = parts[2].trim() == "*";

            if is_connected {
              // Found the connected network, return immediately
              return signal;
            }

            // Store as fallback in case we don't find a connected one
            if fallback_signal.is_none() {
              fallback_signal = Some(signal);
            }
          }
        }
      }

      // Return fallback if we found any matching SSID (but none connected)
      if let Some(signal) = fallback_signal {
        return signal;
      }
    }
  }

  0
}  

fn get_ethernet_connections() -> Vec<EthernetInfo> {
  let output = Command::new("nmcli")
    .args(&["-t", "-f", "NAME,TYPE,DEVICE", "connection", "show"])
    .output();

  if let Ok(output) = output {
    if output.status.success() {
      let stdout = String::from_utf8_lossy(&output.stdout);
      return stdout.lines()
        .filter_map(|line| {
          let parts: Vec<&str> = line.split(':').collect();
          if parts.len() >= 3 && parts[1] == "802-3-ethernet" {
            let name = parts[0].to_string();
            let device = parts[2].to_string();
            let connected = !device.is_empty();

            Some(EthernetInfo {
              name,
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
  let output = Command::new("nmcli")
    .args(&["-t", "-f", "SSID,SIGNAL,SECURITY,IN-USE", "device", "wifi", "list"])
    .output();

  if let Ok(output) = output {
    if output.status.success() {
      let stdout = String::from_utf8_lossy(&output.stdout);

      let networks: Vec<WifiInfo> = stdout.lines()
        .filter_map(|line| {
          let parts: Vec<&str> = line.split(':').collect();
          if parts.len() >= 4 {
            let ssid = parts[0].to_string();
            let signal = parts[1].parse::<u8>().unwrap_or(0);
            let security = parts[2].to_string();
            let connected = parts[3].trim() == "*";

            // Skip empty SSIDs
            if ssid.is_empty() {
              return None;
            }

            // Filter out weak signals (< -90 dBm)
            // Approximate conversion: signal % ≈ 2 * (dBm + 100)
            // So -90 dBm ≈ 20% signal strength
            if signal < 30 {
              return None;
            }

            return Some(WifiInfo {
              ssid,
              signal,
              security,
              connected,
            });
          }
          None
        })
        .collect();

      // Deduplicate by SSID, keeping the strongest signal
      use std::collections::HashMap;
      let mut dedup_map: HashMap<String, WifiInfo> = HashMap::new();

      for network in networks {
        let ssid = network.ssid.clone();
        dedup_map.entry(ssid)
          .and_modify(|existing| {
            // Prioritize connected network, otherwise keep stronger signal
            let should_replace = match (network.connected, existing.connected) {
              (true, false) => true,  // New is connected, existing isn't
              (false, true) => false, // Existing is connected, keep it
              _ => network.signal > existing.signal, // Both same connection status, use signal
            };

            if should_replace {
              *existing = network.clone();
            }
          })
          .or_insert(network);
      }

      let mut result: Vec<WifiInfo> = dedup_map.into_values().collect();

      // Sort with connected network first, then by signal strength (strongest first)
      result.sort_by(|a, b| {
        match (a.connected, b.connected) {
          (true, false) => std::cmp::Ordering::Less,
          (false, true) => std::cmp::Ordering::Greater,
          _ => b.signal.cmp(&a.signal),
        }
      });

      return result;
    }
  }

  Vec::new()
}



