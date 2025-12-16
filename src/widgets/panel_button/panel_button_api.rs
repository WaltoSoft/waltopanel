use gtk::glib::{self, object::ObjectExt, value::ToValue};

use crate::{models::MenuItemModel, types::TypedListStore};
use super::panel_button_imp::PanelButtonImp;

glib::wrapper! {
  pub struct PanelButton(ObjectSubclass<PanelButtonImp>)
    @extends gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PanelButton {
  pub fn from_icon_name_and_label(icon_name: &str, label: &str) -> Self {
    let panel_button: Self = glib::Object::new();
    panel_button.set_property("icon-name", &icon_name.to_value()); 
    panel_button.set_property("text", &label.to_value());
    panel_button
  }

  pub fn from_icon_name(icon_name: &str) -> Self {
    let panel_button: Self = glib::Object::new();
    panel_button.set_property("icon-name", &icon_name.to_value());
    panel_button
  }

  pub fn from_label(label: &str) -> Self {
    let panel_button: Self = glib::Object::new();
    panel_button.set_property("text", &label.to_value());
    panel_button
  }

  pub fn set_menu(&self, menu: TypedListStore<MenuItemModel>) {
    self.set_property("menu", menu.as_list_store());
  }
}