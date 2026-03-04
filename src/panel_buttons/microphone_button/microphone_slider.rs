use gtk::prelude::*;
use gtk::{Box, Button, Label, Orientation, Scale};

#[derive(Clone)]
pub struct MicrophoneSlider {
    container: Box,
    scale: Scale,
    mute_button: Button,
}

impl MicrophoneSlider {
    pub fn new() -> Self {
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        let label = Label::builder()
            .label("Microphone")
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

        container.append(&label);
        container.append(&scale);
        container.append(&mute_button);

        Self { container, scale, mute_button }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    pub fn set_volume(&self, volume: f64) {
        self.scale.set_value(volume.clamp(0.0, 100.0));
    }

    pub fn set_mute_button_label(&self, label: &str) {
        self.mute_button.set_label(label);
    }

    pub fn connect_value_changed<F>(&self, callback: F)
    where
        F: Fn(f64) + 'static,
    {
        self.scale.connect_value_changed(move |scale| {
            callback(scale.value());
        });
    }

    pub fn connect_mute_clicked<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        self.mute_button.connect_clicked(move |_| callback());
    }
}
