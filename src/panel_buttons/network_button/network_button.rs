use gtk::{Widget, prelude::WidgetExt};
use gtk::glib::object::Cast;

use crate::models::MenuItemModel;
use crate::panel_buttons::network_button::network_service::WifiInfo;
use crate::types::TypedListStore;
use crate::traits::CompositeWidget;
use crate::widgets::PanelButton;
use super::network_service::{ConnectionType, NetworkMetrics, NetworkService};

#[derive(Clone)]
pub struct NetworkButton {
  panel_button: PanelButton,
  menu: TypedListStore<MenuItemModel>,
}

impl NetworkButton {
  pub fn new() -> Self {
    let initial_metrics = NetworkService::start(3); // Update every 3 seconds
    let panel_button = PanelButton::new();
    let menu = TypedListStore::<MenuItemModel>::new();
    panel_button.set_menu(menu.clone());  

    panel_button.connect_menu_item_clicked(move |_, item| {
      match item.id().as_str() {
        "enable-wifi" => {
          NetworkService::toggle_wifi(! item.toggled());
        }
        "enable-networking" => {
          NetworkService::toggle_networking(! item.toggled());
        }
        "airplane-mode" => {
          NetworkService::toggle_airplane_mode(! item.toggled());
        }
        "network-settings" => {
          //NetworkService::open_network_settings();
        }
        id if id.starts_with("wifi-") => {
          let ssid = id.trim_start_matches("wifi-");
          NetworkService::connect_to_wifi(ssid);
        }
        _ => {}
      }
    });

    let obj = Self {
      panel_button,
      menu,
    };

    obj.refresh_panel_button(&initial_metrics);
    let obj_clone = obj.clone();

    NetworkService::subscribe(move |metrics| {
      obj_clone.refresh_panel_button(&metrics);
    });

    obj
  }

  fn refresh_panel_button(&self, metrics: &NetworkMetrics) {
    self.refresh_panel_button_icon(metrics);
    self.refresh_tooltip_text(metrics);
    self.refresh_menu(metrics);
  }

  fn refresh_panel_button_icon(&self, metrics: &NetworkMetrics) {
    let new_icon_name = if !metrics.is_networking_enabled {
      "network-offline-symbolic".to_string()
    } else {
      match metrics.connection_type {
        ConnectionType::Ethernet => "network-wired-symbolic".to_string(),
        ConnectionType::Wifi => {
          match metrics.signal_strength {
            0..=25 => "network-wireless-signal-weak-symbolic",
            26..=50 => "network-wireless-signal-ok-symbolic",
            51..=75 => "network-wireless-signal-good-symbolic",
            76..=100 => "network-wireless-signal-excellent-symbolic",
            _ => "network-wireless-symbolic",
          }.to_string()
        }
        ConnectionType::Disconnected => {
          if metrics.is_wifi_enabled {
            "network-wireless-offline-symbolic".to_string()
          } else {
            "network-wireless-disabled-symbolic".to_string()
          }
        }
      }
    };

    if let Some(icon_name) = self.panel_button.icon_name() {
      if icon_name != new_icon_name {
        self.panel_button.set_icon_name(&new_icon_name);
      }
    } else {
      self.panel_button.set_icon_name(&new_icon_name);
    }
  }

  fn refresh_tooltip_text(&self, metrics: &NetworkMetrics) {
    let new_tooltip_text = get_tooltip_text(metrics);

    if let Some(tooltip_text) = self.panel_button.tooltip_text() {
      if tooltip_text != new_tooltip_text {
        self.panel_button.set_tooltip_text(Some(&new_tooltip_text));
      }
    } else {
      self.panel_button.set_tooltip_text(Some(&new_tooltip_text));
    }
  }

  fn refresh_menu(&self, metrics: &NetworkMetrics) {
    let mut menu_index = 0;
    let menu = self.menu.clone();
    
    if metrics.is_networking_enabled {
      self.refresh_ethernet_networks(&mut menu_index, menu.clone(), metrics);
      self.refresh_connected_wifi_networks(&mut menu_index, menu.clone(), metrics);
      self.refresh_available_wifi_networks(&mut menu_index, metrics);
      self.refresh_enable_wifi(&mut menu_index, menu.clone(), metrics);
      self.refresh_network_enabled(&mut menu_index, menu.clone(), metrics);
      self.refresh_airplane_mode(&mut menu_index, menu.clone(), metrics);
    }
    else {
      self.refresh_network_enabled(&mut menu_index, menu.clone(), metrics);
    }

    self.refresh_network_settings(&mut menu_index, menu.clone());

    while menu.count() > menu_index {
      menu.remove(menu_index);
    }    
  }

