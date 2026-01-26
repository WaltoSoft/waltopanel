use adw::ApplicationWindow;
use adw::prelude::AdwApplicationWindowExt;
use gtk::Align;
use gtk::CenterBox;
use gtk::Orientation;
use gtk::prelude::*;
use gtk::Box;
use std::boxed::Box as StdBox;

use crate::config::{WaltoPanelConfig, Margins};
use crate::panel_buttons::ClockButton;
use crate::traits::ApplicationWindowExtensions;
use crate::traits::CompositeWidget;

pub struct SystemPanel {
  pub window: ApplicationWindow,
}

impl SystemPanel {
  pub fn new_with_monitor(app: &adw::Application, monitor_name: String) -> Result<Self, StdBox<dyn std::error::Error>> {
    Self::with_config(app, WaltoPanelConfig::default(), Some(monitor_name))
  }

  fn with_config(
    app: &adw::Application,
    config: WaltoPanelConfig,
    monitor_name: Option<String>,
  ) -> Result<Self, StdBox<dyn std::error::Error>> {
    let window = ApplicationWindow::builder()
      .application(app)
      .title("WaltoPanel")
      .build();

    let _ = window.configure_top_layer_shell(config.height);

    let panel_box =
      Self::create_panel_container(config.button_spacing, &config.margins, monitor_name);

    window.set_content(Some(&panel_box));

    let system_panel = Self {
      window
    };

    Ok(system_panel)
  }
  
  fn create_panel_container(
    spacing: i32,
    margins: &Margins,
    monitor_name: Option<String>,
  ) -> gtk::CenterBox {
    let left_box = Box::builder()
      .orientation(Orientation::Horizontal)
      .spacing(spacing)
      .halign(Align::Start)
      .hexpand(true)
      .css_classes(vec!["panel-container-box"])
      .build();

    let center_box = Box::builder()
      .orientation(Orientation::Horizontal)
      .spacing(spacing)
      .halign(Align::Center)
      .hexpand(true)
      .css_classes(vec!["panel-container-box"])
      .build();

    let right_box = Box::builder()
      .orientation(Orientation::Horizontal)
      .spacing(spacing)
      .halign(Align::End)
      .hexpand(true)
      .css_classes(vec!["panel-container-box"])
      .build();

    let panel_box = CenterBox::builder()
      .orientation(Orientation::Horizontal)
      .margin_start(margins.left)
      .margin_end(margins.right)
      .margin_top(margins.top)
      .margin_bottom(margins.bottom)
      .build();

    let launch_button = crate::panel_buttons::LaunchButton::from_icon_name("view-app-grid-symbolic", "pkill rofi || rofi -show drun");
    let launch_widget = launch_button.widget();
    left_box.append(&launch_widget);

    if let Some(monitor_name) = monitor_name {
      let workspace_button = crate::panel_buttons::WorkspaceButton::new_with_monitor(monitor_name);
      left_box.append(&workspace_button.widget());
    }

    let clock_button = ClockButton::new();
    center_box.append(&clock_button.widget());

    let system_metrics_button = crate::panel_buttons::SystemMetricsButton::new();
    right_box.append(&system_metrics_button.widget());

    let network_button = crate::panel_buttons::NetworkButton::new();
    right_box.append(&network_button.widget());

    let sound_button = crate::panel_buttons::SoundButton::new();
    right_box.append(&sound_button.widget());

    let battery_button = crate::panel_buttons::BatteryButton::new();
    right_box.append(&battery_button.widget());

    let system_close_button = crate::panel_buttons::SystemButton::new();
    right_box.append(&system_close_button.widget());


    panel_box.set_start_widget(Some(&left_box));
    panel_box.set_center_widget(Some(&center_box));
    panel_box.set_end_widget(Some(&right_box));

    panel_box
  }

  pub fn present(&self) {
    self.window.present();
  }
}