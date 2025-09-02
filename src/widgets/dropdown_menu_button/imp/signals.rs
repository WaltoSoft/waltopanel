use adw::subclass::prelude::ObjectSubclassExt;
use gtk::glib::{object::ObjectExt, subclass::Signal, types::StaticType};
use std::sync::OnceLock;

use super::DropdownMenuButtonPrivate;

impl DropdownMenuButtonPrivate {
  pub fn emit_item_selected(&self, item_id: &str) {
    self.obj().emit_by_name::<()>("item-selected", &[&item_id]);
  }

  pub fn emit_item_toggled(&self, item_id: &str, toggled_state: bool) {
    self.obj().emit_by_name::<()>("item-toggled", &[&item_id, &toggled_state]);
  }

  pub fn setup_signals() -> &'static [Signal] {
    static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
    SIGNALS.get_or_init(|| {
      vec![
        Signal::builder("item-selected")
          .param_types([String::static_type()])
          .build(),
        Signal::builder("item-toggled")
          .param_types([String::static_type(), bool::static_type()])
          .build(),
      ]
    })
  }
}