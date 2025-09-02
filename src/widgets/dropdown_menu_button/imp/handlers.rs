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

  pub fn handle_popover_show(state: &MenuState) {
    state.sub_menu_stack.borrow_mut().clear();
    state.sub_menu_breadcrumbs.borrow_mut().clear();
  }

  pub fn handle_menu_item_click(menu_item_id: &str, is_toggable: bool, popover: &OnceCell<Popover>, obj_weak: &WeakRef<DropdownMenuButton>) {
    if let Some(obj) = obj_weak.upgrade() {
      if let Some(popover) = popover.get() {
        popover.popdown();
      }
      
      let imp = obj.imp();
      if is_toggable {
        let new_state = {
          let mut items = imp.state.menu_items.borrow_mut();

          if let Some(item) = items.iter_mut().find(|i| i.id == menu_item_id) {
            item.is_toggled = !item.is_toggled;
            item.is_toggled

          } else {
            false
          }
        };
        imp.rebuild_menu();
        imp.emit_item_toggled(&menu_item_id, new_state);

      } else {
        imp.emit_item_selected(&menu_item_id);
      }
    }

  }

  pub fn handle_submenu_click(submenu_items: Vec<MenuItem>, menu_label: String, obj_weak: WeakRef<DropdownMenuButton>) {
    if let Some(obj) = obj_weak.upgrade() {
      let imp = obj.imp();
      
      let current_menu = imp.state.menu_items.borrow().clone();
      imp.state.sub_menu_stack.borrow_mut().push(current_menu);
      imp.state.sub_menu_breadcrumbs.borrow_mut().push(menu_label.clone());
      
      *imp.state.menu_items.borrow_mut() = submenu_items.clone();
      
      imp.rebuild_menu();
    }
  }

  pub fn handle_back_button_click(obj_weak: WeakRef<DropdownMenuButton>) {
    if let Some(obj) = obj_weak.upgrade() {
      let imp = obj.imp();
      
      let should_rebuild = {
        let mut stack = imp.state.sub_menu_stack.borrow_mut();
        if let Some(previous_menu) = stack.pop() {
          drop(stack); 
          
          imp.state.sub_menu_breadcrumbs.borrow_mut().pop();
          *imp.state.menu_items.borrow_mut() = previous_menu;
          true
        } else {
          false
        }
      };
      
      if should_rebuild {
        imp.rebuild_menu();
      }
    }
  }

  pub fn setup_dropdown_menu_button_handlers(&self){
    let obj_weak = self.obj().downgrade();

    if let Some(button) = self.dropdown_menu_button.get() {
      button.connect_clicked(move |_| {
        if let Some(obj) = obj_weak.upgrade() {
          obj.imp().handle_dropdown_menu_button_click();
        }
      });
    }

    if let Some(popover) = self.popover.get() {
      let state = self.state.clone();
      popover.connect_show(move |_| {
        Self::handle_popover_show(&state);
      });
    }
  }

  pub fn setup_menu_item_handlers(&self, menu_item_container: &gtk::Box, menu_item: &MenuItem) {
    let event_controller = GestureClick::new();
    menu_item_container.add_controller(event_controller.clone());

    if let Some(submenu_items) = menu_item.submenu.clone() {
      let item_label = menu_item.label.clone();
      let obj_weak = self.obj().downgrade();

      event_controller.connect_released(move |_, _, _, _| {
        Self::handle_submenu_click(submenu_items.clone(), item_label.clone(), obj_weak.clone());
      });

    } else {
      let menu_item_id = menu_item.id.clone();
      let is_toggable = menu_item.is_toggleable;
      let popover = self.popover.clone();
      let obj_weak = self.obj().downgrade();

      event_controller.connect_released(move |_, _, _, _| {
        Self::handle_menu_item_click(&menu_item_id, is_toggable, &popover, &obj_weak);
      });
    }
  }

  pub fn setup_back_button_handler(&self, menu_item_container: &Box) {
    let obj_weak = self.obj().downgrade();
    let event_controller = GestureClick::new();

    menu_item_container.add_controller(event_controller.clone());

    event_controller.connect_released(move |_, _, _, _| {
      Self::handle_back_button_click(obj_weak.clone());
    });
  }  
}
