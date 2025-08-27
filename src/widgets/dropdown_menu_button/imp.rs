use gtk::{glib::{self, subclass::Signal}, prelude::*, subclass::prelude::*, Align, GestureClick};
use gtk::{BinLayout, Box, Button, Grid, Image, Label, Popover, Widget};
use gtk::{Orientation, PositionType};
use std::cell::{OnceCell, RefCell};
use std::sync::OnceLock;

use super::state::MenuState;
use crate::{models::MenuItem, widgets::dropdown_menu_button::styling::DropdownStyling};

thread_local! {
  static DROPDOWN_INSTANCES: RefCell<Vec<Popover>> = RefCell::new(Vec::new());
}

pub struct DropdownMenuButtonPrivate {
  button: OnceCell<Button>,
  popover: OnceCell<Popover>,
  state: MenuState,
}

impl Default for DropdownMenuButtonPrivate {
  fn default() -> Self {
    Self {
      button: OnceCell::new(),
      popover: OnceCell::new(),
      state: MenuState::new(),
    }
  }
}

#[glib::object_subclass]
impl ObjectSubclass for DropdownMenuButtonPrivate {
  const NAME: &'static str = "DropdownMenuButton";
  type Type = super::DropdownMenuButton;
  type ParentType = Widget;
  
  fn class_init(klass: &mut Self::Class) {
    klass.set_layout_manager_type::<BinLayout>();
  }
}

impl ObjectImpl for DropdownMenuButtonPrivate {
  fn constructed(&self) {
    self.parent_constructed();
    
    let obj = self.obj();
    let button = Button::new();

    button.set_parent(&*obj);

    let popover = Popover::builder()
      .autohide(false)
      .has_arrow(false)
      .position(PositionType::Bottom)
      .can_focus(true)
      .focusable(true)
      .build();
    
    popover.set_parent(&button);
    Self::register_popover_instance(&popover);

    self.button.set(button.clone()).expect("Button should only be set once during construction");
    self.popover.set(popover.clone()).expect("Popover should only be set once during construction");

    self.setup_button_behavior();
    self.setup_popover_handlers();
  }
    
  fn dispose(&self) {
    if let Some(button) = self.button.get() {
      button.unparent();
    }
    if let Some(popover) = self.popover.get() {
      popover.unparent();
    }
  }
  
  fn signals() -> &'static [Signal] {
    static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
    SIGNALS.get_or_init(|| {
      vec![
        Signal::builder("item-selected")
          .param_types([String::static_type()])
          .build(),
        Signal::builder("item-toggled")
          .param_types([String::static_type(), bool::static_type()])
          .build(),
      ]
    })
  }
}


//-------------------------------------------------------------------------------------------------
// Initialization methods
//-------------------------------------------------------------------------------------------------
impl DropdownMenuButtonPrivate {
  pub fn set_button_text(&self, text: &str) {
    if let Some(button) = self.button.get() {
      button.set_label(text);
    }
  }

  pub fn set_button_icon(&self, icon_name: &str) {
    if let Some(button) = self.button.get() {
      let icon = Image::from_icon_name(icon_name);
      button.set_child(Some(&icon));
    }
  }

  pub fn set_button_icon_and_text(&self, icon_name: &str, text: &str) {
    if let Some(button) = self.button.get() {
      let container = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();

      let icon = Image::from_icon_name(icon_name);
      let label = Label::new(Some(text));

      container.append(&icon);
      container.append(&label);

      button.set_child(Some(&container));
    }
  }

  pub fn set_menu_items(&self, items: Vec<MenuItem>) {
    *self.state.menu_items.borrow_mut() = items;
    self.rebuild_menu();
  }

  pub fn set_item_toggled(&self, item_id: &str, toggled: bool) {
    let mut items = self.state.menu_items.borrow_mut();
    
    if let Some(item) = items.iter_mut().find(|i| i.id == item_id) {
      if item.is_toggleable {
        item.is_toggled = toggled;
      }
    }

    drop(items);
    self.rebuild_menu();
  }
}


impl DropdownMenuButtonPrivate {
  fn emit_item_selected(&self, item_id: &str) {
    self.obj().emit_by_name::<()>("item-selected", &[&item_id]);
  }

  fn emit_item_toggled(&self, item_id: &str, toggled_state: bool) {
    self.obj().emit_by_name::<()>("item-toggled", &[&item_id, &toggled_state]);
  }


  fn setup_button_behavior(&self) {
    if let (Some(button), Some(popover)) = (self.button.get(), self.popover.get()) {
      let popover_click = popover.clone();
      let button_click = button.clone();
      button.connect_clicked(move |_| {
        if popover_click.is_visible() {
          popover_click.popdown();
        } else {
          Self::close_all_other_dropdowns(&popover_click);
          Self::update_popover_alignment(&popover_click, &button_click);
          popover_click.popup();
        }
      });
    }
  }

