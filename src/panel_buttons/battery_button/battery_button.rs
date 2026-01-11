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

    panel_button.add_css_class("panelbutton-rotated");

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

    let tooltip_text = match &metrics.estimated_time {
      Some(time) => format!("Battery: {}%\n{}", metrics.percentage, time),
      None => format!("Battery: {}%", metrics.percentage),
    };

    panel_button.set_tooltip_text(Some(&tooltip_text));
  }

  fn get_battery_icon(percentage: u8, plugged_in: bool) -> String {
    if plugged_in {
      // Charging icons using descriptive names
      match percentage {
        0..=10 => "battery-empty-charging-symbolic",
        11..=20 => "battery-caution-charging-symbolic",
        21..=40 => "battery-low-charging-symbolic",
        41..=80 => "battery-good-charging-symbolic",
        81..=99 => "battery-full-charging-symbolic",
        100 => "battery-full-charged-symbolic",
        _ => "battery-symbolic",
      }
    } else {
      // Discharging icons using descriptive names
      match percentage {
        0..=10 => "battery-empty-symbolic",
        11..=20 => "battery-caution-symbolic",
        21..=40 => "battery-low-symbolic",
        41..=80 => "battery-good-symbolic",
        81..=100 => "battery-full-symbolic",
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