use gtk::{Widget, glib::object::Cast, prelude::WidgetExt};

use crate::traits::CompositeWidget;
use crate::widgets::PanelButton;
use super::{BrightnessService, BrightnessSlider};

pub struct BrightnessButton {
    panel_button: PanelButton,
    _brightness_slider: BrightnessSlider,
}

impl BrightnessButton {
    pub fn new() -> Self {
        BrightnessService::start();

        let panel_button = PanelButton::from_icon_name("display-brightness-symbolic");
        let brightness_slider = BrightnessSlider::new();

        let current = BrightnessService::get_brightness();
        brightness_slider.set_brightness(current);
        Self::update_icon(&panel_button, current);

        panel_button.set_dropdown_widget(Some(brightness_slider.widget().upcast_ref::<Widget>()));

        let pb = panel_button.clone();
        brightness_slider.connect_value_changed(move |brightness| {
            Self::update_icon(&pb, brightness);
            BrightnessService::set_brightness(brightness);
        });

        let pb = panel_button.clone();
        let slider = brightness_slider.clone();
        BrightnessService::subscribe(move |state| {
            Self::update_icon(&pb, state.brightness);
            slider.set_brightness(state.brightness);
        });

        Self {
            panel_button,
            _brightness_slider: brightness_slider,
        }
    }

    fn update_icon(panel_button: &PanelButton, brightness: f64) {
        let icon = if brightness < 34.0 {
            "display-brightness-low-symbolic"
        } else if brightness < 67.0 {
            "display-brightness-medium-symbolic"
        } else {
            "display-brightness-high-symbolic"
        };

        // Fall back to the generic icon if the themed variant isn't available
        let display = gtk::gdk::Display::default();
        let icon_name = if let Some(display) = display {
            let theme = gtk::IconTheme::for_display(&display);
            if theme.has_icon(icon) { icon } else { "display-brightness-symbolic" }
        } else {
            "display-brightness-symbolic"
        };

        panel_button.set_icon_name(icon_name);
        panel_button.set_tooltip_text(Some(&format!("Brightness: {:.0}%", brightness)));
    }
}

impl CompositeWidget for BrightnessButton {
    fn widget(&self) -> &Widget {
        self.panel_button.upcast_ref()
    }
}