  fn refresh_ethernet_networks(&self, menu_index: &mut u32, menu: TypedListStore<MenuItemModel>, metrics: &NetworkMetrics) {
    if metrics.ethernet_connections.len() > 0 {
      if menu.count() > 0 && *menu_index <= menu.count() - 1 {
        let model = menu.get(*menu_index).unwrap();
        update_menu_item_model(&model,"ethernet-list", String::from("Ethernet Networks"), None, true, false, false, TypedListStore::new(), false);
      }
      else {
        let model = MenuItemModel::new("ethernet-list", "Ethernet Networks");
        model.set_disabled(true);
        model.set_allow_toggle(false);
        model.set_toggled(false);
        menu.append(model)
      }
      *menu_index += 1;


      let ethernet_count = metrics.ethernet_connections.len();
      for (i, ethernet_info) in metrics.ethernet_connections.iter().enumerate() {
        let is_last = i == ethernet_count - 1;

        if menu.count() > 0 && *menu_index <= menu.count() - 1 {
          let model = menu.get(*menu_index).unwrap();
          update_menu_item_model(&model, &format!("ethernet-{}", ethernet_info.name), format!("{} ({})", ethernet_info.name, if ethernet_info.connected { "Connected" } else { "Disconnected" }), None, !ethernet_info.connected, false, false, TypedListStore::new(), is_last);
        }
        else {
          let model = MenuItemModel::new(&format!("ethernet-{}", ethernet_info.name), &format!("{} ({})", ethernet_info.name, if ethernet_info.connected { "Connected" } else { "Disconnected" }));
          model.set_disabled(!ethernet_info.connected);
          model.set_allow_toggle(false);
          model.set_toggled(false);

          if is_last {
            model.set_separator_after(true);
          }

          menu.append(model);
        }

        *menu_index += 1;
      }
    }
  }

  fn refresh_connected_wifi_networks(&self, menu_index: &mut u32, menu: TypedListStore<MenuItemModel>, metrics: &NetworkMetrics) {
    if ! metrics.is_airplane_mode && metrics.is_wifi_enabled {
      if metrics.available_wifi_networks.len() > 0 {
        if menu.count() > 0 && *menu_index <= menu.count() - 1 {
          let model = menu.get(*menu_index).unwrap();
          update_menu_item_model(&model,"wifi-list", String::from("Wi-Fi Networks"), None, true, false, false, TypedListStore::new(), false);
        }
        else {
          let model = MenuItemModel::new("wifi-list", "Wi-Fi Networks");
          model.set_disabled(true);
          model.set_allow_toggle(false);
          model.set_toggled(false);
          model.set_separator_after(false);
          menu.append(model)
        }
        *menu_index += 1;

        let connected_wifi: Vec<_> = metrics.available_wifi_networks.iter().filter(|n| n.connected).collect();

        for (i, wifi_info) in connected_wifi.iter().enumerate() {
          let icon_name = get_wifi_icon(wifi_info);

          if menu.count() > 0 && *menu_index <= menu.count() - 1 {
            let model = menu.get(*menu_index).unwrap();
            update_menu_item_model(&model, &format!("wifi-{}", wifi_info.ssid), format!("{}", wifi_info.ssid), Some(icon_name), false, false, false, TypedListStore::new(), false);
          }
          else {
            let model = MenuItemModel::new(&format!("wifi-{}", wifi_info.ssid), &format!("{}", wifi_info.ssid));
            model.set_icon_name(Some(&icon_name));
            model.set_disabled(false);
            model.set_allow_toggle(false);
            model.set_toggled(false);
            model.set_separator_after(false);
            menu.append(model);
          }

          *menu_index += 1;
        }
      }
    }
  }

  fn refresh_available_wifi_networks(&self, parent_menu_index: &mut u32, metrics: &NetworkMetrics) {
    let mut menu_index: u32 = 0;
    let menu = self.menu.clone();

    if ! metrics.is_airplane_mode && metrics.is_wifi_enabled {
      let not_connected_wifi: Vec<_> = metrics.available_wifi_networks.iter().filter(|n| !n.connected).collect();

      let submenu = if menu.count() > 0 && *parent_menu_index <= menu.count() - 1 {
        let model = menu.get(*parent_menu_index).unwrap();
        model.set_id("wifi-available-list");
        model.set_text("Available Networks");
        model.set_disabled(false);
        model.set_allow_toggle(false);
        model.set_toggled(false);
        model.set_separator_after(false);
        model.submenu()
      }
      else {
        let model = MenuItemModel::new("wifi-available-list", "Available Networks");
        model.set_disabled(false);
        model.set_allow_toggle(false);
        model.set_toggled(false);
        model.set_separator_after(false);
        menu.append(model.clone());
        model.submenu()
      };
      *parent_menu_index += 1;

      if not_connected_wifi.len() > 0 {
        for wifi_info in not_connected_wifi.iter() {
          let icon_name = get_wifi_icon(wifi_info);

          if submenu.count() > 0 && menu_index <= submenu.count() - 1 {
            let model = submenu.get(menu_index).unwrap();
            update_menu_item_model(&model, &format!("wifi-{}", wifi_info.ssid), format!("{}", wifi_info.ssid), Some(icon_name), false, false, false, TypedListStore::new(), false);
          }
          else {
            let model = MenuItemModel::new(&format!("wifi-{}", wifi_info.ssid), &format!("{}", wifi_info.ssid));
            model.set_disabled(false);
            model.set_allow_toggle(false);
            model.set_toggled(false);
            model.set_allow_toggle(false);
            model.set_icon_name(Some(&icon_name));
            submenu.append(model);
          }

          menu_index += 1;
        }
      }

      // Remove any extra menu items beyond the current menu_index
      while submenu.count() > menu_index {
        submenu.remove(menu_index);
      }
    }
  }

