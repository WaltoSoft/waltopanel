use adw::subclass::prelude::ObjectSubclassIsExt;
use gtk::glib;

use super::imp::PanelButtonGroupImp;
use crate::widgets::PanelButton;

glib::wrapper! {
  pub struct PanelButtonGroup(ObjectSubclass<PanelButtonGroupImp>)
    @extends gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PanelButtonGroup {
  pub fn new() -> Self {
    glib::Object::new()
  }

  pub fn add_button(&self, button: &PanelButton) {
    self.imp().add_button(button);
  }

  pub fn remove_button(&self, button: &PanelButton) {
    self.imp().remove_button(button);
  }

  pub fn clear(&self) {
    self.imp().clear();
  }

  pub fn get_buttons(&self) -> Vec<PanelButton> {
    self.imp().get_buttons()
  }

  pub fn len(&self) -> usize {
    self.imp().get_buttons().len()
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }
}

impl Default for PanelButtonGroup {
  fn default() -> Self {
    Self::new()
  }
}
