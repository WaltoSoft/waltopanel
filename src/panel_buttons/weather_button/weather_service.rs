use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use gtk::glib;

const LATITUDE: f64 = 35.9978;  // Hardcoded for zip 74037 (Jenks, OK)
const LONGITUDE: f64 = -95.9683;

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

#[derive(Debug, Deserialize)]
struct PointsResponse {
    properties: PointsProperties,
}

#[derive(Debug, Deserialize)]
struct PointsProperties {
    forecast: String,
    #[serde(rename = "forecastHourly")]
    _forecast_hourly: String,
    #[serde(rename = "observationStations")]
    observation_stations: String,
}

#[derive(Debug, Deserialize)]
struct ForecastResponse {
    properties: ForecastProperties,
}

#[derive(Debug, Deserialize)]
struct ForecastProperties {
    periods: Vec<Period>,
}

#[derive(Debug, Deserialize)]
struct Period {
    name: String,
    temperature: i32,
    #[serde(rename = "temperatureUnit")]
    temperature_unit: String,
    #[serde(rename = "shortForecast")]
    short_forecast: String,
    icon: String,
}

#[derive(Debug, Deserialize)]
struct StationsResponse {
    features: Vec<StationFeature>,
}

#[derive(Debug, Deserialize)]
struct StationFeature {
    properties: StationProperties,
}

#[derive(Debug, Deserialize)]
struct StationProperties {
    #[serde(rename = "stationIdentifier")]
    station_identifier: String,
}

#[derive(Debug, Deserialize)]
struct ObservationResponse {
    properties: ObservationProperties,
}

#[derive(Debug, Deserialize)]
struct ObservationProperties {
    temperature: TemperatureValue,
    #[serde(rename = "textDescription")]
    text_description: String,
}

#[derive(Debug, Deserialize)]
struct TemperatureValue {
    value: Option<f64>,
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
    pub fn start(update_interval_secs: u64) -> Option<WeatherData> {
        // Fetch initial weather data
        let initial_weather = Self::fetch_weather_blocking();

        if let Some(ref weather) = initial_weather {
            *CURRENT_WEATHER.lock().unwrap() = Some(weather.clone());
        }

        // Start periodic updates using glib timer
        glib::timeout_add_seconds_local(update_interval_secs as u32, move || {
            // Spawn a thread to fetch weather data
            std::thread::spawn(|| {
                if let Some(weather) = Self::fetch_weather_blocking() {
                    *CURRENT_WEATHER.lock().unwrap() = Some(weather.clone());
                    Self::notify_subscribers(weather);
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

    fn fetch_weather_blocking() -> Option<WeatherData> {
        // Use a blocking reqwest client since we're already in a thread
        let client = reqwest::blocking::Client::builder()
            .user_agent("WaltoPanel/1.0")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .ok()?;

        // Step 1: Get forecast URL from points endpoint
        let points_url = format!(
            "https://api.weather.gov/points/{},{}",
            LATITUDE, LONGITUDE
        );

        let points_response = client
            .get(&points_url)
            .send()
            .ok()?
            .json::<PointsResponse>()
            .ok()?;

        // Step 2: Get observation stations
        let stations_response = client
            .get(&points_response.properties.observation_stations)
            .send()
            .ok()?
            .json::<StationsResponse>()
            .ok()?;

        // Get the first (closest) station
        let station = stations_response.features.first()?;
        let station_id = &station.properties.station_identifier;

        // Step 3: Get current observations from the station
        let observations_url = format!(
            "https://api.weather.gov/stations/{}/observations/latest",
            station_id
        );

        let observation = client
            .get(&observations_url)
            .send()
            .ok()?
            .json::<ObservationResponse>()
            .ok()?;

        // Get current temperature (convert from Celsius to Fahrenheit if needed)
        let current_temp_c = observation.properties.temperature.value?;
        let current_temp_f = current_temp_c * 9.0 / 5.0 + 32.0;
        let current_condition = observation.properties.text_description;

        // Debug: Log what we got from the API
        eprintln!("Weather API - Temp: {:.1}°F, Condition: '{}'", current_temp_f, current_condition);

        // Step 4: Get forecast data for future periods
        let forecast_response = client
            .get(&points_response.properties.forecast)
            .send()
            .ok()?
            .json::<ForecastResponse>()
            .ok()?;

        let periods = forecast_response.properties.periods;
        if periods.is_empty() {
            return None;
        }

        // Convert forecast periods
        let detailed_forecast: Vec<ForecastPeriod> = periods
            .iter()
            .take(7) // Take next 7 periods for detailed forecast
            .map(|p| ForecastPeriod {
                name: p.name.clone(),
                temperature: p.temperature,
                temperature_unit: p.temperature_unit.clone(),
                short_forecast: p.short_forecast.clone(),
                icon_name: Self::get_icon_from_forecast(&p.short_forecast),
            })
            .collect();

        Some(WeatherData {
            temperature: current_temp_f,
            condition: current_condition.clone(),
            icon: Self::get_icon_from_forecast(&current_condition),
            short_forecast: current_condition,
            detailed_forecast,
        })
    }

    fn get_icon_from_forecast(forecast: &str) -> String {
        let forecast_lower = forecast.to_lowercase();

        if forecast_lower.contains("thunder") || forecast_lower.contains("storm") {
            "storm"
        } else if forecast_lower.contains("rain") || forecast_lower.contains("shower") || forecast_lower.contains("drizzle") {
            "rain"
        } else if forecast_lower.contains("snow") || forecast_lower.contains("flurries") || forecast_lower.contains("sleet") {
            "snow"
        } else if forecast_lower.contains("cloud") || forecast_lower.contains("overcast") {
            if forecast_lower.contains("partly") || forecast_lower.contains("few") || forecast_lower.contains("scattered") {
                "partly-cloudy"
            } else {
                "cloudy"
            }
        } else if forecast_lower.contains("clear") || forecast_lower.contains("sunny") || forecast_lower.contains("fair") {
            "clear"
        } else if forecast_lower.contains("fog") || forecast_lower.contains("haze") || forecast_lower.contains("mist") {
            "fog"
        } else if forecast_lower.contains("wind") || forecast_lower.contains("breezy") || forecast_lower.contains("blustery") {
            "windy"
        } else {
            "partly-cloudy" // Default fallback
        }
        .to_string()
    }

    pub fn get_current_weather() -> Option<WeatherData> {
        CURRENT_WEATHER.lock().unwrap().clone()
    }
}
