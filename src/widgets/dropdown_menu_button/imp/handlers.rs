use adw::subclass::prelude::{ObjectSubclassExt, ObjectSubclassIsExt};
use gtk::{glib::WeakRef, prelude::{ButtonExt, PopoverExt, WidgetExt}};
use gtk::{Box, Button, GestureClick, Popover};
use std::cell::OnceCell;

use crate::models::MenuItem;
use super::{DropdownMenuButtonPrivate, state::MenuState};
use super::super::DropdownMenuButton;

impl DropdownMenuButtonPrivate {
  /// Handle the click event for the dropdown menu button.
  /// If the popover is visible, it will be hidden. Otherwise, the popover 
  /// will be shown and all other dropdowns will be closed.
  ///
  /// This ensures that only one dropdown is open at a time.
  /// 
  /// Parameters:
  /// - `popover`: The popover associated with the dropdown menu button.
  /// - `button`: The button that was clicked.
  pub fn handle_dropdown_menu_button_click(&self) {
    if let (Some(button), Some(popover)) = (self.dropdown_menu_button.get(), self.popover.get()) {
      if popover.is_visible() {
        popover.popdown();
      } else {
        Self::close_all_other_dropdowns(popover);
        Self::update_popover_alignment(popover, button);
        popover.popup();
      }
    }
  }

  pub fn handle_popover_show(&self) {
    self.state.reset_to_root_menu();
    self.rebuild_menu();
  }

  pub fn handle_menu_item_click(&self, menu_item: &MenuItem) {
    if let Some(popover) = self.popover.get() {
      popover.popdown();
    }

    if menu_item.is_toggleable {
      self.emit_item_toggled(&menu_item.id, ! menu_item.is_toggled);
    } else {
      self.emit_item_selected(&menu_item.id);
    }
  }

  pub fn handle_submenu_click(&self, menu_item: &MenuItem) {
    if let Some(submenu_items) = menu_item.submenu.clone() {
      let sub_menu_label = menu_item.label.clone();
      let current_menu = self.state.menu_items.borrow().clone();

      self.state.sub_menu_stack.borrow_mut().push(current_menu);
      self.state.sub_menu_breadcrumbs.borrow_mut().push(sub_menu_label);    
      *self.state.menu_items.borrow_mut() = submenu_items;

      self.rebuild_menu();
    }
  }


  pub fn handle_submenu_back_button_click(&self) {
    let mut stack = self.state.sub_menu_stack.borrow_mut();

    let should_rebuild = {
      if let Some(previous_menu) = stack.pop() {
        drop(stack);

        self.state.sub_menu_breadcrumbs.borrow_mut().pop();
        *self.state.menu_items.borrow_mut() = previous_menu;
        true
      }
      else {
        false
      }
    };

    if should_rebuild {
      self.rebuild_menu();
    }
  }

  pub fn attach_dropdown_menu_button_handlers(&self){
    let obj_weak = self.obj().downgrade();

    if let Some(button) = self.dropdown_menu_button.get() {
      button.connect_clicked(move |_| {
        if let Some(obj) = obj_weak.upgrade() {
          obj.imp().handle_dropdown_menu_button_click();
        }
      });
    }

    let obj_weak_popover = self.obj().downgrade();
    if let Some(popover) = self.popover.get() {
      popover.connect_show(move |_| {
        if let Some(obj) = obj_weak_popover.upgrade() {
          obj.imp().handle_popover_show();
        }
      });
    }
  }

  pub fn attach_menu_item_handlers(&self, menu_item_container: &gtk::Box, menu_item: &MenuItem) {
    let obj_weak = self.obj().downgrade();
    let menu_item_clone = menu_item.clone();
    let event_controller = GestureClick::new();

    menu_item_container.add_controller(event_controller.clone());

    if let Some(obj) = obj_weak.upgrade() {
      if menu_item.has_submenu() {
        event_controller.connect_released(move |_, _, _, _| {
          obj.imp().handle_submenu_click(&menu_item_clone);
        });
      }
      else {
        event_controller.connect_released(move |_, _, _, _| {
          obj.imp().handle_menu_item_click(&menu_item_clone);
        });
      }
    }
  }

  pub fn attach_submenu_back_button_handler(&self, menu_item_container: &Box) {
    let obj_weak = self.obj().downgrade();
    let event_controller = GestureClick::new();

    menu_item_container.add_controller(event_controller.clone());

    event_controller.connect_released(move |_, _, _, _| {
      if let Some(obj) = obj_weak.upgrade() {
        obj.imp().handle_submenu_back_button_click();
      }
    });
  }  
}
