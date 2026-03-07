use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use gtk::glib;

#[derive(Debug, Deserialize)]
struct NominatimResult {
    lat: String,
    lon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub temperature: f64,
    pub condition: String,
    pub icon: String,
    pub short_forecast: String,
    pub detailed_forecast: Vec<ForecastPeriod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastPeriod {
    pub name: String,
    pub temperature: i32,
    pub temperature_unit: String,
    pub short_forecast: String,
    pub icon_name: String,
}

// Open-Meteo API response structures
#[derive(Debug, Deserialize)]
struct OpenMeteoResponse {
    current: CurrentWeather,
    daily: DailyForecast,
}

#[derive(Debug, Deserialize)]
struct CurrentWeather {
    temperature_2m: f64,
    weather_code: i32,
    is_day: i32,
}

#[derive(Debug, Deserialize)]
struct DailyForecast {
    time: Vec<String>,
    temperature_2m_max: Vec<f64>,
    temperature_2m_min: Vec<f64>,
    weather_code: Vec<i32>,
}

type Callback = Box<dyn Fn(WeatherData) + 'static>;

thread_local! {
    static SUBSCRIBERS: RefCell<Vec<Callback>> = RefCell::new(Vec::new());
}

lazy_static::lazy_static! {
    static ref CURRENT_WEATHER: Arc<Mutex<Option<WeatherData>>> = Arc::new(Mutex::new(None));
}

pub struct WeatherService;

impl WeatherService {
    pub fn start(update_interval_secs: u64, location: &str) -> Option<WeatherData> {
        let location = location.to_string();

        // Fetch initial weather data
        let initial_weather = Self::fetch_weather_blocking(&location);

        if let Some(ref weather) = initial_weather {
            *CURRENT_WEATHER.lock().unwrap() = Some(weather.clone());
        }

        // Start periodic updates using glib timer
        glib::timeout_add_seconds_local(update_interval_secs as u32, move || {
            eprintln!("Weather timer fired, fetching update...");
            let location = location.clone();
            // Spawn a thread to fetch weather data
            std::thread::spawn(move || {
                match Self::fetch_weather_blocking(&location) {
                    Some(weather) => {
                        eprintln!("Weather update successful: {:.1}°F", weather.temperature);
                        *CURRENT_WEATHER.lock().unwrap() = Some(weather.clone());
                        Self::notify_subscribers(weather);
                    }
                    None => {
                        eprintln!("Weather fetch failed!");
                    }
                }
            });

            glib::ControlFlow::Continue
        });

        initial_weather
    }

    pub fn subscribe<F>(callback: F)
    where
        F: Fn(WeatherData) + 'static,
    {
        SUBSCRIBERS.with(|subscribers| {
            subscribers.borrow_mut().push(Box::new(callback));
        });
    }

    fn notify_subscribers(weather: WeatherData) {
        // Use glib::MainContext to call subscribers on the GTK main thread
        glib::MainContext::default().invoke(move || {
            SUBSCRIBERS.with(|subscribers| {
                for callback in subscribers.borrow().iter() {
                    callback(weather.clone());
                }
            });
        });
    }

    fn geocode(client: &reqwest::blocking::Client, location: &str) -> Option<(f64, f64)> {
        let results: Vec<NominatimResult> = client
            .get("https://nominatim.openstreetmap.org/search")
            .query(&[("q", location), ("format", "json"), ("limit", "1")])
            .send()
            .ok()?
            .json()
            .ok()?;
        let first = results.into_iter().next()?;
        let lat = first.lat.parse::<f64>().ok()?;
        let lon = first.lon.parse::<f64>().ok()?;
        Some((lat, lon))
    }

    fn fetch_weather_blocking(location: &str) -> Option<WeatherData> {
        let client = reqwest::blocking::Client::builder()
            .user_agent("WaltoPanel/1.0")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .ok()?;

        let (lat, lon) = Self::geocode(&client, location).or_else(|| {
            eprintln!("waltopanel: failed to geocode location '{}'", location);
            None
        })?;

        // Open-Meteo API - single request for current weather and daily forecast
        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weather_code,is_day&daily=temperature_2m_max,temperature_2m_min,weather_code&temperature_unit=fahrenheit&timezone=auto",
            lat, lon
        );

