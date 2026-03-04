use gtk::glib;
use std::{cell::RefCell, process::Command};

#[derive(Debug, Clone)]
pub struct VolumeState {
  pub volume: f64,
  pub is_muted: bool,
}

type VolumeCallback = Box<dyn Fn(VolumeState)>;

struct SoundServiceState {
  subscribers: Vec<VolumeCallback>,
}

thread_local! {
  static SOUND_SERVICE: RefCell<Option<SoundServiceState>> = RefCell::new(None);
}

pub struct SoundService;

impl SoundService {
  pub fn start() {
    SOUND_SERVICE.with(|service| {
      if service.borrow().is_some() {
        return; // Already started
      }

      let state = SoundServiceState {
        subscribers: Vec::new(),
      };

      *service.borrow_mut() = Some(state);

      // Start monitoring volume changes in background thread
      std::thread::spawn(|| {
        Self::monitor_volume_changes();
      });
    });
  }

  pub fn subscribe<F>(callback: F)
  where
    F: Fn(VolumeState) + 'static
  {
    SOUND_SERVICE.with(|service| {
      if let Some(ref mut state) = *service.borrow_mut() {
        state.subscribers.push(Box::new(callback));
      }
    });
  }

  fn monitor_volume_changes() {
    use std::thread;
    use std::time::Duration;

    let mut last_volume = Self::get_volume();
    let mut last_muted = Self::is_muted();

    loop {
      thread::sleep(Duration::from_millis(200));

      let current_volume = Self::get_volume();
      let current_muted = Self::is_muted();

      // Only notify if something actually changed
      if current_volume != last_volume || current_muted != last_muted {
        last_volume = current_volume;
        last_muted = current_muted;

        let volume_state = VolumeState {
          volume: current_volume,
          is_muted: current_muted,
        };

        // Notify all subscribers on main thread
        glib::idle_add_once(move || {
          SOUND_SERVICE.with(|service| {
            if let Some(ref state) = *service.borrow() {
              for callback in &state.subscribers {
                callback(volume_state.clone());
              }
            }
          });
        });
      }
    }
  }

  /// Get the current volume percentage (0-100)
  pub fn get_volume() -> f64 {
    let output = Command::new("wpctl")
      .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
      .output();

    if let Ok(output) = output {
      let stdout = String::from_utf8_lossy(&output.stdout);
      // Output format: "Volume: 0.50" (where 0.50 = 50%)
      if let Some(volume_str) = stdout.split_whitespace().nth(1) {
        if let Ok(volume) = volume_str.parse::<f64>() {
          return (volume * 100.0).round();
        }
      }
    }

    50.0 // Default fallback
  }

  /// Set the volume percentage (0-100)
  pub fn set_volume(volume: f64) {
    let volume_decimal = (volume / 100.0).clamp(0.0, 1.0);
    let volume_str = format!("{:.2}", volume_decimal);

    Command::new("wpctl")
      .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &volume_str])
      .output()
      .ok();
  }

  /// Check if audio is muted
  pub fn is_muted() -> bool {
    let output = Command::new("wpctl")
      .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
      .output();

    if let Ok(output) = output {
      let stdout = String::from_utf8_lossy(&output.stdout);
      return stdout.contains("[MUTED]");
    }

    false
  }

  /// Toggle mute state
  pub fn toggle_mute() {
    Command::new("wpctl")
      .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
      .output()
      .ok();
  }
}
