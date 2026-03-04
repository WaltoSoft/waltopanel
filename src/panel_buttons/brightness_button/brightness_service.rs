use gtk::glib;
use std::{cell::RefCell, process::Command};

#[derive(Debug, Clone)]
pub struct BrightnessState {
    pub brightness: f64, // 0-100 percentage
}

type BrightnessCallback = Box<dyn Fn(BrightnessState)>;

struct BrightnessServiceState {
    subscribers: Vec<BrightnessCallback>,
}

thread_local! {
    static BRIGHTNESS_SERVICE: RefCell<Option<BrightnessServiceState>> = RefCell::new(None);
}

pub struct BrightnessService;

impl BrightnessService {
    pub fn start() {
        BRIGHTNESS_SERVICE.with(|service| {
            if service.borrow().is_some() {
                return;
            }
            *service.borrow_mut() = Some(BrightnessServiceState {
                subscribers: Vec::new(),
            });
            std::thread::spawn(|| Self::monitor_changes());
        });
    }

    pub fn subscribe<F>(callback: F)
    where
        F: Fn(BrightnessState) + 'static,
    {
        BRIGHTNESS_SERVICE.with(|service| {
            if let Some(ref mut state) = *service.borrow_mut() {
                state.subscribers.push(Box::new(callback));
            }
        });
    }

    fn monitor_changes() {
        use std::thread;
        use std::time::Duration;

        let mut last = Self::get_brightness();
        loop {
            thread::sleep(Duration::from_millis(500));
            let current = Self::get_brightness();
            if (current - last).abs() > 0.5 {
                last = current;
                let state = BrightnessState { brightness: current };
                glib::idle_add_once(move || {
                    BRIGHTNESS_SERVICE.with(|service| {
                        if let Some(ref s) = *service.borrow() {
                            for cb in &s.subscribers {
                                cb(state.clone());
                            }
                        }
                    });
                });
            }
        }
    }

    /// Returns brightness as a percentage (0–100).
    pub fn get_brightness() -> f64 {
        let current = Self::brightnessctl(&["get"]);
        let max = Self::brightnessctl(&["max"]);
        if max > 0.0 {
            (current / max * 100.0).round()
        } else {
            50.0
        }
    }

    /// Sets brightness from a percentage (1–100).
    pub fn set_brightness(percentage: f64) {
        let pct = (percentage.clamp(1.0, 100.0) as u32).to_string() + "%";
        Command::new("brightnessctl")
            .args(["set", &pct])
            .output()
            .ok();
    }

    fn brightnessctl(args: &[&str]) -> f64 {
        Command::new("brightnessctl")
            .args(args)
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.trim().parse::<f64>().ok())
            .unwrap_or(0.0)
    }
}