  fn setup_popover_handlers(&self) {
    if let Some(popover) = self.popover.get() {
      popover.connect_show({
        let state = &self.state;
        let focused_index = state.focused_item_index.clone();
        let menu_stack = state.sub_menu_stack.clone();
        let breadcrumbs = state.sub_menu_breadcrumbs.clone();
        let menu_boxes = state.menu_boxes.clone();
        
        move |_| {
          *focused_index.borrow_mut() = None;
          menu_stack.borrow_mut().clear();
          breadcrumbs.borrow_mut().clear();
          
          // Clear visual focus
          for container in menu_boxes.borrow().iter() {
            container.remove_css_class("focused");
          }
        }
      });
    }
  }

  pub fn rebuild_menu(&self) {
    if let Some(popover) = self.popover.get() {
      if let Some(_child) = popover.child() {
        popover.set_child(Widget::NONE);
      }

      let items = self.state.menu_items.borrow().clone();
      let has_stack = !self.state.sub_menu_stack.borrow().is_empty();
      let breadcrumbs = self.state.sub_menu_breadcrumbs.borrow().clone();
      
      for item in items.iter() {
        println!("  - {}", item.label);
      }
      
      if items.is_empty() {
        return;
      }

      let menu_box = self.create_menu_container_with_state(&items, has_stack, &breadcrumbs);
      popover.set_child(Some(&menu_box));
    }
  }

  fn create_menu_container_with_state(&self, items: &[MenuItem], has_stack: bool, breadcrumbs: &[String]) -> Widget {
    let menu_box = DropdownStyling::create_styled_menu_container();
    let mut containers = Vec::new();

    if has_stack {
      let back_item = self.create_back_button_with_breadcrumbs(breadcrumbs);
      if let Some(container) = back_item.downcast_ref::<Box>() {
        containers.push(container.clone());
      }
      menu_box.append(&back_item);

      let separator = DropdownStyling::create_styled_separator();
      menu_box.append(&separator);
    }

    for item in items {
      if item.is_separator {
        let separator = DropdownStyling::create_styled_separator();
        menu_box.append(&separator);
      } else {
        let menu_item = self.create_menu_item(item);
        if let Some(container) = menu_item.downcast_ref::<Box>() {
          containers.push(container.clone());
        }
        menu_box.append(&menu_item);
      }
    }

    *self.state.menu_boxes.borrow_mut() = containers;
    menu_box.upcast()
  }

  fn create_menu_item(&self, item: &MenuItem) -> Widget {
    let item_container = DropdownStyling::create_styled_menu_item();

    DropdownStyling::set_item_toggled(&item_container, item.is_toggled);

    self.setup_item_interactions(&item_container, item);

    let content_grid = Grid::builder().column_spacing(12).build();
    let mut col = 0;

    let icon_widget = Self::create_menu_icon(item);
    content_grid.attach(&icon_widget, col, 0, 1, 1);
    col += 1;

    let label = Label::builder()
      .label(&item.label)
      .halign(Align::Start)
      .hexpand(true)
      .build();
    content_grid.attach(&label, col, 0, 1, 1);
    col += 1;

    let arrow_widget = Self::create_submenu_indicator(item.submenu.is_some());
    content_grid.attach(&arrow_widget, col, 0, 1, 1);

    item_container.append(&content_grid);
    item_container.upcast()
  }

  fn create_back_button_with_breadcrumbs(&self, breadcrumbs: &[String]) -> Widget {
    let item_container = DropdownStyling::create_styled_menu_item();

    let content_grid = Grid::builder().column_spacing(12).build();
    let mut col = 0;

    let back_icon = Self::create_back_icon();
    content_grid.attach(&back_icon, col, 0, 1, 1);
    col += 1;

    let back_label = Self::get_back_button_label(breadcrumbs);
    let label = Label::builder()
      .label(&back_label)
      .halign(Align::Start)
      .hexpand(true)
      .build();
    content_grid.attach(&label, col, 0, 1, 1);
    col += 1;

    let placeholder = Box::builder().width_request(16).build();
    content_grid.attach(&placeholder, col, 0, 1, 1);

    item_container.append(&content_grid);

    self.setup_back_button_interaction(&item_container);

    item_container.upcast()
  }

