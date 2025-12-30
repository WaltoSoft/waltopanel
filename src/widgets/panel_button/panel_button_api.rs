use adw::subclass::prelude::ObjectSubclassIsExt;
use gtk::prelude::WidgetExt;
use gtk::{glib::{self, object::ObjectExt, value::ToValue}};
use indexmap::IndexMap;
use uuid::Uuid;
use std::cell::RefCell;

use crate::{models::MenuItemModel, types::TypedListStore};
use super::panel_button_imp::PanelButtonImp;

glib::wrapper! {
  pub struct PanelButton(ObjectSubclass<PanelButtonImp>)
    @extends gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

thread_local! {
  static INSTANCES: RefCell<IndexMap<Uuid, PanelButton>> = RefCell::new(IndexMap::new());
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

  pub fn show_menu(&self) {
    self.imp().show_menu();
  }

  pub fn hide_menu(&self) {
    self.imp().hide_menu();
  }

  pub fn id(&self) -> uuid::Uuid {
    self.imp().id
  }
}

// Instance Management ----------------------------------------------------------------------------
impl PanelButton {
  pub(super) fn register_instance(instance: &PanelButton) {
    INSTANCES.with(|instances| {
      instances.borrow_mut().insert(instance.id(), instance.clone());
    });

    let instance_for_cleanup = instance.clone();
    instance.connect_destroy(move |_| {
      INSTANCES.with(|instances| {
        instances.borrow_mut().shift_remove(&instance_for_cleanup.id());
      });
    });
  }

  pub(super) fn close_other_instances(current_panel_button: &PanelButton) {
    INSTANCES.with(|instances| {
      instances.borrow().values().for_each(|panel_button| {
        if panel_button != current_panel_button {
          panel_button.hide_menu();
        }
      });
    });
  }

  pub(super) fn get_next_instance(&self) -> Option<PanelButton> {
    INSTANCES.with(|instances| {
      let instances = instances.borrow();
      let current_index = instances.get_index_of(&self.id())?;
      let next_index = (current_index + 1) % instances.len();
      instances.get_index(next_index).map(|(_, pb)| pb.clone())
    })
  }

  pub(super) fn get_previous_instance(&self) -> Option<PanelButton> {
    INSTANCES.with(|instances| {
      let instances = instances.borrow();
      let current_index = instances.get_index_of(&self.id())?;
      let prev_index = if current_index == 0 { 
        instances.len() - 1 
      } else { 
        current_index - 1 
      };
      instances.get_index(prev_index).map(|(_, pb)| pb.clone())
    })
  }
}
// End Instance Management ------------------------------------------------------------------------
