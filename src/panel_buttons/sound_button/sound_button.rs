use gtk::{Widget, glib::object::Cast, prelude::WidgetExt};

use crate::traits::CompositeWidget;
use crate::util::process;
use crate::widgets::PanelButton;
use super::{SoundService, VolumeSlider};

pub struct SoundButton {
  panel_button: PanelButton,
  _volume_slider: VolumeSlider,
}

impl SoundButton {
  pub fn new() -> Self {
    let panel_button = PanelButton::from_icon_name("audio-volume-high-symbolic");
    let volume_slider = VolumeSlider::new(&panel_button);

    // Get and set initial volume from system
    let current_volume = SoundService::get_volume();
    volume_slider.set_volume(current_volume);

    // Set the volume slider as the dropdown widget
    panel_button.set_dropdown_widget(Some(volume_slider.widget().upcast_ref::<Widget>()));

    // Connect to volume changes
    let panel_button_clone = panel_button.clone();
    volume_slider.connect_value_changed(move |volume| {
      Self::update_icon(&panel_button_clone, volume);
      SoundService::set_volume(volume);
      Self::play_feedback_sound();
    });

    // Connect to mute button
    let panel_button_clone2 = panel_button.clone();
    let panel_button_clone3 = panel_button.clone();
    let volume_slider_clone = volume_slider.clone();
    volume_slider.connect_mute_clicked(move || {
      SoundService::toggle_mute();
      let current_volume = SoundService::get_volume();
      let is_muted = SoundService::is_muted();

      Self::update_icon(&panel_button_clone2, current_volume);

      // Update mute button label
      if is_muted {
        volume_slider_clone.set_mute_button_label("Unmute");
      } else {
        volume_slider_clone.set_mute_button_label("Mute");
      }

      // Close the dropdown
      panel_button_clone3.hide_menu();
    });

    // Set initial icon and mute button label based on current volume
    Self::update_icon(&panel_button, current_volume);
    if SoundService::is_muted() {
      volume_slider.set_mute_button_label("Unmute");
    } else {
      volume_slider.set_mute_button_label("Mute");
    }

    Self {
      panel_button,
      _volume_slider: volume_slider,
    }
  }

  fn update_icon(panel_button: &PanelButton, volume: f64) {
    let is_muted = SoundService::is_muted();

    let icon_name = if is_muted {
      "audio-volume-muted-symbolic"
    } else if volume == 0.0 {
      "audio-volume-muted-symbolic"
    } else if volume < 33.0 {
      "audio-volume-low-symbolic"
    } else if volume < 66.0 {
      "audio-volume-medium-symbolic"
    } else {
      "audio-volume-high-symbolic"
    };

    let tooltip = if is_muted {
      format!("Volume: {:.0}% (Muted)", volume)
    } else {
      format!("Volume: {:.0}%", volume)
    };

    panel_button.set_icon_name(icon_name);
    panel_button.set_tooltip_text(Some(&tooltip));
  }

  fn play_feedback_sound() {
    // Play a system sound for volume feedback
    // Using the freedesktop sound theme's audio-volume-change sound
    process::spawn_detached("paplay /usr/share/sounds/freedesktop/stereo/audio-volume-change.oga");
  }
}

impl CompositeWidget for SoundButton {
  fn widget(&self) -> Widget {
    self.panel_button.clone().upcast()
  }
}