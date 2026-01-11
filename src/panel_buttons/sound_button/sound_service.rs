use std::process::Command;

pub struct SoundService;

impl SoundService {
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