  fn refresh_enable_wifi(&self, menu_index: &mut u32, menu: TypedListStore<MenuItemModel>, metrics: &NetworkMetrics) {
    if menu.count() > 0 && *menu_index <= menu.count() - 1 {
      let model = menu.get(*menu_index).unwrap();
      update_menu_item_model(&model,"enable-wifi", String::from("Enable WiFi"), None, false, true, metrics.is_wifi_enabled, TypedListStore::new(), true);
    }
    else {
      let model = MenuItemModel::new("enable-wifi", "Enable WiFi");
      model.set_disabled(false);
      model.set_allow_toggle(true);
      model.set_toggled(metrics.is_wifi_enabled);
      model.set_separator_after(true);
      menu.append(model)
    }
    *menu_index += 1;
  }

  fn refresh_network_enabled(&self, menu_index: &mut u32, menu: TypedListStore<MenuItemModel>, metrics: &NetworkMetrics) {
    if ! metrics.is_networking_enabled {
      if menu.count() > 0 && *menu_index <= menu.count() - 1 {
        let model = menu.get(*menu_index).unwrap();
        update_menu_item_model(&model,"network-disabled", String::from("Network Disabled"), None, true, false, false, TypedListStore::new(), false);
      }
      else {
        let model = MenuItemModel::new("network-disabled", "Network Disabled");
        model.set_disabled(true);
        model.set_allow_toggle(false);
        model.set_toggled(false);
        model.set_separator_after(false);
        menu.append(model)
      }
      *menu_index += 1;
    }

    if menu.count() > 0 && *menu_index <= menu.count() - 1 {
      let model = menu.get(*menu_index).unwrap();
      update_menu_item_model(&model,"enable-networking", String::from("Enable Networking"), None, false, true, metrics.is_networking_enabled, TypedListStore::new(), false);
    }
    else {
      let model = MenuItemModel::new("enable-networking", "Enable Networking");
      model.set_disabled(false);
      model.set_allow_toggle(true);
      model.set_toggled(metrics.is_networking_enabled);
      model.set_separator_after(false);
      menu.append(model)
    }
    *menu_index += 1;
  }

  fn refresh_airplane_mode(&self, menu_index: &mut u32, menuy: TypedListStore<MenuItemModel>, metrics: &NetworkMetrics) {
    if menuy.count() > 0 && *menu_index <= menuy.count() - 1 {
      let model = menuy.get(*menu_index).unwrap();
      update_menu_item_model(&model,"airplane-mode", String::from("Airplane Mode"), None, false, true, metrics.is_airplane_mode, TypedListStore::new(), true);
    }
    else {
      let model = MenuItemModel::new("airplane-mode", "Airplane Mode");
      model.set_disabled(false);
      model.set_allow_toggle(true);
      model.set_toggled(metrics.is_airplane_mode);
      model.set_separator_after(true);
      menuy.append(model)
    }
    *menu_index += 1;
  }

  fn refresh_network_settings(&self, menu_index: &mut u32, menuy: TypedListStore<MenuItemModel>) {
    let icon_name = "preferences-system-network".to_string();

    if menuy.count() > 0 && *menu_index <= menuy.count() - 1 {
      let model = menuy.get(*menu_index).unwrap();
      update_menu_item_model(&model,"network-settings", String::from("Network Settings"), Some(icon_name), false, false, false, TypedListStore::new(), false);
    }
    else {
      let model = MenuItemModel::new("network-settings", "Network Settings");
      model.set_disabled(false);
      model.set_icon_name(Some(&icon_name));
      model.set_allow_toggle(false);
      model.set_toggled(false);
      model.set_separator_after(false);
      menuy.append(model)
    }
    *menu_index += 1;
  }
}

impl CompositeWidget for NetworkButton {
  fn widget(&self) -> Widget {
    self.panel_button.clone().upcast()
  }
}

fn get_tooltip_text(metrics: &NetworkMetrics) -> String {
  match metrics.connection_type {
    ConnectionType::Ethernet => {
      format!("Connected to {}", metrics.connection_name)
    }
    ConnectionType::Wifi => {
      format!(
        "Connected to {}\nSignal: {}%",
        metrics.connection_name,
        metrics.signal_strength
      )
    }
    ConnectionType::Disconnected => {
      if metrics.is_wifi_enabled {
        "Not connected".to_string()
      } else {
        "WiFi disabled".to_string()
      }
    }
  }
}

