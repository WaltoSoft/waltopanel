use adw::ApplicationWindow;
use adw::prelude::AdwApplicationWindowExt;
use gtk::Align;
use gtk::CenterBox;
use gtk::Orientation;
use gtk::prelude::*;
use gtk::Box;
use std::boxed::Box as StdBox;

use crate::config::{PanelButtonConfig, PanelLayoutConfig, WaltoPanelConfig, Margins};
use crate::traits::ApplicationWindowExtensions;
use crate::traits::CompositeWidget;

pub struct SystemPanel {
  pub window: ApplicationWindow,
}

impl SystemPanel {
  pub fn new_with_monitor(app: &adw::Application, monitor_name: String) -> Result<Self, StdBox<dyn std::error::Error>> {
    let mut config = WaltoPanelConfig::default();
    config.layout = PanelLayoutConfig::load_from_file();
    Self::with_config(app, config, Some(monitor_name))
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
      Self::create_panel_container(config.button_spacing, &config.margins, &config.layout, monitor_name);

    window.set_content(Some(&panel_box));

    let system_panel = Self {
      window
    };

    Ok(system_panel)
  }

  fn create_panel_container(
    spacing: i32,
    margins: &Margins,
    layout: &PanelLayoutConfig,
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

    Self::append_buttons(&left_box, &layout.left, monitor_name.as_deref());
    Self::append_buttons(&center_box, &layout.center, monitor_name.as_deref());
    Self::append_buttons(&right_box, &layout.right, monitor_name.as_deref());

    panel_box.set_start_widget(Some(&left_box));
    panel_box.set_center_widget(Some(&center_box));
    panel_box.set_end_widget(Some(&right_box));

    panel_box
  }

  fn append_buttons(container: &Box, buttons: &[PanelButtonConfig], monitor_name: Option<&str>) {
    for button in buttons {
      match button {
        PanelButtonConfig::Launch { icon, command } => {
          let btn = crate::panel_buttons::LaunchButton::from_icon_name(icon, command.clone());
          container.append(btn.widget());
        }
        PanelButtonConfig::Clock => {
          let btn = crate::panel_buttons::ClockButton::new();
          container.append(btn.widget());
        }
        PanelButtonConfig::Weather { location } => {
          let btn = crate::panel_buttons::WeatherButton::new(&location);
          container.append(btn.widget());
        }
        PanelButtonConfig::Workspace => {
          if let Some(name) = monitor_name {
            let btn = crate::panel_buttons::WorkspaceButton::new_with_monitor(name.to_string());
            container.append(btn.widget());
          }
        }
        PanelButtonConfig::Network => {
          let btn = crate::panel_buttons::NetworkButton::new();
          container.append(btn.widget());
        }
        PanelButtonConfig::Brightness => {
          let btn = crate::panel_buttons::BrightnessButton::new();
          container.append(btn.widget());
        }
        PanelButtonConfig::Microphone => {
          let btn = crate::panel_buttons::MicrophoneButton::new();
          container.append(btn.widget());
        }
        PanelButtonConfig::Sound => {
          let btn = crate::panel_buttons::SoundButton::new();
          container.append(btn.widget());
        }
        PanelButtonConfig::Battery => {
          let btn = crate::panel_buttons::BatteryButton::new();
          container.append(btn.widget());
        }
        PanelButtonConfig::System => {
          let btn = crate::panel_buttons::SystemButton::new();
          container.append(btn.widget());
        }
        PanelButtonConfig::SystemMetrics => {
          let btn = crate::panel_buttons::SystemMetricsButton::new();
          container.append(btn.widget());
        }
      }
    }
  }

  pub fn present(&self) {
    self.window.present();
  }
}
