use gtk::{gio::ListStore, prelude::*};
use gtk::glib::{self, Object, object_subclass, ParamSpec, ParamSpecString, ParamSpecBoolean, ParamSpecObject, Value};
use gtk::subclass::prelude::*;
use std::cell::RefCell;

use crate::types::TypedListStore;

use super::MenuItemModel;

pub struct MenuItemModelImp {
  pub(super) id: RefCell<String>,
  pub(super) text: RefCell<String>,
  pub(super) icon_name: RefCell<Option<String>>,
  pub(super) post_label_icon_name: RefCell<Option<String>>,
  pub(super) toggled: RefCell<bool>,
  pub(super) allow_toggle: RefCell<bool>,
  pub(super) separator_after: RefCell<bool>,
  pub(super) disabled: RefCell<bool>,
  pub(super) submenu: RefCell<TypedListStore<MenuItemModel>>,
}

impl Default for MenuItemModelImp {
  fn default() -> Self {
    Self {
      id: RefCell::new(String::new()),
      text: RefCell::new(String::new()),
      icon_name: RefCell::new(None),
      post_label_icon_name: RefCell::new(None),
      toggled: RefCell::new(false),
      allow_toggle: RefCell::new(false),
      separator_after: RefCell::new(false),
      disabled: RefCell::new(false),
      submenu: RefCell::new(TypedListStore::new()),
    }
  }
}

#[object_subclass]
impl ObjectSubclass for MenuItemModelImp {
  const NAME: &'static str = "MenuItemModel";
  type Type = MenuItemModel;
  type ParentType = Object;
}

impl ObjectImpl for MenuItemModelImp {
  fn properties() -> &'static [ParamSpec] {
    use std::sync::OnceLock;
    static PROPERTIES: OnceLock<Vec<ParamSpec>> = OnceLock::new();
    PROPERTIES.get_or_init(|| {
      vec![
        ParamSpecString::builder("id").build(),
        ParamSpecString::builder("text").build(),
        ParamSpecString::builder("icon-name").build(),
        ParamSpecString::builder("post-label-icon-name").build(),
        ParamSpecBoolean::builder("toggled").build(),
        ParamSpecBoolean::builder("allow-toggle").build(),
        ParamSpecBoolean::builder("separator-after").build(),
        ParamSpecBoolean::builder("disabled").build(),
        ParamSpecObject::builder::<ListStore>("submenu").build(),
      ]
    })
  }

  fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
    match pspec.name() {
      "id" => self.id.borrow().to_value(),
      "text" => self.text.borrow().to_value(),
      "icon-name" => self.icon_name.borrow().to_value(),
      "post-label-icon-name" => self.post_label_icon_name.borrow().to_value(),
      "toggled" => self.toggled.borrow().to_value(),
      "allow-toggle" => self.allow_toggle.borrow().to_value(),
      "separator-after" => self.separator_after.borrow().to_value(),
      "disabled" => self.disabled.borrow().to_value(),
      "submenu" => {
        let typed_store = self.submenu.borrow();
        typed_store.as_list_store().to_value()
      }
      _ => unimplemented!(),
    }
  }

  fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
    match pspec.name() {
      "id" => {
          let id = value.get().expect("type checked upstream");
          self.id.replace(id);
      }
      "text" => {
          let text = value.get().expect("type checked upstream");
          self.text.replace(text);
      }
      "icon-name" => {
          let icon_name = value.get().expect("type checked upstream");
          self.icon_name.replace(icon_name);
      }
      "post-label-icon-name" => {
          let post_label_icon_name = value.get().expect("type checked upstream");
          self.post_label_icon_name.replace(post_label_icon_name);
      }
      "toggled" => {
          let toggled = value.get().expect("type checked upstream");
          self.toggled.replace(toggled);
      }
      "allow-toggle" => {
          let allow_toggle = value.get().expect("type checked upstream");
          self.allow_toggle.replace(allow_toggle);
      }
      "separator-after" => {
          let separator_after = value.get().expect("type checked upstream");
          self.separator_after.replace(separator_after);
      }
      "disabled" => {
          let disabled = value.get().expect("type checked upstream");
          self.disabled.replace(disabled);
      }
      "submenu" => {
          let list_store: ListStore = value.get().expect("type checked upstream");
          let typed_store = TypedListStore::from_list_store(list_store);
          self.submenu.replace(typed_store);
      }
      _ => unimplemented!(),
    }
  }
}