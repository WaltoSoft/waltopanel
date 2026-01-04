use gtk::{Widget, glib::object::Cast, prelude::WidgetExt};

use crate::traits::CompositeWidget;
use crate::widgets::PanelButton;
use super::{BatteryService, BatteryMetrics};


pub struct BatteryButton {
  panel_button: PanelButton,
}

impl BatteryButton {
  pub fn new() -> Self {
    let metrics = BatteryService::start();
    let panel_button = PanelButton::new();

    Self::update_ui(&panel_button, metrics);

    let panel_button_clone = panel_button.clone();
    BatteryService::subscribe(move |metrics| {
      Self::update_ui(&panel_button_clone, metrics);
    });

    panel_button.connect_destroy(|_| {
      BatteryService::stop();
    });

    Self { panel_button }
  }

  fn update_ui(panel_button: &PanelButton, metrics: BatteryMetrics) {
    let icon_name = Self::get_battery_icon(metrics.percentage, metrics.plugged_in);
    panel_button.set_icon_name(&icon_name);
    panel_button.set_tooltip_text(Some(&format!("{}%", metrics.percentage)));
  }

  fn get_battery_icon(percentage: u8, plugged_in: bool) -> String {
    if plugged_in {
      // Charging icons
      match percentage {
        0..=10 => "battery-level-0-charging-symbolic",
        11..=20 => "battery-level-10-charging-symbolic",
        21..=30 => "battery-level-20-charging-symbolic",
        31..=40 => "battery-level-30-charging-symbolic",
        41..=50 => "battery-level-40-charging-symbolic",
        51..=60 => "battery-level-50-charging-symbolic",
        61..=70 => "battery-level-60-charging-symbolic",
        71..=80 => "battery-level-70-charging-symbolic",
        81..=90 => "battery-level-80-charging-symbolic",
        91..=99 => "battery-level-90-charging-symbolic",
        100 => "battery-level-100-charged-symbolic",
        _ => "battery-symbolic",
      }
    } else {
      // Discharging icons
      match percentage {
        0..=10 => "battery-level-0-symbolic",
        11..=20 => "battery-level-10-symbolic",
        21..=30 => "battery-level-20-symbolic",
        31..=40 => "battery-level-30-symbolic",
        41..=50 => "battery-level-40-symbolic",
        51..=60 => "battery-level-50-symbolic",
        61..=70 => "battery-level-60-symbolic",
        71..=80 => "battery-level-70-symbolic",
        81..=90 => "battery-level-80-symbolic",
        91..=100 => "battery-level-100-symbolic",
        _ => "battery-symbolic",
      }
    }.to_string()
  }
}

impl CompositeWidget for BatteryButton {
  fn widget(&self) -> Widget {
    self.panel_button.clone().upcast()
  }
}