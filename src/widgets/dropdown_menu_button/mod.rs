mod imp;
mod navigation;
mod state;
mod styling;
mod utils;
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

impl DropdownMenuButton {
  pub fn new() -> Self {
    glib::Object::new()
  }

  pub fn set_text(&self, text: &str) {
    if let Some(button) = self.imp().button.get() {
      button.set_label(text);
    }
  }

  pub fn set_icon(&self, icon_name: &str) {
    if let Some(button) = self.imp().button.get() {
      let icon = gtk::Image::from_icon_name(icon_name);
      button.set_child(Some(&icon));
    }
  }

  pub fn set_icon_and_text(&self, icon_name: &str, text: &str) {
    if let Some(button) = self.imp().button.get() {
      let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(6)
        .build();

      let icon = gtk::Image::from_icon_name(icon_name);
      let label = gtk::Label::new(Some(text));

      container.append(&icon);
      container.append(&label);

      button.set_child(Some(&container));
    }
  }

  pub fn set_menu_items(&self, items: Vec<MenuItem>) {
    *self.imp().state.menu_items.borrow_mut() = items;
    self.imp().rebuild_menu();
  }

  pub fn connect_item_selected<F>(&self, callback: F) -> glib::SignalHandlerId
  where
    F: Fn(&Self, &str) + 'static,
  {
    self.connect_local("item-selected", false, move |values| {
      let dropdown = values[0].get::<DropdownMenuButton>().unwrap();
      let item_id = values[1].get::<String>().unwrap();
      callback(&dropdown, &item_id);
      None
    })
  }

  pub fn connect_item_toggled<F>(&self, callback: F) -> glib::SignalHandlerId
  where
    F: Fn(&Self, &str, bool) + 'static,
  {
    self.connect_local("item-toggled", false, move |values| {
      let dropdown = values[0].get::<DropdownMenuButton>().unwrap();
      let item_id = values[1].get::<String>().unwrap();
      let toggled_state = values[2].get::<bool>().unwrap();
      callback(&dropdown, &item_id, toggled_state);
      None
    })
  }

  pub fn toggle_item(&self, item_id: &str) -> bool {
    let new_state = {
      let mut items = self.imp().state.menu_items.borrow_mut();
      if let Some(item) = items.iter_mut().find(|i| i.id == item_id) {
        if item.is_toggleable {
          item.is_toggled = !item.is_toggled;
          item.is_toggled
        } else {
          item.is_toggled
        }
      } else {
        false
      }
    };
    self.imp().rebuild_menu();
    new_state
  }

  pub fn set_item_toggled(&self, item_id: &str, toggled: bool) {
    let mut items = self.imp().state.menu_items.borrow_mut();
    if let Some(item) = items.iter_mut().find(|i| i.id == item_id) {
      if item.is_toggleable {
        item.is_toggled = toggled;
      }
    }
    drop(items);
    self.imp().rebuild_menu();
  }
}
