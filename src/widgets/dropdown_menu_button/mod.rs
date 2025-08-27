mod imp;
mod navigation;
mod state;
mod styling;
mod builder;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use crate::models::MenuItem;

glib::wrapper! {
  pub struct DropdownMenuButton(ObjectSubclass<imp::DropdownMenuButtonPrivate>)
    @extends gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

//-------------------------------------------------------------------------------------------------
// Public API
//-------------------------------------------------------------------------------------------------
impl DropdownMenuButton {
  pub fn new() -> Self {
    glib::Object::new()
  }

  pub fn set_text(&self, text: &str) {
    self.imp().set_button_text(text);
  }

  pub fn set_icon(&self, icon_name: &str) {
    self.imp().set_button_icon(icon_name);
  }

  pub fn set_icon_and_text(&self, icon_name: &str, text: &str) {
    self.imp().set_button_icon_and_text(icon_name, text);
  }

  pub fn set_menu_items(&self, items: Vec<MenuItem>) {
    self.imp().set_menu_items(items);
  }

  pub fn set_item_toggled(&self, item_id: &str, toggled: bool) {
    self.imp().set_item_toggled(item_id, toggled);
  }
}

//-------------------------------------------------------------------------------------------------
// Event Handlers Provided
//-------------------------------------------------------------------------------------------------
impl DropdownMenuButton {
  pub fn connect_item_selected<F>(&self, callback: F) -> glib::SignalHandlerId
  where
    F: Fn(&Self, &str) + 'static,
  {
    self.connect_local("item-selected", false, move |values| {
      let Ok(dropdown) = values[0].get::<DropdownMenuButton>() else {
        return None;
      };
      let Ok(item_id) = values[1].get::<String>() else {
        return None;
      };
      callback(&dropdown, &item_id);
      None
    })
  }

  pub fn connect_item_toggled<F>(&self, callback: F) -> glib::SignalHandlerId
  where
    F: Fn(&Self, &str, bool) + 'static,
  {
    self.connect_local("item-toggled", false, move |values| {
      let Ok(dropdown) = values[0].get::<DropdownMenuButton>() else {
        return None;
      };
      let Ok(item_id) = values[1].get::<String>() else {
        return None;
      };
      let Ok(toggled_state) = values[2].get::<bool>() else {
        return None;
      };
      callback(&dropdown, &item_id, toggled_state);
      None
    })
  }
}