use gtk::{glib, prelude::*, Image, Widget};
use std::path::PathBuf;

use crate::{traits::CompositeWidget, widgets::PanelButtonBuilder};
use crate::widgets::PanelButton;
use super::weather_service::WeatherService;
use super::forecast_widget::ForecastWidget;

pub struct WeatherButton {
  panel_button: PanelButton,
}

impl WeatherButton {
  pub fn new() -> Self {
    let initial_data = WeatherService::start(300);

    // Create dropdown widget before the builder so it stays alive
    let forecast_widget = ForecastWidget::new();
    let dropdown_widget: Widget = forecast_widget.widget().clone().upcast();

    // Update forecast widget with initial data if available
    if let Some(ref weather_data) = initial_data {
      forecast_widget.update(weather_data);
    }

    let panel_button =
      if let Some(ref weather_data) = initial_data {
        let weather_icon = Self::get_weather_icon(&weather_data.icon);
        let temperature_text = format!("{}°F", weather_data.temperature as i32);

        PanelButtonBuilder::new()
          .custom_widget(Some(weather_icon.upcast()))
          .text(&temperature_text)
          .dropdown_widget(dropdown_widget)
          .build()
      }
      else {
        PanelButtonBuilder::new()
          .text("--°F")
          .dropdown_widget(dropdown_widget)
          .build()
      };

    // Subscribe to weather updates
    let panel_button_clone = panel_button.clone();
    let forecast_widget_clone = forecast_widget.clone();
    WeatherService::subscribe(move |weather| {
      let panel_button = panel_button_clone.clone();
      let forecast_widget = forecast_widget_clone.clone();
      let weather_clone = weather.clone();

      // Defer widget updates to the next GTK main loop iteration
      glib::idle_add_local_once(move || {
        // Update panel button text
        let temp_text = format!("{}°F", weather_clone.temperature as i32);
        panel_button.set_text(&temp_text);

        // Update tooltip
        let tooltip = format!(
          "{}\n{}°F",
          weather_clone.short_forecast,
          weather_clone.temperature as i32
        );
        panel_button.set_tooltip_text(Some(&tooltip));

        // Update forecast dropdown
        forecast_widget.update(&weather_clone);
      });
    });

    Self {
      panel_button,
    }
  }

  fn get_weather_icon(icon_name: &str) -> Image {
    let image = Image::builder()
        .pixel_size(16)
        .build();

    let icon_path = Self::get_icon_path(icon_name);
     if icon_path.exists() {
        image.set_from_file(Some(&icon_path));
     } else {
        eprintln!("Weather icon not found: {:?}", icon_path);
     }

     image
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

impl CompositeWidget for WeatherButton {
  fn widget(&self) -> &Widget {
    self.panel_button.upcast_ref()
  }
}


/*
pub struct WeatherButton {
    panel_button: PanelButton,
    _dropdown_box: gtk::Box,
}

impl WeatherButton {
    pub fn new() -> Self {
        // COMMENTED OUT FOR DEBUGGING - Weather service
        // // Start weather service with 5-minute updates (300 seconds)
        // let initial_weather = WeatherService::start(300);
        let initial_weather: Option<WeatherData> = None;

        // Create panel button with temperature text
        let panel_button = PanelButton::new();

        // COMMENTED OUT FOR DEBUGGING - Custom weather icon
        // // Create custom weather icon image
        // let weather_icon = Image::builder()
        //     .pixel_size(24)
        //     .build();

        // // Set initial icon
        // if let Some(ref weather) = initial_weather {
        //     Self::update_icon(&weather_icon, &weather.icon);
        // } else {
        //     Self::update_icon(&weather_icon, "partly-cloudy");
        // }

        // // Set the icon as a custom widget (after setting the initial icon)
        // panel_button.set_custom_widget(Some(weather_icon.upcast_ref::<Widget>()));

        // Set initial temperature text
        if let Some(ref weather) = initial_weather {
            let temp_text = format!("{}°F", weather.temperature as i32);
            panel_button.set_text(&temp_text);
        } else {
            panel_button.set_text("-- °F");
        }

        // Create dropdown box directly - no wrapper
        let dropdown_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
        dropdown_box.set_margin_top(12);
        dropdown_box.set_margin_bottom(12);
        dropdown_box.set_margin_start(12);
        dropdown_box.set_margin_end(12);

        // Set dropdown widget
        panel_button.set_dropdown_widget(Some(dropdown_box.upcast_ref()));

        // Set initial tooltip
        if let Some(ref weather) = initial_weather {
            Self::update_panel_button(&panel_button, weather);
        } else {
            panel_button.set_tooltip_text(Some("Weather data unavailable"));
        }

        let obj = Self {
            panel_button,
            _dropdown_box: dropdown_box,
        };

        // COMMENTED OUT FOR DEBUGGING - Weather updates subscription
        // // Subscribe to weather updates
        // let panel_button_clone = obj.panel_button.clone();

        // WeatherService::subscribe(move |weather| {
        //     let panel_button = panel_button_clone.clone();
        //     let weather_clone = weather.clone();

        //     // Defer widget updates to the next GTK main loop iteration
        //     glib::idle_add_local_once(move || {
        //         // Only update text and tooltip
        //         Self::update_panel_button(&panel_button, &weather_clone);
        //     });
        // });

        obj
    }

    fn update_icon(image: &Image, icon_name: &str) {
        let icon_path = Self::get_icon_path(icon_name);
        if icon_path.exists() {
            image.set_from_file(Some(&icon_path));
        } else {
            eprintln!("Weather icon not found: {:?}", icon_path);
        }
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

    fn update_panel_button(panel_button: &PanelButton, weather: &WeatherData) {
        // Update temperature text
        let temp_text = format!("{}°F", weather.temperature as i32);
        panel_button.set_text(&temp_text);

        // Update tooltip
        let tooltip = format!(
            "{}\n{}°F",
            weather.short_forecast,
            weather.temperature as i32
        );
        panel_button.set_tooltip_text(Some(&tooltip));
    }
}

impl CompositeWidget for WeatherButton {
    fn widget(&self) -> Widget {
        self.panel_button.clone().upcast()
    }
}
 */
