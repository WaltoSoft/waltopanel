use gtk::gdk::{Key, prelude::{DisplayExt, MonitorExt, SurfaceExt}};
use gtk::gio::prelude::ListModelExt;
use gtk::glib::{object::Cast, Propagation};
use gtk::{Align, Box, EventControllerKey, EventControllerMotion, ListBox, ListBoxRow, Orientation, Popover, PositionType, SelectionMode, Widget};
use gtk::prelude::{BoxExt, ListBoxRowExt, NativeExt, PopoverExt, WidgetExt};
use std::{cell::{OnceCell, RefCell}, rc::Rc};
use std::boxed::Box as StdBox;

use crate::helpers::ui_helpers;
use crate::models::MenuItemModel;
use crate::traits::CompositeWidget;
use crate::types::TypedListStore;
use super::menu_item::MenuItem;
use super::back_button::BackButton;

#[derive(Clone)]
pub struct Menu {
  popover: Popover,
  menu_data: Rc<OnceCell<TypedListStore<MenuItemModel>>>,
  current_menu: Rc<RefCell<TypedListStore<MenuItemModel>>>,
  menu_stack: Rc<RefCell<Vec<TypedListStore<MenuItemModel>>>>,
  breadcrumbs: Rc<RefCell<Vec<String>>>,
  before_menu_show_callback: Rc<OnceCell<StdBox<dyn Fn()>>>,
  menu_clicked_callback: Rc<OnceCell<StdBox<dyn Fn(&MenuItemModel)>>>
}

impl Menu {
  pub fn new() -> Self {
    let popover = Popover::builder()
      .autohide(true)
      .has_arrow(false)
      .position(PositionType::Bottom)
      .can_focus(true)
      .focusable(true)
      .build();

    popover.connect_show(move |popover| {
      // When the popover is shown, set the panel button to active state
      // Parent chain: Popover -> Button -> PanelButton
      if let Some(panel_button) = popover.parent().and_then(|b| b.parent()) {
        panel_button.set_state_flags(gtk::StateFlags::ACTIVE, false);
      }
    });

    popover.connect_hide(move |popover| {
      // When the popover is hidden, unset the panel button active state
      if let Some(panel_button) = popover.parent().and_then(|b| b.parent()) {
        panel_button.unset_state_flags(gtk::StateFlags::ACTIVE);
      }
    });

    let key_controller = EventControllerKey::new();
    popover.add_controller(key_controller.clone());

    let menu = Self {
      popover,
      menu_data: Rc::new(OnceCell::new()),
      current_menu: Rc::new(RefCell::new(TypedListStore::new())),
      menu_stack: Rc::new(RefCell::new(Vec::new())),
      breadcrumbs: Rc::new(RefCell::new(Vec::new())),
      before_menu_show_callback: Rc::new(OnceCell::new()),
      menu_clicked_callback: Rc::new(OnceCell::new()),
    };


    menu
  }

  pub fn set_menu(&self, menu: TypedListStore<MenuItemModel>) {
    self.menu_data.set(menu).expect("Menu can only be set once");
  }

  pub fn toggle_visibility(&self) {
    if self.popover.is_visible() {
      self.popover.popdown();
    } else {
      let self_clone = self.clone();

      if let Some(callback) = self_clone.before_menu_show_callback.get() {
        callback();
      }

      self.update_popover_alignment();
      self.reset_menu();
      self.popover.popup();
    }
  }

  pub fn hide_menu(&self) {
    if self.popover.is_visible() {
      self.popover.popdown();
    }
  }

  fn current_menu(&self) -> TypedListStore<MenuItemModel> {
    self.current_menu.borrow().clone()
  }

  fn is_submenu(&self) -> bool {
    ! self.menu_stack.borrow().is_empty()
  }
 
  fn menu_has_toggable_items(&self) -> bool {
    self.current_menu().iter().any(|item| item.allow_toggle())
  }

  fn menu_has_icons(&self) -> bool {
    self.current_menu().iter().any(|item| item.icon_name().is_some())
  }

  pub fn get_back_button_label(&self) -> String {
    self.breadcrumbs.borrow().last().cloned().unwrap_or_else(|| "Back".to_string())
  }
 
  fn reset_menu(&self) {
    if let Some(menu_data) = self.menu_data.get() {
      *self.current_menu.borrow_mut() = menu_data.clone();
    }

    self.menu_stack.borrow_mut().clear();
    self.breadcrumbs.borrow_mut().clear();

    self.rebuild_menu();
  }

  fn rebuild_menu(&self) {
    if let Some(_) = self.popover.child() {
      self.popover.set_child(Widget::NONE);
    }

    if self.current_menu().is_empty() {
      return;
    }

    let (menu_box, list_box, _menu_items) = self.create_menu();
    self.popover.set_child(Some(&menu_box));
    
    // Focus the list box and select the first row for keyboard navigation
    list_box.grab_focus();
    if let Some(first_row) = list_box.row_at_index(0) {
      list_box.select_row(Some(&first_row));
      first_row.grab_focus();
    }
  }

