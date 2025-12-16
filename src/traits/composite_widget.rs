use gtk::{Widget, prelude::WidgetExt};



pub trait CompositeWidget {
  fn widget(&self) -> Widget;
  fn set_parent(&self, parent: &impl CompositeWidget) {
    self.widget().set_parent(&parent.widget());
  }
  fn unparent(&self) {
    self.widget().unparent();
  }
}