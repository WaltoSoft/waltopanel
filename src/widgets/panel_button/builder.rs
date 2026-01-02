use gtk::glib::object::{Object, ObjectBuilder};
use crate::widgets::PanelButton;

pub struct PanelButtonBuilder {
  builder: ObjectBuilder<'static, PanelButton>,
} 

impl PanelButtonBuilder {
  fn new() -> Self {
    Self {
      builder: Object::builder(),
    }
  }

  pub fn icon_name(self, icon_name: impl Into<String>) -> Self {
    Self {
      builder: self.builder.property("icon-name", icon_name.into()),
    }
  }

  pub fn text(self, text: impl Into<String>) -> Self {
    Self {
      builder: self.builder.property("text", text.into()),
    }
  }

  pub fn build(self) -> PanelButton {
    self.builder.build()
  }
}