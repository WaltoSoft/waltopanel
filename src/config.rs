use gtk4_layer_shell::Layer;
use serde::Deserialize;

#[derive(Clone)]
pub struct WaltoPanelConfig {
  pub height: i32,
  pub _layer: Layer,
  pub margins: Margins,
  pub button_spacing: i32,
  pub layout: PanelLayoutConfig,
}

impl Default for WaltoPanelConfig {
  fn default() -> Self {
    Self {
      height: 16,
      _layer: Layer::Top,
      margins: Margins {
        top: 8,
        bottom: 8,
        left: 8,
        right: 8,
      },
      button_spacing: 0,
      layout: PanelLayoutConfig::default(),
    }
  }
}

#[derive(Clone)]
pub struct Margins {
  pub top: i32,
  pub bottom: i32,
  pub left: i32,
  pub right: i32,
}

#[derive(Clone, Deserialize, Default)]
pub struct PanelLayoutConfig {
  #[serde(default)]
  pub left: Vec<PanelButtonConfig>,
  #[serde(default)]
  pub center: Vec<PanelButtonConfig>,
  #[serde(default)]
  pub right: Vec<PanelButtonConfig>,
}

impl PanelLayoutConfig {
  pub fn default_layout() -> Self {
    Self {
      left: vec![PanelButtonConfig::Launch {
        icon: "view-app-grid-symbolic".to_string(),
        command: "pkill rofi || /home/billy/.config/waltoland/scripts/rofi-alphabetical-apps.sh"
          .to_string(),
      }],
      center: vec![],
      right: vec![PanelButtonConfig::Clock],
    }
  }

  pub fn load_from_file() -> Self {
    let config_path = std::env::var("HOME").ok().map(|home| {
      std::path::PathBuf::from(home).join(".config/waltopanel/config.json")
    });

    if let Some(path) = config_path {
      if path.exists() {
        match std::fs::read_to_string(&path) {
          Ok(content) => match serde_json::from_str::<PanelLayoutConfig>(&content) {
            Ok(layout) => return layout,
            Err(e) => eprintln!("waltopanel: failed to parse config: {}", e),
          },
          Err(e) => eprintln!("waltopanel: failed to read config: {}", e),
        }
      }
    }

    Self::default_layout()
  }
}

#[derive(Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PanelButtonConfig {
  Launch { icon: String, command: String },
  Clock,
  Weather { location: String },
  Workspace,
  Network,
  Brightness,
  Microphone,
  Sound,
  Battery,
  System,
  SystemMetrics,
}
