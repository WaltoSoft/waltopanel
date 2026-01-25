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
        id if id.starts_with("connected-wifi-") => {
          // Connected network clicked - show disconnect confirmation
          let ssid = id.trim_start_matches("connected-wifi-");
          NetworkService::confirm_disconnect_wifi(ssid);
        }
        id if id.starts_with("wifi-") => {
          // Available network clicked - connect
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
    }
    else {
      self.refresh_network_enabled(&mut menu_index, menu.clone(), metrics);
    }

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
    if metrics.is_wifi_enabled {
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

        for wifi_info in connected_wifi.iter() {
          let icon_name = get_wifi_icon(wifi_info);
          let lock_icon = get_wifi_lock_icon(wifi_info);

          if menu.count() > 0 && *menu_index <= menu.count() - 1 {
            let model = menu.get(*menu_index).unwrap();
            update_menu_item_model(&model, &format!("connected-wifi-{}", wifi_info.ssid), wifi_info.ssid.clone(), Some(icon_name), false, false, false, TypedListStore::new(), false);
            model.set_post_label_icon_name(lock_icon.as_deref());
          }
          else {
            let model = MenuItemModel::new(&format!("connected-wifi-{}", wifi_info.ssid), &wifi_info.ssid);
            model.set_icon_name(Some(&icon_name));
            model.set_post_label_icon_name(lock_icon.as_deref());
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

    if metrics.is_wifi_enabled {
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
          let lock_icon = get_wifi_lock_icon(wifi_info);

          if submenu.count() > 0 && menu_index <= submenu.count() - 1 {
            let model = submenu.get(menu_index).unwrap();
            update_menu_item_model(&model, &format!("wifi-{}", wifi_info.ssid), wifi_info.ssid.clone(), Some(icon_name), false, false, false, TypedListStore::new(), false);
            model.set_post_label_icon_name(lock_icon.as_deref());
          }
          else {
            let model = MenuItemModel::new(&format!("wifi-{}", wifi_info.ssid), &wifi_info.ssid);
            model.set_disabled(false);
            model.set_allow_toggle(false);
            model.set_toggled(false);
            model.set_allow_toggle(false);
            model.set_icon_name(Some(&icon_name));
            model.set_post_label_icon_name(lock_icon.as_deref());
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
  // Use the same icons as the panel for visual consistency
  match network.signal {
    0..=25 => "network-wireless-signal-weak-symbolic",
    26..=50 => "network-wireless-signal-ok-symbolic",
    51..=75 => "network-wireless-signal-good-symbolic",
    76..=100 => "network-wireless-signal-excellent-symbolic",
    _ => "network-wireless-symbolic",
  }.to_string()
}

fn get_wifi_lock_icon(network: &WifiInfo) -> Option<String> {
  if !network.security.is_empty() && network.security != "open" {
    Some("system-lock-screen-symbolic".to_string())
  } else {
    None
  }
}