  fn setup_item_interactions(&self, item_container: &Box, item: &MenuItem) {
    let event_controller = GestureClick::new();
    item_container.add_controller(event_controller.clone());

    if let Some(submenu_items) = item.submenu.clone() {
      let item_label = item.label.clone();
      let obj_weak = self.obj().downgrade();

      event_controller.connect_released(move |_, _, _, _| {
        if let Some(obj) = obj_weak.upgrade() {
          let imp = obj.imp();
          
          let current_menu = imp.state.menu_items.borrow().clone();
          imp.state.sub_menu_stack.borrow_mut().push(current_menu);
          imp.state.sub_menu_breadcrumbs.borrow_mut().push(item_label.clone());
          
          *imp.state.menu_items.borrow_mut() = submenu_items.clone();
          
          imp.rebuild_menu();
        }
      });
    } else {
      let item_id = item.id.clone();
      let is_toggleable = item.is_toggleable;
      let popover = self.popover.clone();
      let obj_weak = self.obj().downgrade();

      event_controller.connect_released(move |_, _, _, _| {
        if let Some(obj) = obj_weak.upgrade() {
          if let Some(popover) = popover.get() {
            popover.popdown();
          }
          
          let imp = obj.imp();
          if is_toggleable {
            // Toggle the item state
            let new_state = {
              let mut items = imp.state.menu_items.borrow_mut();
              if let Some(item) = items.iter_mut().find(|i| i.id == item_id) {
                item.is_toggled = !item.is_toggled;
                item.is_toggled
              } else {
                false
              }
            };
            imp.rebuild_menu();
            imp.emit_item_toggled(&item_id, new_state);
          } else {
            imp.emit_item_selected(&item_id);
          }
        }
      });
    }
  }

  fn setup_back_button_interaction(&self, item_container: &Box) {
    let event_controller = GestureClick::new();
    item_container.add_controller(event_controller.clone());

    let obj_weak = self.obj().downgrade();

    event_controller.connect_released(move |_, _, _, _| {
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
    });
  }

  // Utility methods (moved from utils.rs)
  fn update_popover_alignment(popover: &Popover, button: &Button) {
    if let Some(surface) = button.native().and_then(|n| n.surface()) {
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

      let (button_x, _) = button
        .root()
        .and_then(|root| button.translate_coordinates(&root, 0.0, 0.0))
        .unwrap_or((0.0, 0.0));

      let button_width = button.allocated_width();
      let menu_width = 200;
      let space_right = monitor_geometry.width() - (button_x as i32 + button_width);

      if space_right >= menu_width {
        popover.set_halign(Align::Start);
      } else {
        popover.set_halign(Align::End);
      }
    }
  }

  fn create_menu_icon(item: &MenuItem) -> Widget {
    if item.is_toggled {
      let checkmark = Image::from_icon_name("object-select-symbolic");
      checkmark.set_pixel_size(16);
      checkmark.upcast()
    } else if let Some(icon_name) = &item.icon {
      let icon = Image::from_icon_name(icon_name);
      icon.set_pixel_size(16);
      icon.upcast()
    } else {
      let placeholder = Box::builder()
        .width_request(16)
        .height_request(16)
        .build();
      placeholder.upcast()
    }
  }

  fn create_submenu_indicator(has_submenu: bool) -> Widget {
    if has_submenu {
      let arrow = Image::from_icon_name("go-next-symbolic");
      arrow.set_pixel_size(12);
      arrow.set_halign(Align::End);
      arrow.upcast()
    } else {
      let placeholder = Box::builder().width_request(16).build();
      placeholder.upcast()
    }
  }

  fn create_back_icon() -> Image {
    let icon = Image::from_icon_name("go-previous-symbolic");
    icon.set_pixel_size(16);
    icon
  }

  fn get_back_button_label(breadcrumbs: &[String]) -> String {
    breadcrumbs.last().cloned().unwrap_or_else(|| "Back".to_string())
  }

  fn register_popover_instance(popover: &Popover) {
    DROPDOWN_INSTANCES.with(|instances| {
      instances.borrow_mut().push(popover.clone());
    });

    let popover_for_cleanup = popover.clone();
    popover.connect_destroy(move |_| {
      DROPDOWN_INSTANCES.with(|instances| {
        let mut instances = instances.borrow_mut();
        instances.retain(|p| p != &popover_for_cleanup);
      });
    });
  }

  fn close_all_other_dropdowns(current_popover: &Popover) {
    DROPDOWN_INSTANCES.with(|instances| {
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

impl WidgetImpl for DropdownMenuButtonPrivate {
  fn measure(&self, orientation: Orientation, for_size: i32) -> (i32, i32, i32, i32) {
    if let Some(button) = self.button.get() {
      button.measure(orientation, for_size)
    } else {
      (0, 0, -1, -1)
    }
  }
  
  fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
    if let Some(button) = self.button.get() {
      button.allocate(width, height, baseline, None);
    }
  }
}