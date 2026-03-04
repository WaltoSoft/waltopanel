use gtk::prelude::*;
use gtk::{Box, Label, Orientation, Scale};

#[derive(Clone)]
pub struct BrightnessSlider {
    container: Box,
    scale: Scale,
}

impl BrightnessSlider {
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
            .label("Brightness")
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

        scale.set_range(1.0, 100.0);
        scale.set_increments(1.0, 5.0);
        scale.set_value(50.0);

        container.append(&label);
        container.append(&scale);

        Self { container, scale }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    pub fn set_brightness(&self, brightness: f64) {
        self.scale.set_value(brightness.clamp(1.0, 100.0));
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
