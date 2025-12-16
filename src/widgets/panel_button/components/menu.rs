use gtk::gdk::prelude::{DisplayExt, MonitorExt, SurfaceExt};
use gtk::gio::prelude::ListModelExt;
use gtk::glib::object::Cast;
use gtk::{Align, Box, Orientation, Popover, PositionType, Widget};
use gtk::prelude::{BoxExt, NativeExt, PopoverExt, WidgetExt};
use std::cell::{OnceCell, RefCell};
use std::rc::Rc;

use crate::helpers::ui_helpers;
use crate::models::MenuItemModel;
use crate::traits::CompositeWidget;
use crate::types::TypedListStore;
use super::menu_item::MenuItem;
use super::back_button::BackButton;

#[derive(Debug, Clone)]
pub struct Menu {
  popover: Popover,
  menu_data: Rc<OnceCell<TypedListStore<MenuItemModel>>>,
  current_menu: Rc<RefCell<TypedListStore<MenuItemModel>>>,
  menu_stack: Rc<RefCell<Vec<TypedListStore<MenuItemModel>>>>,
  breadcrumbs: Rc<RefCell<Vec<String>>>,
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

    Self {
      popover,
      menu_data: Rc::new(OnceCell::new()),
      current_menu: Rc::new(RefCell::new(TypedListStore::new())),
      menu_stack: Rc::new(RefCell::new(Vec::new())),
      breadcrumbs: Rc::new(RefCell::new(Vec::new())),
    }
  }

  pub fn set_menu(&self, menu: TypedListStore<MenuItemModel>) {
    self.menu_data.set(menu).expect("Menu can only be set once");
  }

  pub fn toggle_visibility(&self) {
    if self.popover.is_visible() {
      self.popover.popdown();
    } else {

      self.update_popover_alignment();
      //Self::close_other_button_menus(popover);
      self.reset_menu();
      self.popover.popup();
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

    let menu_box = self.create_menu();
    self.popover.set_child(Some(&menu_box));
  }

  fn create_menu(&self) -> Box {
    let menu_box = ui_helpers::create_styled_box(Orientation::Vertical, 0, vec!["dropdown-menu".to_string()]);
    
    if self.is_submenu() {
      let back_button = BackButton::new("".to_string());
      let separator = ui_helpers::create_menu_separator();

      menu_box.append(&back_button.widget());
      menu_box.append(&separator);
    }

    for menu_item in &self.current_menu() {
      if menu_item.is_separator() {
        let separator = ui_helpers::create_menu_separator();
        menu_box.append(&separator);
      }
      else {
        let menu_item = MenuItem::new(menu_item, self.menu_has_toggable_items(), self.menu_has_icons());
        menu_box.append(&menu_item.widget());
      }
    }    
    
    menu_box
  }

  pub fn update_popover_alignment(&self) {
    let popover = &self.popover;
    let Some(button_menu_box) = popover.parent() else {
      return;
    };

    // Ensure the button is realized and has a root before proceeding
    if !button_menu_box.is_realized() || button_menu_box.root().is_none() {
      return;
    }

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
        .and_then(|root| {
          // Use compute_point instead of translate_coordinates to avoid the assertion
          button_menu_box.compute_point(&root, &gtk::graphene::Point::new(0.0, 0.0))
            .map(|point| (point.x() as f64, point.y() as f64))
        })
        .unwrap_or((0.0, 0.0));

      let button_width = button_menu_box.allocated_width();
      let menu_width = 200;
      let space_right = monitor_geometry.width() - (button_x as i32 + button_width);

      if space_right >= menu_width {
        popover.set_halign(Align::Start);
      } else {
        popover.set_halign(Align::End);
      }
    }
  }
}

impl CompositeWidget for Menu {
  fn widget(&self) -> Widget {
    self.popover.clone().upcast()
  }
}
