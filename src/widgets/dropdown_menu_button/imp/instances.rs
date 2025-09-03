use std::cell::RefCell;
use adw::subclass::prelude::ObjectSubclassIsExt;
use gtk::{prelude::{PopoverExt, WidgetExt}, Popover};

use crate::widgets::DropdownMenuButton;

use super::DropdownMenuButtonPrivate;

thread_local! {
  static INSTANCES: RefCell<Vec<DropdownMenuButton>> = RefCell::new(Vec::new());
}

impl DropdownMenuButtonPrivate {
  pub fn register_instance(instance: &DropdownMenuButton) {
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

  pub fn close_all_other_dropdowns(current_popover: &Popover) {
    INSTANCES.with(|instances| {
      instances.borrow().iter().for_each(|dropdown| {
        if let Some(popover) = dropdown.imp().popover.get() {
          if popover != current_popover && popover.is_visible() {
              popover.popdown();
           }
        }
      });
    });
  }
}