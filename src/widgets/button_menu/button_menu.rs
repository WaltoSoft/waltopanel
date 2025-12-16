use adw::subclass::prelude::ObjectSubclassIsExt;
use gtk::glib;

use crate::{models::MenuItemModel, types::TypedListStore};
use super::imp::ButtonMenuPrivate;

glib::wrapper! {
  pub struct ButtonMenu(ObjectSubclass<ButtonMenuPrivate>)
    @extends gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ButtonMenu{
  pub fn from_icon_name_and_label(icon_name: &str, label: &str) -> Self {
    let button_menu: Self = glib::Object::new();
    button_menu.imp().set_icon_name(icon_name);
    button_menu.imp().set_label(label);
    button_menu
  }

  pub fn from_icon_name(icon_name: &str) -> Self {
    let button_menu: Self = glib::Object::new();
    button_menu.imp().set_icon_name(icon_name);
    button_menu
  }

  pub fn from_label(label: &str) -> Self {
    let button_menu: Self = glib::Object::new();
    button_menu.imp().set_label(label);
    button_menu
  }

  pub fn set_menu(&self, menu: TypedListStore<MenuItemModel>) {
    self.imp().set_menu(menu);
  }
}