use gtk::{
    prelude::*,
    Align, Box as GtkBox, Image, Label, Orientation,
};
use std::path::PathBuf;
use super::weather_service::WeatherData;

#[derive(Clone)]
pub struct ForecastWidget {
    container: GtkBox,
}

impl ForecastWidget {
    pub fn new() -> Self {
        let container = GtkBox::new(Orientation::Vertical, 8);
        container.set_margin_top(12);
        container.set_margin_bottom(12);
        container.set_margin_start(12);
        container.set_margin_end(12);

        // Start with just a simple label
        let test_label = Label::new(Some("Weather Forecast"));
        container.append(&test_label);

        Self { container }
    }

    pub fn update(&self, weather: &WeatherData) {
        // Clear existing children
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }

        // Add current conditions header
        let current_header = Label::new(Some("Current Conditions"));
        current_header.add_css_class("heading");
        current_header.set_halign(Align::Start);
        self.container.append(&current_header);

        // Add current conditions
        let current_box = self.create_forecast_row(
            "",
            weather.temperature as i32,
            "F",
            &weather.short_forecast,
            &weather.icon,
            true,
        );
        self.container.append(&current_box);

        // Add separator
        let separator = gtk::Separator::new(Orientation::Horizontal);
        separator.set_margin_top(8);
        separator.set_margin_bottom(8);
        self.container.append(&separator);

        // Add forecast header
        let forecast_header = Label::new(Some("Forecast"));
        forecast_header.add_css_class("heading");
        forecast_header.set_halign(Align::Start);
        self.container.append(&forecast_header);

        // Add detailed forecast periods
        for period in &weather.detailed_forecast {
            let period_box = self.create_forecast_row(
                &period.name,
                period.temperature,
                &period.temperature_unit,
                &period.short_forecast,
                &period.icon_name,
                false,
            );
            self.container.append(&period_box);
        }
    }

    fn create_forecast_row(
        &self,
        name: &str,
        temperature: i32,
        temp_unit: &str,
        forecast: &str,
        icon_name: &str,
        is_current: bool,
    ) -> GtkBox {
        let row = GtkBox::new(Orientation::Horizontal, 12);
        row.set_margin_top(4);
        row.set_margin_bottom(4);

        // Icon
        let icon = Image::builder()
            .pixel_size(32)
            .halign(Align::Start)
            .build();

        let icon_path = Self::get_icon_path(icon_name);
        if icon_path.exists() {
            icon.set_from_file(Some(&icon_path));
        } else {
            // Fallback to symbolic icon from theme if custom file doesn't exist
            eprintln!("Weather icon file not found: {:?}, using fallback", icon_path);
            icon.set_from_file(Some(&Self::get_icon_path("partly-cloudy")));
        }

        row.append(&icon);

        // Content box (period name, temp, forecast)
        let content_box = GtkBox::new(Orientation::Vertical, 4);
        content_box.set_hexpand(true);

        if !name.is_empty() {
            let name_label = Label::new(Some(name));
            name_label.add_css_class("caption");
            name_label.set_halign(Align::Start);
            content_box.append(&name_label);
        }

        // Temperature and forecast on same line if current, otherwise separate
        if is_current {
            let temp_forecast_box = GtkBox::new(Orientation::Horizontal, 8);

            let temp_label = Label::new(Some(&format!("{}°{}", temperature, temp_unit)));
            temp_label.add_css_class("title-2");
            temp_label.set_halign(Align::Start);
            temp_forecast_box.append(&temp_label);

            let forecast_label = Label::new(Some(forecast));
            forecast_label.set_halign(Align::Start);
            temp_forecast_box.append(&forecast_label);

            content_box.append(&temp_forecast_box);
        } else {
            let temp_label = Label::new(Some(&format!("{}°{}", temperature, temp_unit)));
            temp_label.add_css_class("title-4");
            temp_label.set_halign(Align::Start);
            content_box.append(&temp_label);

            let forecast_label = Label::new(Some(forecast));
            forecast_label.add_css_class("caption");
            forecast_label.set_halign(Align::Start);
            forecast_label.set_wrap(true);
            forecast_label.set_max_width_chars(30);
            content_box.append(&forecast_label);
        }

        row.append(&content_box);

        row
    }

    fn get_icon_path(icon_name: &str) -> PathBuf {
        // Try to get the path relative to the executable
        let mut path = std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("."));

        // Go up to the project root (assuming executable is in target/debug or target/release)
        path.pop(); // Remove executable name
        if path.ends_with("debug") || path.ends_with("release") {
            path.pop(); // Remove debug/release
            path.pop(); // Remove target
        }

        path.push("resources");
        path.push("weather-icons");
        path.push(format!("{}.svg", icon_name));

        // Fallback to absolute path
        if !path.exists() {
            PathBuf::from(format!("/home/billy/Git/waltopanel/resources/weather-icons/{}.svg", icon_name))
        } else {
            path
        }
    }
}

impl ForecastWidget {
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }
}
