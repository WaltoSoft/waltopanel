use std::cell::RefCell;
use adw::subclass::prelude::ObjectSubclassIsExt;
use gtk::{prelude::{PopoverExt, WidgetExt}, Popover};

use super::{ButtonMenu, ButtonMenuPrivate};

thread_local! {
  static INSTANCES: RefCell<Vec<ButtonMenu>> = RefCell::new(Vec::new());
}

impl ButtonMenuPrivate {
  pub fn register_instance(instance: &ButtonMenu) {
    INSTANCES.with(|instances| {
      instances.borrow_mut().push(instance.clone());
    });

    let instance_for_cleanup = instance.clone();
    instance.connect_destroy(move |_| {
      INSTANCES.with(|instances| {
        let mut instances = instances.borrow_mut();
        instances.retain(|i| i != &instance_for_cleanup);
      });
    });
  }

  pub fn close_other_button_menus(current_popover: &Popover) {
    INSTANCES.with(|instances| {
      instances.borrow().iter().for_each(|button_menu| {
        if let Some(popover) = button_menu.imp().popover.get() {
          if popover != current_popover && popover.is_visible() {
              popover.popdown();
           }
        }
      });
    });
  }
}