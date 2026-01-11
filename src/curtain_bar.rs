use adw::ApplicationWindow;
use adw::prelude::AdwApplicationWindowExt;
use gtk::Align;
use gtk::Orientation;
use gtk::prelude::*;
use gtk::Box;
use std::boxed::Box as StdBox;

use crate::config::{CurtainBarConfig, Margins};
use crate::traits::ApplicationWindowExtensions;
use crate::traits::CompositeWidget;
use crate::widgets::PanelButton;

pub struct CurtainBar {
  pub window: ApplicationWindow,
}

impl CurtainBar {
  pub fn new(app: &adw::Application) -> Result<Self, StdBox<dyn std::error::Error>> {
    Self::with_config(app, CurtainBarConfig::default())
  }

  fn with_config(
    app: &adw::Application,
    config: CurtainBarConfig,
  ) -> Result<Self, StdBox<dyn std::error::Error>> {
    let window = ApplicationWindow::builder()
      .application(app)
      .title("Curtain Bar")
      .build();

    let _ = window.configure_top_layer_shell(config.height);

    let panel_box =
      Self::create_panel_container(config.button_spacing, &config.margins);

    window.set_content(Some(&panel_box));

    let curtain_bar = Self {
      window
    };

    Ok(curtain_bar)
  }

  fn create_panel_container(
    spacing: i32,
    margins: &Margins,
  ) -> gtk::Box {
    let left_box = Box::builder()
      .orientation(Orientation::Horizontal)
      .spacing(spacing)
      .hexpand(true)
      .halign(Align::Start)
      .build();

    let center_box = Box::builder()
      .orientation(Orientation::Horizontal)
      .spacing(spacing)
      .halign(Align::Center)
      .build();

    let right_box = Box::builder()
      .orientation(Orientation::Horizontal)
      .spacing(spacing)
      .hexpand(true)
      .halign(Align::End)
      .build();

    let panel_box = Box::builder()
      .orientation(Orientation::Horizontal)
      .spacing(8)
      .margin_start(margins.left)
      .margin_end(margins.right)
      .margin_top(margins.top)
      .margin_bottom(margins.bottom)
      .build();

    panel_box.append(&left_box);
    panel_box.append(&center_box);
    panel_box.append(&right_box);

    let launch_button = crate::panel_buttons::LaunchButton::from_icon_name("system-run-symbolic", "pkill rofi || rofi -show drun");
    let launch_widget = launch_button.widget();
    left_box.append(&launch_widget);


    let system_metrics_button = crate::panel_buttons::SystemMetricsButton::new();
    right_box.append(&system_metrics_button.widget());

    let battery_button = crate::panel_buttons::BatteryButton::new();
    right_box.append(&battery_button.widget());

    let system_close_button = crate::panel_buttons::SystemButton::new();
    right_box.append(&system_close_button.widget());




    panel_box
  }

  pub fn present(&self) {
    self.window.present();
  }
}