  fn create_menu(&self) -> (Box, ListBox, Vec<MenuItem>) {
    let menu_box = ui_helpers::create_styled_box(Orientation::Vertical, 0, vec!["menu".to_string()]);
    
    let list_box = ListBox::new();
    list_box.set_selection_mode(SelectionMode::Browse); // Highlights focused row
    list_box.add_css_class("menu-list");
    
    let mut menu_items = Vec::new();

    if self.is_submenu() {
      let back_button = BackButton::new(self.get_back_button_label());
      let menu_clone = self.clone();

      back_button.connect_clicked(move || {
        menu_clone.show_submenu_parent();
      });

      let back_row = ListBoxRow::new();
      back_row.set_child(Some(&back_button.widget()));
      back_row.add_css_class("menu-back-row");
      back_row.add_css_class("has-separator-after");  // Add separator after back button
      
      // Focus and select row on hover
      let motion = EventControllerMotion::new();
      let back_row_clone = back_row.clone();
      let list_box_clone = list_box.clone();
      motion.connect_enter(move |_, _, _| {
        list_box_clone.select_row(Some(&back_row_clone));
        back_row_clone.grab_focus();
      });
      back_row.add_controller(motion);
      
      // Connect row activation (Enter key) for back button
      let menu_clone = self.clone();
      back_row.connect_activate(move |_row| {
        menu_clone.show_submenu_parent();
      });
      
      list_box.append(&back_row);
    }  

    for menu_item in &self.current_menu() {
      if menu_item.is_separator() {
        // Instead of adding a separator row, mark the previous row with a CSS class
        // The visual separator will be created with CSS border-bottom
        if let Some(last_row) = list_box.last_child() {
          if let Some(row) = last_row.downcast_ref::<ListBoxRow>() {
            row.add_css_class("has-separator-after");
          }
        }
      }
      else {
        let model_clone = menu_item.clone();
        let menu_item_widget = MenuItem::new(menu_item, self.menu_has_toggable_items(), self.menu_has_icons(), self.is_submenu());
        let menu_clone = self.clone();

        menu_item_widget.connect_clicked(move |model|{
          if model.has_submenu() {
            menu_clone.show_submenu(model);
          } else {
            if let Some(callback) = menu_clone.menu_clicked_callback.get() {
              callback(model);
            }
            menu_clone.hide_menu();
          }
        });

        let row = ListBoxRow::new();
        row.set_child(Some(&menu_item_widget.widget()));
        
        // Focus and select row on hover
        let motion = EventControllerMotion::new();
        let row_clone = row.clone();
        let list_box_clone = list_box.clone();
        motion.connect_enter(move |_, _, _| {
          list_box_clone.select_row(Some(&row_clone));
          row_clone.grab_focus();
        });
        row.add_controller(motion);
        
        // Connect row activation (Enter key)
        let menu_clone = self.clone();
        row.connect_activate(move |_row| {
          if model_clone.has_submenu() {
            menu_clone.show_submenu(&model_clone);
          } else {
            if let Some(callback) = menu_clone.menu_clicked_callback.get() {
              callback(&model_clone);
            }
            menu_clone.hide_menu();
          }
        });
        
        list_box.append(&row);
        menu_items.push(menu_item_widget);
      }
    }    
    
    menu_box.append(&list_box);
    (menu_box, list_box, menu_items)
  }

  fn show_submenu(&self, menu_item: &MenuItemModel) {
    let submenu_items = menu_item.submenu();
    let sub_menu_label = menu_item.text().clone();
    let current_menu = self.current_menu.borrow().clone();

    self.menu_stack.borrow_mut().push(current_menu);
    self.breadcrumbs.borrow_mut().push(sub_menu_label);    
    *self.current_menu.borrow_mut() = submenu_items;

    self.rebuild_menu();
  }

  fn show_submenu_parent(&self) {
    let mut stack  = self.menu_stack.borrow_mut();

    if let Some(previous_menu) = stack.pop() {
      drop(stack);

      self.breadcrumbs.borrow_mut().pop();
      *self.current_menu.borrow_mut() = previous_menu;

      self.rebuild_menu();
    }
  }

  pub fn update_popover_alignment(&self) {
    let popover = &self.popover;
    let Some(button_menu_box) = popover.parent() else {
      return;
    };

    if let Some(surface) = button_menu_box.native().and_then(|n| n.surface()) {
      let display = surface.display();
      
      let monitor = display
        .monitor_at_surface(&surface)
        .or_else(|| {
          display.monitors().item(0)?.downcast().ok()
        });
      
      let Some(monitor) = monitor else {
        return;
      };
      
      let monitor_geometry = monitor.geometry();

      let (button_x, _) = button_menu_box
        .root()
        .and_then(|root| button_menu_box.translate_coordinates(&root, 0.0, 0.0))
        .unwrap_or((0.0, 0.0));

      let button_width = button_menu_box.allocated_width();
      let menu_width = 200;  //TODO: Magic number needs to be fixed.
      let space_right = monitor_geometry.width() - (button_x as i32 + button_width);

      if space_right >= menu_width {
        popover.set_halign(Align::Start);
      } else {
        popover.set_halign(Align::End);
      }
    }
  }

  pub fn connect_menu_clicked<F>(&self, callback: F)
  where
    F: Fn(&MenuItemModel) + 'static,
  {
    self.menu_clicked_callback.set(StdBox::new(callback)).ok().expect("Menu clicked callback can only be set once");
  }
}

impl CompositeWidget for Menu {
  fn widget(&self) -> Widget {
    self.popover.clone().upcast()
  }
}

impl std::fmt::Debug for Menu {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Menu")
      .field("popover", &self.popover)
      .field("menu_data", &self.menu_data)
      .field("current_menu", &self.current_menu)
      .field("menu_stack", &self.menu_stack)
      .field("breadcrumbs", &self.breadcrumbs)
      .field("menu_clicked_callback", &"<callback>")
      .finish()
  }
}