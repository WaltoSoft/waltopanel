use gtk::glib;
use std::{cell::RefCell, process::Command};

#[derive(Debug, Clone)]
pub struct MicrophoneState {
    pub volume: f64,
    pub is_muted: bool,
}

type MicrophoneCallback = Box<dyn Fn(MicrophoneState)>;

struct MicrophoneServiceState {
    subscribers: Vec<MicrophoneCallback>,
}

thread_local! {
    static MICROPHONE_SERVICE: RefCell<Option<MicrophoneServiceState>> = RefCell::new(None);
}

pub struct MicrophoneService;

impl MicrophoneService {
    pub fn start() {
        MICROPHONE_SERVICE.with(|service| {
            if service.borrow().is_some() {
                return;
            }
            *service.borrow_mut() = Some(MicrophoneServiceState {
                subscribers: Vec::new(),
            });
            std::thread::spawn(|| Self::monitor_changes());
        });
    }

    pub fn subscribe<F>(callback: F)
    where
        F: Fn(MicrophoneState) + 'static,
    {
        MICROPHONE_SERVICE.with(|service| {
            if let Some(ref mut state) = *service.borrow_mut() {
                state.subscribers.push(Box::new(callback));
            }
        });
    }

    fn monitor_changes() {
        use std::thread;
        use std::time::Duration;

        let mut last_volume = Self::get_volume();
        let mut last_muted = Self::is_muted();

        loop {
            thread::sleep(Duration::from_millis(200));
            let current_volume = Self::get_volume();
            let current_muted = Self::is_muted();

            if current_volume != last_volume || current_muted != last_muted {
                last_volume = current_volume;
                last_muted = current_muted;
                let state = MicrophoneState { volume: current_volume, is_muted: current_muted };
                glib::idle_add_once(move || {
                    MICROPHONE_SERVICE.with(|service| {
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

    pub fn get_volume() -> f64 {
        let output = Command::new("wpctl")
            .args(["get-volume", "@DEFAULT_AUDIO_SOURCE@"])
            .output();
        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(v) = stdout.split_whitespace().nth(1) {
                if let Ok(v) = v.parse::<f64>() {
                    return (v * 100.0).round();
                }
            }
        }
        50.0
    }

    pub fn set_volume(volume: f64) {
        let v = format!("{:.2}", (volume / 100.0).clamp(0.0, 1.0));
        Command::new("wpctl")
            .args(["set-volume", "@DEFAULT_AUDIO_SOURCE@", &v])
            .output()
            .ok();
    }

    pub fn is_muted() -> bool {
        let output = Command::new("wpctl")
            .args(["get-volume", "@DEFAULT_AUDIO_SOURCE@"])
            .output();
        if let Ok(output) = output {
            return String::from_utf8_lossy(&output.stdout).contains("[MUTED]");
        }
        false
    }

    pub fn toggle_mute() {
        Command::new("wpctl")
            .args(["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"])
            .output()
            .ok();
    }
}
