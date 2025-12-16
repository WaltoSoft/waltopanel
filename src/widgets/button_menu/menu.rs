use gtk::prelude::{BoxExt, GridExt};
use gtk::{Box, Grid, Label, Widget};
use gtk::{Align, Orientation, prelude::PopoverExt};

use super::ButtonMenuPrivate;
use crate::helpers::ui_helpers;
use crate::models::{MenuItemModel};
use crate::types::TypedListStore;

impl ButtonMenuPrivate {
  pub fn reset_menu(&self) {
    if let Some(menu_data) = self.menu_data.get() {
      *self.current_menu.borrow_mut() = menu_data.clone();
    }

    self.sub_menu_stack.borrow_mut().clear();
    self.breadcrumbs.borrow_mut().clear();

    self.rebuild_menu();
  }

  pub fn rebuild_menu(&self) {
    if let Some(popover) = self.popover.get() {
      if let Some(_child) = popover.child() {
        popover.set_child(Widget::NONE);
      }

      if self.is_current_menu_empty() {
        return;
      }

      let menu_box = self.create_dropdown_menu();
      popover.set_child(Some(&menu_box));
    }
  }

  pub fn show_submenu(&self, menu_item: &MenuItemModel) {
    let submenu_items = menu_item.submenu();
    let sub_menu_label = menu_item.text().clone();
    let current_menu = self.current_menu.borrow().clone();

    self.sub_menu_stack.borrow_mut().push(current_menu);
    self.breadcrumbs.borrow_mut().push(sub_menu_label);    
    *self.current_menu.borrow_mut() = submenu_items;

    self.rebuild_menu();
  }

  fn is_submenu(&self) -> bool {
    ! self.sub_menu_stack.borrow().is_empty()
  }

  fn current_menu(&self) -> TypedListStore<MenuItemModel> {
    self.current_menu.borrow().clone()
  }
  
  fn is_current_menu_empty(&self) -> bool {
    self.current_menu().is_empty()
  }    //attach button handler


  pub fn get_back_button_label(&self) -> String {
    self.breadcrumbs.borrow().last().cloned().unwrap_or_else(|| "Back".to_string())
  }
 
  fn menu_has_toggable_items(&self) -> bool {
    self.current_menu().iter().any(|item| item.allow_toggle())
  }

  fn menu_has_items_with_icons(&self) -> bool {
    self.current_menu().iter().any(|item| item.icon_name().is_some())
  }

  fn create_dropdown_menu(&self) -> Box {
    let menu_box = ui_helpers::create_styled_box(Orientation::Vertical, 0, vec!["dropdown-menu".to_string()]);

    if self.is_submenu() {
      let back_item = self.create_back_button();
      let separator = ui_helpers::create_menu_separator();

      menu_box.append(&back_item);
      menu_box.append(&separator);
    }

    for menu_item in &self.current_menu() {
      if menu_item.is_separator() {
        let separator = ui_helpers::create_menu_separator();
        menu_box.append(&separator);
      }
      else {
        let menu_item = self.create_menu_item(&menu_item);
        menu_box.append(&menu_item);
      }
    }

    menu_box
  }

  fn create_menu_item(&self, menu_item: &MenuItemModel) -> Box {
    let menu_item_box = ui_helpers::create_styled_box(Orientation::Horizontal, 0, vec!["dropdown-item".to_string()]);
    let content_grid = self.create_menu_item_grid(menu_item);

    self.attach_menu_item_handlers(&menu_item_box, menu_item);

    menu_item_box.append(&content_grid);
    menu_item_box
  }

  fn create_back_button(&self) -> Box {
    let menu_item_box = ui_helpers::create_styled_box(gtk::Orientation::Horizontal, 0, vec!["dropdown-item".to_string()]);
    let content_grid = Self::create_back_button_grid(&self.get_back_button_label(),16);

    //attach button handler

    menu_item_box.append(&content_grid);
    menu_item_box
  }


  fn create_menu_item_grid(&self, menu_item: &MenuItemModel) -> Grid {
    let mut col = 0;
    let icon_size = 16;
    let content_grid = Grid::builder().column_spacing(12).build();
    let icon_widget = ui_helpers::create_icon_widget(menu_item.icon_name(), icon_size);

    let toggled_icon = if self.menu_has_toggable_items() && menu_item.toggled() {
      Some("object-select-symbolic".to_string())
    } else {
      None
    };

    if self.menu_has_toggable_items() {
      let toggled_icon_widget = ui_helpers::create_icon_widget(toggled_icon, 16);  

      content_grid.attach(&toggled_icon_widget, col, 0, 1, 1);
      col += 1;
    }

    if self.menu_has_items_with_icons() {
      content_grid.attach(&icon_widget, col, 0, 1, 1);
      col += 1;
    }

    let label = Label::builder()
      .label(menu_item.text())
      .halign(Align::Start)
      .hexpand(true)
      .build();    

    content_grid.attach(&label, col, 0, 1, 1);
    col += 1;

    if menu_item.has_submenu() {
      let sub_menu_icon = Some("go-next-symbolic".to_string());
      let arrow_icon_widget = ui_helpers::create_icon_widget(sub_menu_icon,icon_size);
      content_grid.attach(&arrow_icon_widget, col, 0, 1,1);
    } 

    content_grid
  }

  pub fn create_back_button_grid(label: &str, icon_size: i32) -> Grid {
    let content_grid = Grid::builder().column_spacing(12).build();

    let back_icon_widget = ui_helpers::create_icon_widget(Some("go-previous-symbolic".to_string()), icon_size);
    content_grid.attach(&back_icon_widget, 0, 0, 1, 1);

    let label = Label::builder()
      .label(label)
      .halign(Align::Start)
      .hexpand(true)
      .build();
    content_grid.attach(&label, 1, 0, 1, 1);

    content_grid
  }  

}