        let response = client
            .get(&url)
            .send()
            .ok()?
            .json::<OpenMeteoResponse>()
            .ok()?;

        let current_temp = response.current.temperature_2m;
        let current_code = response.current.weather_code;
        let is_day = response.current.is_day == 1;
        let condition = Self::weather_code_to_description(current_code);
        let icon = Self::weather_code_to_icon(current_code, is_day);

        eprintln!("Weather API (Open-Meteo) - Temp: {:.1}°F, Condition: '{}'", current_temp, condition);

        // Build daily forecast
        let day_names = ["Today", "Tomorrow"];
        let detailed_forecast: Vec<ForecastPeriod> = response.daily.time
            .iter()
            .enumerate()
            .take(7)
            .map(|(i, date)| {
                let name = if i < day_names.len() {
                    day_names[i].to_string()
                } else {
                    // Parse date and get day name
                    Self::date_to_day_name(date)
                };
                let high = response.daily.temperature_2m_max.get(i).copied().unwrap_or(0.0) as i32;
                let low = response.daily.temperature_2m_min.get(i).copied().unwrap_or(0.0) as i32;
                let code = response.daily.weather_code.get(i).copied().unwrap_or(0);
                let forecast_desc = Self::weather_code_to_description(code);

                ForecastPeriod {
                    name,
                    temperature: high,
                    temperature_unit: "F".to_string(),
                    short_forecast: format!("{} (Low: {}°F)", forecast_desc, low),
                    icon_name: Self::weather_code_to_icon(code, true),
                }
            })
            .collect();

        Some(WeatherData {
            temperature: current_temp,
            condition: condition.clone(),
            icon,
            short_forecast: condition,
            detailed_forecast,
        })
    }

    fn weather_code_to_description(code: i32) -> String {
        // WMO Weather interpretation codes
        match code {
            0 => "Clear sky",
            1 => "Mainly clear",
            2 => "Partly cloudy",
            3 => "Overcast",
            45 | 48 => "Foggy",
            51 => "Light drizzle",
            53 => "Moderate drizzle",
            55 => "Dense drizzle",
            56 | 57 => "Freezing drizzle",
            61 => "Slight rain",
            63 => "Moderate rain",
            65 => "Heavy rain",
            66 | 67 => "Freezing rain",
            71 => "Slight snow",
            73 => "Moderate snow",
            75 => "Heavy snow",
            77 => "Snow grains",
            80 => "Slight rain showers",
            81 => "Moderate rain showers",
            82 => "Violent rain showers",
            85 => "Slight snow showers",
            86 => "Heavy snow showers",
            95 => "Thunderstorm",
            96 | 99 => "Thunderstorm with hail",
            _ => "Unknown",
        }.to_string()
    }

    fn weather_code_to_icon(code: i32, is_day: bool) -> String {
        match code {
            0 => if is_day { "clear" } else { "clear-night" },
            1 | 2 => if is_day { "partly-cloudy" } else { "partly-cloudy-night" },
            3 => "cloudy",
            45 | 48 => "fog",
            51 | 53 | 55 | 56 | 57 => "rain",
            61 | 63 | 65 | 66 | 67 | 80 | 81 | 82 => "rain",
            71 | 73 | 75 | 77 | 85 | 86 => "snow",
            95 | 96 | 99 => "storm",
            _ => if is_day { "partly-cloudy" } else { "partly-cloudy-night" },
        }.to_string()
    }

    fn date_to_day_name(date: &str) -> String {
        // Parse YYYY-MM-DD format and return day name
        use chrono::{NaiveDate, Datelike, Weekday};

        if let Ok(parsed) = NaiveDate::parse_from_str(date, "%Y-%m-%d") {
            match parsed.weekday() {
                Weekday::Mon => "Monday",
                Weekday::Tue => "Tuesday",
                Weekday::Wed => "Wednesday",
                Weekday::Thu => "Thursday",
                Weekday::Fri => "Friday",
                Weekday::Sat => "Saturday",
                Weekday::Sun => "Sunday",
            }.to_string()
        } else {
            date.to_string()
        }
    }
}