fn update_menu_item_model(model: &MenuItemModel, id: &str, text: String, icon_name: Option<String>, disabled: bool, allow_toggle: bool, toggled: bool, submenu: TypedListStore<MenuItemModel>, separator_after: bool) {
  if model.id() != id {
    model.set_id(id);
  }

  if model.text() != text {
    model.set_text(&text);
  }

  if model.icon_name() != icon_name {
    model.set_icon_name(icon_name.as_deref());
  }

  if model.disabled() != disabled {
    model.set_disabled(disabled);
  }

  if model.allow_toggle() != allow_toggle {
    model.set_allow_toggle(allow_toggle);
  }

  if model.toggled() != toggled {
    model.set_toggled(toggled);
  }

  model.set_submenu(submenu.as_list_store().clone());

  if model.separator_after() != separator_after {
    model.set_separator_after(separator_after);
  }
}

fn get_wifi_icon(network: &WifiInfo) -> String {
  // Map signal strength to NetworkManager icon levels (0, 25, 50, 75, 100)
  let signal_level = match network.signal {
    0..=12 => "00",
    13..=37 => "25",
    38..=62 => "50",
    63..=87 => "75",
    88..=100 => "100",
    _ => "50",
  };

  if ! network.security.is_empty() {
    format!("nm-signal-{}-secure-symbolic", signal_level)
  } else {
    format!("nm-signal-{}-symbolic", signal_level)
  }
}



