use gtk::{glib, gio::ListStore, prelude::*};

use crate::types::TypedListStore;
use super::imp::MenuItemModelImp;

glib::wrapper! {
  pub struct MenuItemModel(ObjectSubclass<MenuItemModelImp>);
}

impl MenuItemModel {
  pub fn new(id: &str, text: &str) -> Self {
    let obj: Self = glib::Object::builder()
      .property("id", id)
      .property("text", text)
      .property("submenu", ListStore::new::<MenuItemModel>())
      .build();
    obj
  }

  pub fn id(&self) -> String {
    self.property("id")
  }
  
  pub fn set_id(&self, id: &str) {
    self.set_property("id", id);
  }
  
  pub fn text(&self) -> String {
    self.property("text")
  }
  
  pub fn set_text(&self, text: &str) {
    self.set_property("text", text);
  }
  
  pub fn icon_name(&self) -> Option<String> {
    self.property("icon-name")
  }
  
  pub fn set_icon_name(&self, icon_name: Option<&str>) {
    self.set_property("icon-name", icon_name);
  }
  
  pub fn toggled(&self) -> bool {
    self.property("toggled")
  }
  
  pub fn set_toggled(&self, toggled: bool) {
    self.set_property("toggled", toggled);
  }
  
  pub fn allow_toggle(&self) -> bool {
    self.property("allow-toggle")
  }
  
  pub fn set_allow_toggle(&self, allow_toggle: bool) {
    self.set_property("allow-toggle", allow_toggle);
  }

  pub fn separator_after(&self) -> bool {
    self.property("separator-after")
  }

  pub fn set_separator_after(&self, separator_after: bool) {
    self.set_property("separator-after", separator_after);
  }
  
  pub fn submenu(&self) -> TypedListStore<MenuItemModel> {
    let list_store: ListStore = self.property("submenu");
    TypedListStore::from_list_store(list_store)
  }
  
  pub fn set_submenu(&self, submenu: ListStore) {
    self.set_property("submenu", submenu);
  }
  
  pub fn has_submenu(&self) -> bool {
    let submenu = self.submenu();
    !submenu.is_empty()
  }
}