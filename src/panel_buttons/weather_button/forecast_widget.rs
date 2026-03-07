use gtk::{
    prelude::*,
    Align, Box as GtkBox, Button, Entry, Image, Label, Orientation,
};
use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;
use super::weather_service::WeatherData;

#[derive(Clone)]
pub struct ForecastWidget {
    container: GtkBox,
    location_label: Label,
    weather_box: GtkBox,
    on_location_changed: Rc<RefCell<Option<Box<dyn Fn(String)>>>>,
}

impl ForecastWidget {
    pub fn new() -> Self {
        let container = GtkBox::new(Orientation::Vertical, 8);
        container.set_margin_top(12);
        container.set_margin_bottom(12);
        container.set_margin_start(12);
        container.set_margin_end(12);

        // Header row: location label + edit button
        let header_row = GtkBox::new(Orientation::Horizontal, 8);
        let location_label = Label::new(None);
        location_label.add_css_class("heading");
        location_label.set_halign(Align::Start);
        location_label.set_hexpand(true);
        let edit_button = Button::from_icon_name("document-edit-symbolic");
        edit_button.set_has_frame(false);
        header_row.append(&location_label);
        header_row.append(&edit_button);
        container.append(&header_row);

        let weather_box = GtkBox::new(Orientation::Vertical, 8);
        container.append(&weather_box);

        let on_location_changed: Rc<RefCell<Option<Box<dyn Fn(String)>>>> =
            Rc::new(RefCell::new(None));

        {
            let location_label = location_label.clone();
            let on_location_changed = on_location_changed.clone();
            edit_button.connect_clicked(move |btn| {
                let current = location_label.text().to_string();
                let cb = on_location_changed.clone();

                let dialog = gtk::Window::builder()
                    .title("Edit Location")
                    .modal(true)
                    .resizable(false)
                    .default_width(360)
                    .build();

                // Close the dropdown popover before presenting the modal
                let mut ancestor = btn.parent();
                while let Some(p) = ancestor {
                    if let Ok(popover) = p.clone().downcast::<gtk::Popover>() {
                        popover.popdown();
                        break;
                    }
                    ancestor = p.parent();
                }

                if let Some(root) = btn.root() {
                    if let Ok(parent_win) = root.downcast::<gtk::Window>() {
                        dialog.set_transient_for(Some(&parent_win));
                    }
                }

                let vbox = GtkBox::new(Orientation::Vertical, 12);
                vbox.set_margin_top(20);
                vbox.set_margin_bottom(20);
                vbox.set_margin_start(20);
                vbox.set_margin_end(20);

                let entry = Entry::new();
                entry.set_text(&current);
                entry.set_placeholder_text(Some("City, state or zip code..."));
                vbox.append(&entry);

                let btn_row = GtkBox::new(Orientation::Horizontal, 8);
                btn_row.set_halign(Align::End);
                let cancel_btn = Button::with_label("Cancel");
                let save_btn = Button::with_label("Save");
                save_btn.add_css_class("suggested-action");
                btn_row.append(&cancel_btn);
                btn_row.append(&save_btn);
                vbox.append(&btn_row);

                dialog.set_child(Some(&vbox));

                // Save: called by Enter key in entry or Save button click
                entry.connect_activate({
                    let dialog = dialog.clone();
                    let entry = entry.clone();
                    let cb = cb.clone();
                    move |_| {
                        let text = entry.text().to_string();
                        if !text.is_empty() {
                            if let Some(ref f) = *cb.borrow() { f(text); }
                        }
                        dialog.close();
                    }
                });

                save_btn.connect_clicked({
                    let dialog = dialog.clone();
                    let entry = entry.clone();
                    let cb = cb.clone();
                    move |_| {
                        let text = entry.text().to_string();
                        if !text.is_empty() {
                            if let Some(ref f) = *cb.borrow() { f(text); }
                        }
                        dialog.close();
                    }
                });

                cancel_btn.connect_clicked({
                    let dialog = dialog.clone();
                    move |_| dialog.close()
                });

                dialog.present();
            });
        }

        Self { container, location_label, weather_box, on_location_changed }
    }

    pub fn connect_location_changed<F: Fn(String) + 'static>(&self, callback: F) {
        *self.on_location_changed.borrow_mut() = Some(Box::new(callback));
    }

    pub fn update(&self, weather: &WeatherData) {
        self.location_label.set_text(&weather.location_name);

        // Clear and rebuild the dynamic weather section
        while let Some(child) = self.weather_box.first_child() {
            self.weather_box.remove(&child);
        }

        // Add current conditions
        let current_box = self.create_forecast_row(
            "",
            weather.temperature as i32,
            "F",
            &weather.short_forecast,
            &weather.icon,
            true,
        );
        self.weather_box.append(&current_box);

        // Add separator
        let separator = gtk::Separator::new(Orientation::Horizontal);
        separator.set_margin_top(8);
        separator.set_margin_bottom(8);
        self.weather_box.append(&separator);

        // Add forecast header
        let forecast_header = Label::new(Some("Forecast"));
        forecast_header.add_css_class("heading");
        forecast_header.set_halign(Align::Start);
        self.weather_box.append(&forecast_header);

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
            self.weather_box.append(&period_box);
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

        if path.exists() {
            return path;
        }

        // Fallback to installed path
        let installed = PathBuf::from(format!("/usr/share/waltopanel/weather-icons/{}.svg", icon_name));
        if installed.exists() {
            return installed;
        }

        path
    }
}

impl ForecastWidget {
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }
}