/*
use gtk::{Widget, glib::object::Cast, prelude::WidgetExt};

use crate::{models::{MenuBuilder, MenuItemModel}, panel_buttons::network_button::network_service::WifiInfo, traits::CompositeWidget, types::TypedListStore, widgets::PanelButton};
use super::network_service::{ConnectionType, NetworkMetrics, NetworkService};

#[derive(Clone)]
pub struct NetworkButton {
  panel_button: PanelButton,
  menu: TypedListStore<MenuItemModel>,
}

impl NetworkButton {
  pub fn new() -> Self {
    let metrics = NetworkService::start(3);  // Update every 3 seconds
    let panel_button = PanelButton::new();

    update_ui(&panel_button, &metrics);

    let menu = Self::build_menu_static(&metrics);
    panel_button.set_menu(menu.clone());

    let obj = Self{
      panel_button,
      menu,
    };

    let obj_clone = obj.clone();

    NetworkService::subscribe(move |metrics| {
      update_ui(&obj_clone.panel_button, &metrics);
      let new_menu = Self::build_menu_static(&metrics);
      obj_clone.adjust_menu(&new_menu);
    });

    obj
  }

  fn adjust_menu(&self, menu: &TypedListStore<MenuItemModel>) {
    let new_count = menu.count();
    let current_count = self.menu.count();

    // Update existing items by comparing IDs and syncing properties
    let min_count = new_count.min(current_count);
    for i in 0..min_count {
      if let (Some(new_item), Some(current_item)) = (menu.get(i), self.menu.get(i)) {
        // If IDs match, update the current item's properties
        if new_item.id() == current_item.id() {
          current_item.set_text(&new_item.text());
          current_item.set_icon_name(new_item.icon_name().as_deref());
          current_item.set_toggled(new_item.toggled());
          current_item.set_allow_toggle(new_item.allow_toggle());
          current_item.set_separator_after(new_item.separator_after());
          current_item.set_disabled(new_item.disabled());

          // Sync submenu if present - recursively adjust instead of replacing
          if new_item.has_submenu() {
            let current_submenu = current_item.submenu();
            let new_submenu = new_item.submenu();
            Self::adjust_submenu(&current_submenu, &new_submenu);
          }
        } else {
          // IDs don't match, replace the item at this position
          self.menu.remove(i);
          self.menu.insert(i, &new_item);
        }
      }
    }

    // Remove extra items if new menu is shorter
    if current_count > new_count {
      for _ in new_count..current_count {
        self.menu.remove(new_count);
      }
    }

    // Add new items if new menu is longer
    if new_count > current_count {
      for i in current_count..new_count {
        if let Some(new_item) = menu.get(i) {
          self.menu.append(&new_item);
        }
      }
    }
  }

  fn adjust_submenu(current: &TypedListStore<MenuItemModel>, new: &TypedListStore<MenuItemModel>) {
    let new_count = new.count();
    let current_count = current.count();

    // Update existing items by comparing IDs and syncing properties
    let min_count = new_count.min(current_count);
    for i in 0..min_count {
      if let (Some(new_item), Some(current_item)) = (new.get(i), current.get(i)) {
        // If IDs match, update the current item's properties
        if new_item.id() == current_item.id() {
          current_item.set_text(&new_item.text());
          current_item.set_icon_name(new_item.icon_name().as_deref());
          current_item.set_toggled(new_item.toggled());
          current_item.set_allow_toggle(new_item.allow_toggle());
          current_item.set_separator_after(new_item.separator_after());
          current_item.set_disabled(new_item.disabled());

          // Recursively adjust nested submenus
          if new_item.has_submenu() {
            let current_submenu = current_item.submenu();
            let new_submenu = new_item.submenu();
            Self::adjust_submenu(&current_submenu, &new_submenu);
          }
        } else {
          // IDs don't match, replace the item at this position
          current.remove(i);
          current.insert(i, &new_item);
        }
      }
    }

    // Remove extra items if new submenu is shorter
    if current_count > new_count {
      for _ in new_count..current_count {
        current.remove(new_count);
      }
    }

    // Add new items if new submenu is longer
    if new_count > current_count {
      for i in current_count..new_count {
        if let Some(new_item) = new.get(i) {
          current.append(&new_item);
        }
      }
    }
  }

  fn build_menu_static(metrics: &NetworkMetrics) -> TypedListStore<MenuItemModel> {
    let builder = if metrics.networking_enabled {
      let mut inside_builder = MenuBuilder::new()
        .item_if(metrics.ethernet_connections.len() > 0, "ethernet-title", "Ethernet Network")
        .disabled();

      for eth in &metrics.ethernet_connections {
        let status = if eth.connected { "Connected" } else { "Disconnected" };
        let label = format!("{} ({})", eth.name, status);
        inside_builder = inside_builder
          .item(format!("ethernet-{}", eth.name), label)
          .disabled_if(!eth.connected);
      }

      inside_builder = inside_builder.separator();

      inside_builder = inside_builder
        .item("wifi-title", "WiFi Networks")
        .disabled();


      if metrics.wifi_enabled {
        let connected_wifi = get_connected_wifi(&metrics.available_wifi_networks);
        
        if let Some(wifi) = connected_wifi {
          inside_builder = inside_builder
            .item("connected-wifi", &wifi.ssid)
            .icon(get_wifi_icon(&wifi))
            .item(format!("wifi-disconnect-{}", &wifi.ssid), "Disconnect");
        }
        else {
          inside_builder = inside_builder
            .item("no-wifi", "Not connected")
            .disabled();
        }

        let available_networks_clone = metrics.available_wifi_networks.clone();
        
        inside_builder = inside_builder
          .item("available-networks", "Available Networks")
          .submenu(move |builder| {
            build_available_wifis_submenu(builder, &available_networks_clone)
          })
          .separator()
          .item("enable-networking", "Enable Networking")
          .toggled(metrics.networking_enabled)
          .item("wifi-enabled", "Enable WiFi")
          .toggled(metrics.wifi_enabled)
        
      }
      else {
        inside_builder = inside_builder
          .item("no-wifi", "WiFi Disabled")
          .disabled();
      }

      inside_builder
    }
    else {
      MenuBuilder::new()
        .item("network-disabled", "Networking Disabled")
        .disabled()
    };

    builder.build()
  }    


}

fn build_available_wifis_submenu(
  builder: MenuBuilder,
  available_wifis: &[WifiInfo]
) -> TypedListStore<MenuItemModel> {
  // Filter out the currently connected network
  let filtered: Vec<&WifiInfo> = available_wifis.iter().filter(|n| !n.connected).collect();

  if filtered.is_empty() {
    return builder.item("no-networks", "No networks available").build();
  }

  let first = filtered[0];
  let icon = get_wifi_icon(first);
  let label = first.ssid.clone();

  let mut item_builder = builder
    .item(format!("network-{}", first.ssid), label)
    .icon(&icon);

  for wifi in &filtered[1..] {
    let icon = get_wifi_icon(wifi);
    let label = wifi.ssid.clone();

    item_builder = item_builder
      .item(format!("network-{}", wifi.ssid), label)
      .icon(&icon);
  }

  item_builder.build()
}



impl CompositeWidget for NetworkButton {
  fn widget(&self) -> Widget {
    self.panel_button.clone().upcast()
  }
}

fn update_ui(panel_button: &PanelButton, metrics: &NetworkMetrics) {
  let icon_name = get_panel_button_icon(metrics);
  panel_button.set_icon_name(&icon_name);

  let tooltip_text = get_tooltip_text(metrics);
  panel_button.set_tooltip_text(Some(&tooltip_text));
}

fn get_connected_wifi(wifi_networks: &Vec<WifiInfo>) -> Option<WifiInfo> {
  wifi_networks
    .iter()
    .find(|network| network.connected)
    .cloned()
}

fn get_panel_button_icon(metrics: &NetworkMetrics) -> String {
  match metrics.connection_type {
    ConnectionType::Ethernet => "network-wired-symbolic".to_string(),
    ConnectionType::Wifi => {
      match metrics.signal_strength {
        0..=25 => "network-wireless-signal-weak-symbolic",
        26..=50 => "network-wireless-signal-ok-symbolic",
        51..=75 => "network-wireless-signal-good-symbolic",
        76..=100 => "network-wireless-signal-excellent-symbolic",
        _ => "network-wireless-symbolic",
      }.to_string()
    }
    ConnectionType::Disconnected => {
      if metrics.wifi_enabled {
        "network-wireless-offline-symbolic".to_string()
      } else {
        "network-wireless-disabled-symbolic".to_string()
      }
    }
  }
}

fn get_wifi_icon(network: &WifiInfo) -> String {
  // Map signal strength to NetworkManager icon levels (0, 25, 50, 75, 100)
  let signal_level = match network.signal {
    0..=12 => "00",
    13..=37 => "25",
    38..=62 => "50",
    63..=87 => "75",
    88..=100 => "100",
    _ => "50",
  };

  if ! network.security.is_empty() {
    format!("nm-signal-{}-secure-symbolic", signal_level)
  } else {
    format!("nm-signal-{}-symbolic", signal_level)
  }
}

fn get_tooltip_text(metrics: &NetworkMetrics) -> String {
  match metrics.connection_type {
    ConnectionType::Ethernet => {
      format!("Connected to {}", metrics.connection_name)
    }
    ConnectionType::Wifi => {
      format!(
        "Connected to {}\nSignal: {}%",
        metrics.connection_name,
        metrics.signal_strength
      )
    }
    ConnectionType::Disconnected => {
      if metrics.wifi_enabled {
        "Not connected".to_string()
      } else {
        "WiFi disabled".to_string()
      }
    }
  }
}
*/



