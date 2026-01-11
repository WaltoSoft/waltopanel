use gtk::prelude::*;
use gtk::{Box, Button, Label, Orientation, Scale};

use crate::util::process;
use crate::widgets::PanelButton;

#[derive(Clone)]
pub struct VolumeSlider {
  container: Box,
  scale: Scale,
  label: Label,
  mute_button: Button,
}

impl VolumeSlider {
  pub fn new(panel_button: &PanelButton) -> Self {
    let container = Box::builder()
      .orientation(Orientation::Vertical)
      .spacing(8)
      .margin_top(12)
      .margin_bottom(12)
      .margin_start(12)
      .margin_end(12)
      .build();

    let label = Label::builder()
      .label("Volume")
      .halign(gtk::Align::Start)
      .build();

    let scale = Scale::builder()
      .orientation(Orientation::Horizontal)
      .draw_value(true)
      .value_pos(gtk::PositionType::Right)
      .hexpand(true)
      .width_request(200)
      .digits(0)
      .build();

    scale.set_range(0.0, 100.0);
    scale.set_increments(1.0, 1.0);
    scale.set_value(50.0);

    let mute_button = Button::builder()
      .label("Mute")
      .margin_top(4)
      .build();

    let settings_button = Button::builder()
      .label("Audio Settings")
      .margin_top(4)
      .build();

    let panel_button_clone = panel_button.clone();
    settings_button.connect_clicked(move |_| {
      process::spawn_detached("pavucontrol");
      panel_button_clone.hide_menu();
    });

    container.append(&label);
    container.append(&scale);
    container.append(&mute_button);
    container.append(&settings_button);

    Self {
      container,
      scale,
      label,
      mute_button,
    }
  }

  pub fn connect_mute_clicked<F>(&self, callback: F)
  where
    F: Fn() + 'static,
  {
    self.mute_button.connect_clicked(move |_| {
      callback();
    });
  }

  pub fn set_mute_button_label(&self, label: &str) {
    self.mute_button.set_label(label);
  }

  pub fn widget(&self) -> &Box {
    &self.container
  }

  pub fn set_volume(&self, volume: f64) {
    self.scale.set_value(volume.clamp(0.0, 100.0));
  }

  pub fn volume(&self) -> f64 {
    self.scale.value()
  }

  pub fn connect_value_changed<F>(&self, callback: F)
  where
    F: Fn(f64) + 'static,
  {
    self.scale.connect_value_changed(move |scale| {
      callback(scale.value());
    });
  }
}
