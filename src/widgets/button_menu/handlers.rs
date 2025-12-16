use adw::subclass::prelude::{ObjectSubclassExt, ObjectSubclassIsExt};
use gtk::Box;
use gtk::{prelude::WidgetExt, GestureClick};

use crate::models::MenuItemModel;

use super::ButtonMenuPrivate;

impl ButtonMenuPrivate {
  pub fn attach_button_menu_handlers(&self) {
    self.connect_button_menu_click_handler();
    self.connect_dropdown_show_handler();
  }

  pub fn attach_menu_item_handlers(&self, menu_item_box: &Box, menu_item: &MenuItemModel) {
    let obj_weak = self.obj().downgrade();
    let menu_item_clone = menu_item.clone();
    let event_controller: GestureClick = GestureClick::new();

    menu_item_box.add_controller(event_controller.clone());

    if let Some(obj) = obj_weak.upgrade() {
      if menu_item.has_submenu() {
        event_controller.connect_released(move |_, _, _, _| {
          obj.imp().show_submenu(&menu_item_clone);
        });
      }
      else {
        event_controller.connect_released(move |_, _, _, _| {
//          obj.imp().handle_menu_item_click(&menu_item_clone);
        });
      }
    }
  }


  fn connect_button_menu_click_handler(&self) {
    let obj_weak = self.obj().downgrade();
    if let Some(button_menu_box) = self.button_menu_box.get() {
      let click_gesture = GestureClick::new();
      button_menu_box.add_controller(click_gesture.clone());

      click_gesture.connect_released(move |_, _, _, _| {
        if let Some(obj) = obj_weak.upgrade() {
          obj.imp().toggle_popover();
        }
      });
    }
  }

  fn connect_dropdown_show_handler(&self) {
    let obj_weak = self.obj().downgrade();
    if let Some(popover) = self.popover.get() {
      popover.connect_show(move |_| {
        if let Some(obj) = obj_weak.upgrade() {
          obj.imp().reset_menu();
        }
      });
    }
  }
}