/*
use gtk::{Widget, glib::object::Cast, prelude::WidgetExt};
use std::cell::RefCell;
use std::rc::Rc;

use crate::traits::CompositeWidget;
use crate::widgets::PanelButton;
use crate::models::{MenuItemModel, MenuBuilder};
use crate::types::TypedListStore;
use super::NetworkService;
use super::network_service::{NetworkMetrics, ConnectionType, WifiInfo, EthernetInfo};

pub struct NetworkButton {
  panel_button: PanelButton,
  ethernet_items: Rc<RefCell<Vec<MenuItemModel>>>,
  connected_wifi_items: Rc<RefCell<Vec<MenuItemModel>>>,
  networks_submenu: Rc<RefCell<Option<MenuItemModel>>>,
  wifi_toggle_item: Rc<RefCell<Option<MenuItemModel>>>,
}

impl NetworkButton {
  pub fn new() -> Self {
    let metrics = NetworkService::start();
    let panel_button = PanelButton::new();
    let ethernet_items: Rc<RefCell<Vec<MenuItemModel>>> = Rc::new(RefCell::new(Vec::new()));
    let connected_wifi_items: Rc<RefCell<Vec<MenuItemModel>>> = Rc::new(RefCell::new(Vec::new()));
    let networks_submenu: Rc<RefCell<Option<MenuItemModel>>> = Rc::new(RefCell::new(None));
    let wifi_toggle_item: Rc<RefCell<Option<MenuItemModel>>> = Rc::new(RefCell::new(None));

    // Build initial menu
    let menu = Self::build_menu(&metrics, &ethernet_items, &connected_wifi_items, &networks_submenu, &wifi_toggle_item);
    panel_button.set_menu(menu);

    // Set initial icon and tooltip
    Self::update_icon_and_tooltip(&panel_button, &metrics);

    // Subscribe to network changes
    let panel_button_clone = panel_button.clone();
    let ethernet_items_clone = ethernet_items.clone();
    let connected_wifi_items_clone = connected_wifi_items.clone();
    let networks_submenu_clone = networks_submenu.clone();
    let wifi_toggle_item_clone = wifi_toggle_item.clone();
  
    NetworkService::subscribe(move |metrics| {
      Self::update_ui(&panel_button_clone, &ethernet_items_clone, &connected_wifi_items_clone, &networks_submenu_clone, &wifi_toggle_item_clone, &metrics);
    });

    // Handle menu item clicks
    let panel_button_clone2 = panel_button.clone();
    panel_button.connect_menu_item_clicked(move |pb, item| {
      Self::handle_menu_click(pb, item);
    });

    // Clean up on destroy
    panel_button.connect_destroy(|_| {
      NetworkService::stop();
    });

    Self {
      panel_button,
      ethernet_items,
      connected_wifi_items,
      networks_submenu,
      wifi_toggle_item,
    }
  }

  fn build_menu(
    metrics: &NetworkMetrics,
    ethernet_items_ref: &Rc<RefCell<Vec<MenuItemModel>>>,
    connected_wifi_items_ref: &Rc<RefCell<Vec<MenuItemModel>>>,
    networks_submenu_ref: &Rc<RefCell<Option<MenuItemModel>>>,
    wifi_toggle_ref: &Rc<RefCell<Option<MenuItemModel>>>
  ) -> TypedListStore<MenuItemModel> {
    let available_networks = metrics.available_networks.clone();
    let ethernet_connections = metrics.ethernet_connections.clone();
    let wifi_enabled = metrics.wifi_enabled;

    // Build the menu starting with ethernet section
    let mut menu_builder = MenuBuilder::new()
      .item("ethernet-title", "Ethernet Network")
      .disabled();

    // Add ethernet connections
    let mut ethernet_index = 1;
    for eth in &ethernet_connections {
      let status = if eth.enabled { "Connected" } else { "Disconnected" };
      let label = format!("{} ({})", eth.name, status);
  
      menu_builder = menu_builder
        .item(format!("ethernet-{}", eth.name), label);
  
      ethernet_index += 1;
    }

    menu_builder = menu_builder.separator();

    menu_builder = menu_builder
      .item("wifi-title", "WiFi Networks")
      .disabled();

    let connected_networks: Vec<&WifiInfo> = available_networks.iter().filter(|n| n.in_use).collect();

    for network in &connected_networks {
      let icon = Self::get_network_item_icon(network);

      menu_builder = menu_builder
        .item(format!("connected-network-{}", network.ssid), &network.ssid)
        .icon(&icon);
    }

    // Add Available Networks submenu
    let available_networks_clone = available_networks.clone();
    menu_builder = menu_builder
      .item("available-networks", "Available Networks")
        .submenu(move |builder| {
          Self::build_networks_submenu(builder, &available_networks_clone)
        })
        .separator()
      .item("toggle-wifi", "Enable WiFi")
      .toggle();

    // Set the toggle state based on wifi_enabled
    if wifi_enabled {
      menu_builder = menu_builder.toggled_on();
    }

    let menu = menu_builder
      .item("network-settings", "Network Settings")
      .build();

/*     // Store references to ethernet items (indices 1 through separator_index-1)
    let mut ethernet_items = Vec::new();
    for i in 1..separator_index {
      if let Some(item) = menu.get(i) {
        ethernet_items.push(item);
      }
    }
    *ethernet_items_ref.borrow_mut() = ethernet_items;

    // Store references to connected WiFi network items (indices wifi_title_index+1 through wifi_title_index+connected_count)
    let mut connected_wifi_items = Vec::new();
    for i in 0..connected_networks.len() {
      if let Some(item) = menu.get(wifi_title_index + 1 + i as u32) {
        connected_wifi_items.push(item);
      }
    }
    *connected_wifi_items_ref.borrow_mut() = connected_wifi_items;

    // Calculate index for Available Networks (after wifi title + connected networks)
    let available_networks_index = wifi_title_index + 1 + connected_networks.len() as u32;

    // Store reference to the "Available Networks" menu item
    if let Some(networks_item) = menu.get(available_networks_index) {
      *networks_submenu_ref.borrow_mut() = Some(networks_item);
    }

    // Store reference to the WiFi toggle item (comes after Available Networks and a separator)
    if let Some(wifi_item) = menu.get(available_networks_index + 2) {
      *wifi_toggle_ref.borrow_mut() = Some(wifi_item);
    } */

    menu
  }

  fn build_networks_submenu(
    builder: MenuBuilder,
    networks: &[WifiInfo]
  ) -> TypedListStore<MenuItemModel> {
    // Filter out the currently connected network
    let available: Vec<&WifiInfo> = networks.iter().filter(|n| !n.in_use).collect();

    if available.is_empty() {
      return builder.item("no-networks", "No networks available").build();
    }

    let first = available[0];
    let icon = Self::get_network_item_icon(first);
    let label = first.ssid.clone();

    let mut item_builder = builder
      .item(format!("network-{}", first.ssid), label)
      .icon(&icon);

    for network in &available[1..] {
      let icon = Self::get_network_item_icon(network);
      let label = network.ssid.clone();

      item_builder = item_builder
        .item(format!("network-{}", network.ssid), label)
        .icon(&icon);
    }

    item_builder.build()
  }

  fn get_network_item_icon(network: &WifiInfo) -> String {
    // Map signal strength to NetworkManager icon levels (0, 25, 50, 75, 100)
    let signal_level = match network.signal {
      0..=12 => "00",
      13..=37 => "25",
      38..=62 => "50",
      63..=87 => "75",
      88..=100 => "100",
      _ => "50",
    };

    if !network.security.is_empty() {
      format!("nm-signal-{}-secure-symbolic", signal_level)
    } else {
      format!("nm-signal-{}-symbolic", signal_level)
    }
  }

  fn update_ui(
    panel_button: &PanelButton,
    ethernet_items_ref: &Rc<RefCell<Vec<MenuItemModel>>>,
    connected_wifi_items_ref: &Rc<RefCell<Vec<MenuItemModel>>>,
    networks_submenu_ref: &Rc<RefCell<Option<MenuItemModel>>>,
    wifi_toggle_ref: &Rc<RefCell<Option<MenuItemModel>>>,
    metrics: &NetworkMetrics
  ) {
    // Update icon and tooltip
    Self::update_icon_and_tooltip(panel_button, metrics);

    // Update ethernet items
    Self::update_ethernet_items(ethernet_items_ref, &metrics.ethernet_connections);

    // Update connected WiFi network items
    Self::update_connected_wifi_items(connected_wifi_items_ref, &metrics.available_networks);

    // Update the networks submenu
    Self::update_networks_submenu(networks_submenu_ref, &metrics.available_networks);

    // Update WiFi toggle state
    Self::update_wifi_toggle(wifi_toggle_ref, metrics.wifi_enabled);
  }

  fn update_icon_and_tooltip(panel_button: &PanelButton, metrics: &NetworkMetrics) {
    let icon_name = Self::get_network_icon(&metrics);
    panel_button.set_icon_name(&icon_name);

    let tooltip = Self::get_tooltip_text(&metrics);
    panel_button.set_tooltip_text(Some(&tooltip));
  }

  fn update_ethernet_items(
    ethernet_items_ref: &Rc<RefCell<Vec<MenuItemModel>>>,
    ethernet_connections: &[EthernetInfo]
  ) {
    let ethernet_items = ethernet_items_ref.borrow();

    // Update each ethernet item's text based on current status
    for (i, eth) in ethernet_connections.iter().enumerate() {
      if let Some(item) = ethernet_items.get(i) {
        let status = if eth.enabled { "Connected" } else { "Disconnected" };
        let label = format!("{} ({})", eth.name, status);
        item.set_text(&label);
      }
    }
  }

  fn update_connected_wifi_items(
    connected_wifi_items_ref: &Rc<RefCell<Vec<MenuItemModel>>>,
    networks: &[WifiInfo]
  ) {
    let connected_wifi_items = connected_wifi_items_ref.borrow();
    let connected_networks: Vec<&WifiInfo> = networks.iter().filter(|n| n.in_use).collect();

    // Update each connected WiFi item's text and icon
    for (i, network) in connected_networks.iter().enumerate() {
      if let Some(item) = connected_wifi_items.get(i) {
        item.set_text(&network.ssid);
        let icon = Self::get_network_item_icon(network);
        item.set_icon_name(Some(&icon));
      }
    }
  }

  fn update_networks_submenu(
    networks_submenu_ref: &Rc<RefCell<Option<MenuItemModel>>>,
    networks: &[WifiInfo]
  ) {
    if let Some(ref networks_item) = *networks_submenu_ref.borrow() {
      // Clear existing submenu
      let submenu = networks_item.submenu();
      let count = submenu.count();
      for _ in 0..count {
        submenu.remove(0);
      }

      // Filter out the currently connected network
      let available: Vec<&WifiInfo> = networks.iter().filter(|n| !n.in_use).collect();

      // Add updated networks
      if available.is_empty() {
        let no_networks = MenuItemModel::new("no-networks", "No networks available");
        submenu.append(&no_networks);
      } else {
        for network in available {
          let icon = Self::get_network_item_icon(network);
          let label = network.ssid.clone();
          let item = MenuItemModel::new(&format!("network-{}", network.ssid), &label);
          item.set_icon_name(Some(&icon));
          submenu.append(&item);
        }
      }
    }
  }

  fn update_wifi_toggle(
    wifi_toggle_ref: &Rc<RefCell<Option<MenuItemModel>>>,
    wifi_enabled: bool
  ) {
    if let Some(ref wifi_item) = *wifi_toggle_ref.borrow() {
      wifi_item.set_toggled(wifi_enabled);
    }
  }

  fn handle_menu_click(panel_button: &PanelButton, item: &MenuItemModel) {
    let id = item.id();

    match id.as_str() {
      "toggle-wifi" => {
        NetworkService::toggle_wifi();
        panel_button.hide_menu();
      }
      "network-settings" => {
        NetworkService::open_network_settings();
        panel_button.hide_menu();
      }
      id if id.starts_with("network-") => {
        let ssid = id.strip_prefix("network-").unwrap();

        // Check if this network is already connected
        if let Some(metrics) = NetworkService::get_current_state() {
          let is_connected = metrics.available_networks.iter()
            .any(|n| n.ssid == ssid && n.in_use);

          // Only attempt to connect if not already connected
          if !is_connected {
            NetworkService::connect_to_network(ssid);
          }
        }

        panel_button.hide_menu();
      }
      _ => {}
    }
  }

  fn get_network_icon(metrics: &NetworkMetrics) -> String {
    match metrics.connection_type {
      ConnectionType::Ethernet => "network-wired-symbolic".to_string(),
      ConnectionType::Wifi => {
        // Show signal strength for WiFi
        Self::get_wifi_icon(metrics.signal_strength)
      }
      ConnectionType::Disconnected => {
        if metrics.wifi_enabled {
          "network-wireless-offline-symbolic".to_string()
        } else {
          "network-wireless-disabled-symbolic".to_string()
        }
      }
    }
  }

  fn get_wifi_icon(signal: u8) -> String {
    match signal {
      0..=25 => "network-wireless-signal-weak-symbolic",
      26..=50 => "network-wireless-signal-ok-symbolic",
      51..=75 => "network-wireless-signal-good-symbolic",
      76..=100 => "network-wireless-signal-excellent-symbolic",
      _ => "network-wireless-symbolic",
    }.to_string()
  }

  fn get_tooltip_text(metrics: &NetworkMetrics) -> String {
    match metrics.connection_type {
      ConnectionType::Ethernet => {
        format!("Connected to {}", metrics.connection_name)
      }
      ConnectionType::Wifi => {
        format!(
          "Connected to {}\nSignal: {}%",
          metrics.connection_name,
          metrics.signal_strength
        )
      }
      ConnectionType::Disconnected => {
        if metrics.wifi_enabled {
          "Not connected".to_string()
        } else {
          "WiFi disabled".to_string()
        }
      }
    }
  }
}

impl CompositeWidget for NetworkButton {
  fn widget(&self) -> Widget {
    self.panel_button.clone().upcast()
  }
}
*/