use crate::{models::MenuItemModel, types::TypedListStore};

use super::ButtonMenuPrivate;

impl ButtonMenuPrivate {
  pub(super) fn set_icon_name(&self, icon_name: &str) {
    *self.icon_name.borrow_mut() = Some(icon_name.to_string());
    self.refresh_icon_image();
  }

  pub(super) fn set_label(&self, label: &str) {
    *self.text.borrow_mut() = Some(label.to_string());
    self.refresh_text_label();
  }

  pub (super) fn set_menu(&self, menu: TypedListStore<MenuItemModel>) {
    self.menu_data.set(menu).expect("set_menu called twice");
  }
}