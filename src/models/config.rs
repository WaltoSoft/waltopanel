use gtk4_layer_shell::Layer;

#[derive(Clone)]
pub struct CurtainBarConfig {
  pub height: i32,
  pub layer: Layer,
  pub margins: Margins,
  pub spacing: i32,
}

impl Default for CurtainBarConfig {
  fn default() -> Self {
    Self {
      height: 40,
      layer: Layer::Top,
      margins: Margins {
        top: 4,
        bottom: 4,
        left: 8,
        right: 8,
      },
      spacing: 8,
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