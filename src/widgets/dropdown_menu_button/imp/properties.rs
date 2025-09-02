use gtk::{prelude::{BoxExt, ButtonExt}, Image, Label, Orientation};

use crate::models::MenuItem;
use super::DropdownMenuButtonPrivate;

impl DropdownMenuButtonPrivate {
  pub fn set_button_text(&self, text: &str) {
    if let Some(button) = self.dropdown_menu_button.get() {
      button.set_label(text);
    }
  }

  pub fn set_button_icon(&self, icon_name: &str) {
    if let Some(button) = self.dropdown_menu_button.get() {
      let icon = Image::from_icon_name(icon_name);
      button.set_child(Some(&icon));
    }
  }

  pub fn set_button_icon_and_text(&self, icon_name: &str, text: &str) {
    if let Some(button) = self.dropdown_menu_button.get() {
      let button_container = Self::create_styled_box(Orientation::Horizontal, 6, vec![]);

      let icon = Image::from_icon_name(icon_name);
      let label = Label::new(Some(text));

      button_container.append(&icon);
      button_container.append(&label);

      button.set_child(Some(&button_container));
    }
  }

  pub fn set_menu_items(&self, items: Vec<MenuItem>) {
    *self.state.menu_items.borrow_mut() = items;
    self.rebuild_menu();
  }

  pub fn set_menuitem_toggled(&self, item_id: &str, toggled: bool) {
    let mut items = self.state.menu_items.borrow_mut();
    
    if let Some(item) = items.iter_mut().find(|i| i.id == item_id) {
      if item.is_toggleable {
        item.is_toggled = toggled;
      }
    }

    drop(items);
    self.rebuild_menu();
  }
}
