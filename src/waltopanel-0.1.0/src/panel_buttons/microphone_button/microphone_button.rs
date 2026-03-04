use gtk::{Widget, glib::object::Cast, prelude::WidgetExt};

use crate::traits::CompositeWidget;
use crate::widgets::PanelButton;
use super::{MicrophoneService, MicrophoneSlider};

pub struct MicrophoneButton {
    panel_button: PanelButton,
    _microphone_slider: MicrophoneSlider,
}

impl MicrophoneButton {
    pub fn new() -> Self {
        MicrophoneService::start();

        let panel_button = PanelButton::from_icon_name("audio-input-microphone-symbolic");
        let slider = MicrophoneSlider::new();

        let current_volume = MicrophoneService::get_volume();
        let is_muted = MicrophoneService::is_muted();
        slider.set_volume(current_volume);
        slider.set_mute_button_label(if is_muted { "Unmute" } else { "Mute" });
        Self::update_icon(&panel_button, current_volume, is_muted);

        panel_button.set_dropdown_widget(Some(slider.widget().upcast_ref::<Widget>()));

        // Slider value changed
        let pb = panel_button.clone();
        slider.connect_value_changed(move |volume| {
            let muted = MicrophoneService::is_muted();
            Self::update_icon(&pb, volume, muted);
            MicrophoneService::set_volume(volume);
        });

        // Mute button
        let pb2 = panel_button.clone();
        let pb3 = panel_button.clone();
        let slider_clone = slider.clone();
        slider.connect_mute_clicked(move || {
            MicrophoneService::toggle_mute();
            let volume = MicrophoneService::get_volume();
            let is_muted = MicrophoneService::is_muted();
            Self::update_icon(&pb2, volume, is_muted);
            slider_clone.set_mute_button_label(if is_muted { "Unmute" } else { "Mute" });
            pb3.hide_menu();
        });

        // External changes (e.g. hardware mute button)
        let pb = panel_button.clone();
        let slider_clone = slider.clone();
        MicrophoneService::subscribe(move |state| {
            Self::update_icon(&pb, state.volume, state.is_muted);
            slider_clone.set_volume(state.volume);
            slider_clone.set_mute_button_label(if state.is_muted { "Unmute" } else { "Mute" });
        });

        Self {
            panel_button,
            _microphone_slider: slider,
        }
    }

    fn update_icon(panel_button: &PanelButton, volume: f64, is_muted: bool) {
        let icon = if is_muted || volume == 0.0 {
            "microphone-sensitivity-muted-symbolic"
        } else if volume < 50.0 {
            "microphone-sensitivity-low-symbolic"
        } else {
            "microphone-sensitivity-high-symbolic"
        };

        // Fall back to the generic mic icon if themed variants aren't available
        let display = gtk::gdk::Display::default();
        let icon_name = if let Some(display) = display {
            let theme = gtk::IconTheme::for_display(&display);
            if theme.has_icon(icon) { icon } else { "audio-input-microphone-symbolic" }
        } else {
            "audio-input-microphone-symbolic"
        };

        let tooltip = if is_muted {
            format!("Microphone: {:.0}% (Muted)", volume)
        } else {
            format!("Microphone: {:.0}%", volume)
        };

        panel_button.set_icon_name(icon_name);
        panel_button.set_tooltip_text(Some(&tooltip));
    }
}

impl CompositeWidget for MicrophoneButton {
    fn widget(&self) -> &Widget {
        self.panel_button.upcast_ref()
    }
}
