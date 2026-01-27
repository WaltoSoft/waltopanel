use gtk::{Widget, glib::object::{Object, ObjectBuilder}};
use crate::widgets::PanelButton;

pub struct PanelButtonBuilder {
  builder: ObjectBuilder<'static, PanelButton>,
} 

impl PanelButtonBuilder {
  pub fn new() -> Self {
    Self {
      builder: Object::builder(),
    }
  }

  pub fn custom_widget(self, widget: Option<Widget>) -> Self {
    Self {
      builder: self.builder.property("custom-widget", widget),
    }
  }

  pub fn dropdown_widget(self, widget: Widget) -> Self {
    Self {
      builder: self.builder.property("dropdown-widget", widget),
    }
  }

  pub fn _icon_name(self, icon_name: impl Into<String>) -> Self {
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