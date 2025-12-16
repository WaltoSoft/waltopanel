use gtk::{glib, gio::ListStore, prelude::*};

use crate::types::TypedListStore;

use super::imp::MenuItemModelPrivate;

glib::wrapper! {
  pub struct MenuItemModel(ObjectSubclass<MenuItemModelPrivate>);
}

impl MenuItemModel {
  pub fn new(id: &str, text: &str) -> Self {
    let obj: Self = glib::Object::builder()
      .property("id", id)
      .property("text", text)
      .build();

      obj.set_property("submenu", ListStore::new::<MenuItemModel>());
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
  
  pub fn is_separator(&self) -> bool {
    self.property("is-separator")
  }
  
  pub fn set_is_separator(&self, is_separator: bool) {
    self.set_property("is-separator", is_separator);
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

  pub fn with_icon(self, icon: &str) -> Self {
    self.set_icon_name(Some(icon));
    self
  }

  pub fn with_toggle(self) -> Self {
    self.set_allow_toggle(true);
    self
  }

  pub fn toggled_on(self) -> Self {
    self.set_toggled(true);
    self.set_allow_toggle(true);
    self
  }

  pub fn separator() -> Self {
    let obj: Self = glib::Object::builder()
      .property("is-separator", true)
      .build();
    obj.set_property("submenu", ListStore::new::<MenuItemModel>());
    obj
  }
}