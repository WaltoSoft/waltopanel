use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::OnceCell;

use super::state::MenuState;
use crate::models::MenuItem;

pub struct DropdownMenuButtonPrivate {
  pub button: OnceCell<gtk::Button>,
  pub popover: OnceCell<gtk::Popover>,
  pub state: MenuState,
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
  type ParentType = gtk::Widget;
  
  fn class_init(klass: &mut Self::Class) {
    klass.set_layout_manager_type::<gtk::BinLayout>();
  }
}

impl ObjectImpl for DropdownMenuButtonPrivate {
  fn constructed(&self) {
    self.parent_constructed();
    
    let obj = self.obj();
    
    let button = gtk::Button::new();
    button.set_parent(&*obj);
    self.button.set(button.clone()).unwrap();
    
    let popover = gtk::Popover::builder()
      .autohide(false)
      .has_arrow(false)
      .position(gtk::PositionType::Bottom)
      .can_focus(true)
      .focusable(true)
      .build();
    
    popover.set_parent(&button);
    self.popover.set(popover.clone()).unwrap();
    
    super::utils::DropdownUtils::register_popover_instance(&popover);
    
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
  
  fn signals() -> &'static [glib::subclass::Signal] {
    static SIGNALS: std::sync::OnceLock<Vec<glib::subclass::Signal>> = std::sync::OnceLock::new();
    SIGNALS.get_or_init(|| {
      vec![
        glib::subclass::Signal::builder("item-selected")
          .param_types([String::static_type()])
          .build(),
        glib::subclass::Signal::builder("item-toggled")
          .param_types([String::static_type(), bool::static_type()])
          .build(),
      ]
    })
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
          super::utils::DropdownUtils::close_all_other_dropdowns(&popover_click);
          super::utils::DropdownUtils::update_popover_alignment(&popover_click, &button_click);
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
        popover.set_child(gtk::Widget::NONE);
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

  fn create_menu_container_with_state(&self, items: &[MenuItem], has_stack: bool, breadcrumbs: &[String]) -> gtk::Widget {
    let menu_box = super::styling::DropdownStyling::create_styled_menu_container();
    let mut containers = Vec::new();

    if has_stack {
      let back_item = self.create_back_button_with_breadcrumbs(breadcrumbs);
      if let Some(container) = back_item.downcast_ref::<gtk::Box>() {
        containers.push(container.clone());
      }
      menu_box.append(&back_item);

      let separator = super::styling::DropdownStyling::create_styled_separator();
      menu_box.append(&separator);
    }

    for item in items {
      if item.is_separator {
        let separator = super::styling::DropdownStyling::create_styled_separator();
        menu_box.append(&separator);
      } else {
        let menu_item = self.create_menu_item(item);
        if let Some(container) = menu_item.downcast_ref::<gtk::Box>() {
          containers.push(container.clone());
        }
        menu_box.append(&menu_item);
      }
    }

    *self.state.menu_boxes.borrow_mut() = containers;
    menu_box.upcast()
  }

  fn create_menu_item(&self, item: &MenuItem) -> gtk::Widget {
    let item_container = super::styling::DropdownStyling::create_styled_menu_item();

    super::styling::DropdownStyling::set_item_toggled(&item_container, item.is_toggled);

    self.setup_item_interactions(&item_container, item);

    let content_grid = gtk::Grid::builder().column_spacing(12).build();
    let mut col = 0;

    let icon_widget = super::utils::DropdownUtils::create_menu_icon(item);
    content_grid.attach(&icon_widget, col, 0, 1, 1);
    col += 1;

    let label = gtk::Label::builder()
      .label(&item.label)
      .halign(gtk::Align::Start)
      .hexpand(true)
      .build();
    content_grid.attach(&label, col, 0, 1, 1);
    col += 1;

    let arrow_widget = super::utils::DropdownUtils::create_submenu_indicator(item.submenu.is_some());
    content_grid.attach(&arrow_widget, col, 0, 1, 1);

    item_container.append(&content_grid);
    item_container.upcast()
  }

  fn create_back_button_with_breadcrumbs(&self, breadcrumbs: &[String]) -> gtk::Widget {
    let item_container = super::styling::DropdownStyling::create_styled_menu_item();

    let content_grid = gtk::Grid::builder().column_spacing(12).build();
    let mut col = 0;

    let back_icon = super::utils::DropdownUtils::create_back_icon();
    content_grid.attach(&back_icon, col, 0, 1, 1);
    col += 1;

    let back_label = super::utils::DropdownUtils::get_back_button_label(breadcrumbs);
    let label = gtk::Label::builder()
      .label(&back_label)
      .halign(gtk::Align::Start)
      .hexpand(true)
      .build();
    content_grid.attach(&label, col, 0, 1, 1);
    col += 1;

    let placeholder = gtk::Box::builder().width_request(16).build();
    content_grid.attach(&placeholder, col, 0, 1, 1);

    item_container.append(&content_grid);

    self.setup_back_button_interaction(&item_container);

    item_container.upcast()
  }

  fn setup_item_interactions(&self, item_container: &gtk::Box, item: &MenuItem) {
    let event_controller = gtk::GestureClick::new();
    item_container.add_controller(event_controller.clone());

    if item.submenu.is_some() {
      let submenu_items = item.submenu.clone().unwrap();
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
            let new_state = obj.toggle_item(&item_id);
            imp.emit_item_toggled(&item_id, new_state);
          } else {
            imp.emit_item_selected(&item_id);
          }
        }
      });
    }
  }

  fn setup_back_button_interaction(&self, item_container: &gtk::Box) {
    let event_controller = gtk::GestureClick::new();
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
}

impl WidgetImpl for DropdownMenuButtonPrivate {
  fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
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