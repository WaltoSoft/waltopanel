use adw::ApplicationWindow;
use gtk::prelude::WidgetExt;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::boxed::Box as StdBox;

const SIZE_AUTOMATIC: i32 = -1;

pub trait ApplicationWindowExtensions {
  fn configure_top_layer_shell(
    &self,
    size: i32,
  ) -> Result<(), StdBox<dyn std::error::Error>>;
}

impl ApplicationWindowExtensions for ApplicationWindow {
  fn configure_top_layer_shell(
    &self,
    size: i32,
  ) -> Result<(), StdBox<dyn std::error::Error>> {
    self.init_layer_shell();
    self.set_layer(Layer::Top);
    self.auto_exclusive_zone_enable();
    self.set_anchor(Edge::Top, true);
    self.set_anchor(Edge::Left, true);
    self.set_anchor(Edge::Right, true);
    self.set_anchor(Edge::Bottom, false);
    self.set_keyboard_mode(KeyboardMode::OnDemand);
    self.set_can_focus(true);
    self.set_focusable(true);
    self.set_size_request(SIZE_AUTOMATIC, size);

    Ok(())
  }
}