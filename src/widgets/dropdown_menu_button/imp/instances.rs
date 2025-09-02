use std::cell::RefCell;
use gtk::{prelude::{PopoverExt, WidgetExt}, Button, Popover};

use super::DropdownMenuButtonPrivate;

thread_local! {
  static BUTTON_INSTANCES: RefCell<Vec<Button>> = RefCell::new(Vec::new());
  static POPOVER_INSTANCES: RefCell<Vec<Popover>> = RefCell::new(Vec::new());
}

impl DropdownMenuButtonPrivate {
  pub fn register_instance(button: &Button, popover: &Popover) {
    BUTTON_INSTANCES.with(|instances| {
      instances.borrow_mut().push(button.clone());
    });

    POPOVER_INSTANCES.with(|instances| {
      instances.borrow_mut().push(popover.clone());
    });

    let button_for_cleanup = button.clone();
    button.connect_destroy(move |_| {
      BUTTON_INSTANCES.with(|instances| {
        let mut instances = instances.borrow_mut();
        instances.retain(|b| b != &button_for_cleanup);
      });
    });

    let popover_for_cleanup = popover.clone();
    popover.connect_destroy(move |_| {
      POPOVER_INSTANCES.with(|instances| {
        let mut instances = instances.borrow_mut();
        instances.retain(|p| p != &popover_for_cleanup);
      });
    });
  }

  pub fn close_all_other_dropdowns(current_popover: &Popover) {
    POPOVER_INSTANCES.with(|instances| {
      let mut instances = instances.borrow_mut();
      instances.retain(|popover| {
        if let Some(_parent) = popover.parent() {
          if popover != current_popover && popover.is_visible() {
            popover.popdown();
          }
          true
        } else {
          false
        }
      });
    });
  }
}