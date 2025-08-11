use gtk::{gdk, glib, {prelude::*}};
use std::{{cell::RefCell},{rc::Rc}};

use crate::models::MenuItem;

thread_local! {
  static DROPDOWN_INSTANCES: RefCell<Vec<gtk::Popover>> = RefCell::new(Vec::new());
}


#[derive(Clone)]
pub struct DropdownButton {
  pub menu_items: Rc<RefCell<Vec<MenuItem>>>,
  pub on_item_selected: Rc<RefCell<Option<Box<dyn Fn(&str, bool) + 'static>>>>,
  button: gtk::Button,
  popover: gtk::Popover,
  focused_item_index: Rc<RefCell<Option<usize>>>,
  menu_boxes: Rc<RefCell<Vec<gtk::Box>>>,
  sub_menu_stack: Rc<RefCell<Vec<Vec<MenuItem>>>>,
  sub_menu_breadcrumbs: Rc<RefCell<Vec<String>>>,
}

impl DropdownButton {
  pub fn new() -> Self {
    let button = gtk::Button::new();

    let popover = gtk::Popover::builder()
      .autohide(false)
      .has_arrow(false)
      .position(gtk::PositionType::Bottom)
      .can_focus(true)
      .focusable(true)
      .build();

    popover.set_parent(&button);

    let menu_items = Rc::new(RefCell::new(Vec::new()));
    let popover_clone = popover.clone();
    let button_clone = button.clone();

    button.connect_clicked(move |_| {
      if popover_clone.is_visible() {
        popover_clone.popdown();
      } else {
        Self::close_all_other_dropdowns(&popover_clone);
        Self::update_popover_alignment(&popover_clone, &button_clone);
        popover_clone.popup();
        popover_clone.grab_focus();
      }
    });

    // Add hover behavior for menu switching
    let hover_controller = gtk::EventControllerMotion::new();
    let popover_hover = popover.clone();
    let button_hover = button.clone();
    hover_controller.connect_enter(move |_, _, _| {
      // Check if any dropdown is currently open
      if Self::any_dropdown_is_open() {
        Self::close_all_other_dropdowns(&popover_hover);
        Self::update_popover_alignment(&popover_hover, &button_hover);
        popover_hover.popup();
        popover_hover.grab_focus();
      }
    });
    button.add_controller(hover_controller);

    let dropdown_button = Self {
      button: button.clone(),
      popover: popover.clone(),
      menu_items: menu_items.clone(),
      on_item_selected: Rc::new(RefCell::new(None)),
      focused_item_index: Rc::new(RefCell::new(None)),
      menu_boxes: Rc::new(RefCell::new(Vec::new())),
      sub_menu_stack: Rc::new(RefCell::new(Vec::new())),
      sub_menu_breadcrumbs: Rc::new(RefCell::new(Vec::new())),
    };

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

    dropdown_button.setup_keyboard_navigation();
    dropdown_button
  }

  pub fn with_text(self, text: &str) -> Self {
    self.button.set_label(text);
    self
  }

  pub fn with_icon(self, icon_name: &str) -> Self {
    let icon = gtk::Image::from_icon_name(icon_name);
    self.button.set_child(Some(&icon));
    self
  }

  pub fn with_icon_and_text(self, icon_name: &str, text: &str) -> Self {
    let container = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .spacing(6)
      .build();

    let icon = gtk::Image::from_icon_name(icon_name);
    let label = gtk::Label::new(Some(text));

    container.append(&icon);
    container.append(&label);

    self.button.set_child(Some(&container));
    self
  }

  pub fn set_menu_items(&self, items: Vec<MenuItem>) {
    *self.menu_items.borrow_mut() = items;
    self.rebuild_menu();
  }

  pub fn on_item_toggled<F>(self, callback: F) -> Self
  where
    F: Fn(&str, bool) + 'static,
  {
    *self.on_item_selected.borrow_mut() = Some(Box::new(callback));
    self
  }

  fn setup_keyboard_navigation(&self) {
    let key_controller = gtk::EventControllerKey::new();
    let focused_index = self.focused_item_index.clone();
    let menu_items = self.menu_items.clone();
    let menu_boxes = self.menu_boxes.clone();
    let callback = self.on_item_selected.clone();
    let popover = self.popover.clone();
    let menu_stack = self.sub_menu_stack.clone();
    let sub_menu_breadcrumbs = self.sub_menu_breadcrumbs.clone();
    let dropdown_self = self.clone();

    key_controller.connect_key_pressed(move |_, key, _, _| {
      let container_count = menu_boxes.borrow().len();

      if container_count == 0 {
        return false.into();
      }

      let focused_item_data = if matches!(key, gdk::Key::Return | gdk::Key::KP_Enter) {
        if let Some(focused_idx) = *focused_index.borrow() {
          let items = menu_items.borrow();
          let non_separator_items: Vec<_> = items
            .iter()
            .enumerate()
            .filter(|(_, item)| !item.is_separator)
            .collect();
          non_separator_items
            .get(focused_idx)
            .map(|(_, item)| (item.id.clone(), item.submenu.clone()))
        } else {
          None
        }
      } else {
        None
      };

      match key {
        gdk::Key::Down => {
          let mut current_focused = focused_index.borrow_mut();
          let new_index = match *current_focused {
            None => 0,
            Some(idx) => (idx + 1) % container_count,
          };
          *current_focused = Some(new_index);
          drop(current_focused);
          Self::update_visual_focus(&menu_boxes, &[], Some(new_index));
          true.into()
        }
        gdk::Key::Up => {
          let mut current_focused = focused_index.borrow_mut();
          let new_index = match *current_focused {
            None => container_count - 1,
            Some(0) => container_count - 1,
            Some(idx) => idx - 1,
          };
          *current_focused = Some(new_index);
          drop(current_focused);
          Self::update_visual_focus(&menu_boxes, &[], Some(new_index));
          true.into()
        }
        gdk::Key::Return | gdk::Key::KP_Enter => {
          if let Some((item_id, submenu)) = focused_item_data {
            if submenu.is_some() {
              // Submenu navigation handled by right arrow key
            } else {
              popover.popdown();
              if let Some(cb) = callback.borrow().as_ref() {
                cb(&item_id, false);
              }
            }
          }
          true.into()
        }
        gdk::Key::Right => {
          if let Some(focused_idx) = *focused_index.borrow() {
            let (submenu_items, item_label) = {
              let items = menu_items.borrow();
              let non_separator_items: Vec<_> = items
                .iter()
                .enumerate()
                .filter(|(_, item)| !item.is_separator)
                .collect();

              if let Some((_, focused_item)) = non_separator_items.get(focused_idx) {
                if let Some(submenu) = &focused_item.submenu {
                  (Some(submenu.clone()), focused_item.label.clone())
                } else {
                  (None, String::new())
                }
              } else {
                (None, String::new())
              }
            };

            if let Some(submenu) = submenu_items {
              let dropdown_clone = dropdown_self.clone();
              let menu_items_clone = menu_items.clone();
              let menu_stack_clone = menu_stack.clone();
              let sub_menu_breadcrumbs_clone = sub_menu_breadcrumbs.clone();
              let focused_index_clone = focused_index.clone();
              let submenu_clone = submenu.clone();
              let item_label_clone = item_label.clone();

              glib::idle_add_local_once(move || {
                let current_menu = menu_items_clone.borrow().clone();

                menu_stack_clone.borrow_mut().push(current_menu);
                sub_menu_breadcrumbs_clone.borrow_mut().push(item_label_clone);
                *menu_items_clone.borrow_mut() = submenu_clone;
                *focused_index_clone.borrow_mut() = None;
                dropdown_clone.rebuild_menu();
              });
              return true.into();
            }
          }
          false.into()
        }
        gdk::Key::Left => {
          if !menu_stack.borrow().is_empty() {
            let dropdown_clone = dropdown_self.clone();
            let menu_items_clone = menu_items.clone();
            let menu_stack_clone = menu_stack.clone();
            let sub_menu_breadcrumbs_clone = sub_menu_breadcrumbs.clone();
            let focused_index_clone = focused_index.clone();

            glib::idle_add_local_once(move || {
              let previous_menu = menu_stack_clone.borrow_mut().pop();
              if let Some(previous_menu) = previous_menu {
                sub_menu_breadcrumbs_clone.borrow_mut().pop();
                *menu_items_clone.borrow_mut() = previous_menu;
                *focused_index_clone.borrow_mut() = None;
                dropdown_clone.rebuild_menu();
              }
            });
            return true.into();
          }
          false.into()
        }
        gdk::Key::Escape => {
          popover.popdown();
          true.into()
        }
        _ => false.into(),
      }
    });

    self.popover.add_controller(key_controller);

    let focused_index_clone = self.focused_item_index.clone();
    let menu_boxes_clone = self.menu_boxes.clone();
    let menu_items_clone = self.menu_items.clone();

    let menu_stack_clone = self.sub_menu_stack.clone();
    let sub_menu_breadcrumbs_clone = self.sub_menu_breadcrumbs.clone();

    self.popover.connect_show(move |_| {
      *focused_index_clone.borrow_mut() = None;
      menu_stack_clone.borrow_mut().clear();
      sub_menu_breadcrumbs_clone.borrow_mut().clear();

      let items = menu_items_clone.borrow();
      let non_separator_items: Vec<_> = items
        .iter()
        .enumerate()
        .filter(|(_, item)| !item.is_separator)
        .collect();
      Self::update_visual_focus(&menu_boxes_clone, &non_separator_items, None);
    });
  }

  fn update_visual_focus(
    menu_boxes: &Rc<RefCell<Vec<gtk::Box>>>,
    _non_separator_items: &[(usize, &MenuItem)],
    focused_index: Option<usize>,
  ) {
    let containers = menu_boxes.borrow();

    for (container_idx, container) in containers.iter().enumerate() {
      if let Some(focused_idx) = focused_index {
        if container_idx == focused_idx {
          let css_provider = gtk::CssProvider::new();
          css_provider.load_from_data(
            "* { background-color: @theme_selected_bg_color; color: @theme_selected_fg_color; }",
          );
          container
            .style_context()
            .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1);
        } else {
          let css_provider = gtk::CssProvider::new();
          css_provider.load_from_data("* { background-color: transparent; }");
          container
            .style_context()
            .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1);
        }
      } else {
        let css_provider = gtk::CssProvider::new();
        css_provider.load_from_data("* { background-color: transparent; }");
        container
          .style_context()
          .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1);
      }
    }
  }

  pub fn toggle_item(&self, item_id: &str) -> bool {
    let new_state = {
      let mut items = self.menu_items.borrow_mut();
      if let Some(item) = items.iter_mut().find(|i| i.id == item_id) {
        item.is_toggled = !item.is_toggled;
        item.is_toggled
      } else {
        false
      }
    };
    self.rebuild_menu();
    new_state
  }

  pub fn set_item_toggled(&self, item_id: &str, toggled: bool) {
    {
      let mut items = self.menu_items.borrow_mut();
      if let Some(item) = items.iter_mut().find(|i| i.id == item_id) {
        item.is_toggled = toggled;
      }
    }
    self.rebuild_menu();
  }

  fn rebuild_menu(&self) {
    if let Some(_child) = self.popover.child() {
      self.popover.set_child(gtk::Widget::NONE);
    }

    let items = self.menu_items.borrow();
    if items.is_empty() {
      return;
    }

    let menu_box = self.create_menu_container(&items);
    self.popover.set_child(Some(&menu_box));
  }

  fn create_menu_container(&self, items: &[MenuItem]) -> gtk::Widget {
    let menu_box = gtk::Box::builder()
      .orientation(gtk::Orientation::Vertical)
      .spacing(0)
      .css_classes(vec!["dropdown-menu".to_string()])
      .build();

    let mut containers = Vec::new();

    let has_stack = !self.sub_menu_stack.borrow().is_empty();
    if has_stack {
      let back_item = self.create_back_button();
      if let Some(container) = back_item.downcast_ref::<gtk::Box>() {
        containers.push(container.clone());
      }
      menu_box.append(&back_item);

      let separator = gtk::Separator::builder()
        .orientation(gtk::Orientation::Horizontal)
        .css_classes(vec!["dropdown-separator".to_string()])
        .build();
      menu_box.append(&separator);
    }

    for item in items {
      if item.is_separator {
        let separator = gtk::Separator::builder()
          .orientation(gtk::Orientation::Horizontal)
          .css_classes(vec!["dropdown-separator".to_string()])
          .build();
        menu_box.append(&separator);
      } else {
        let menu_item = self.create_menu_item(item);

        if let Some(container) = menu_item.downcast_ref::<gtk::Box>() {
          containers.push(container.clone());
        }
        menu_box.append(&menu_item);
      }
    }

    *self.menu_boxes.borrow_mut() = containers;
    menu_box.upcast()
  }

  fn create_menu_item(&self, item: &MenuItem) -> gtk::Widget {
    let item_container = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .css_classes(vec!["dropdown-item".to_string()])
      .build();

    if item.is_toggled {
      item_container.add_css_class("toggled");
    }

    self.setup_hover_styling(&item_container);

    if item.submenu.is_some() {
      let submenu_items = item.submenu.clone().unwrap();
      let item_label = item.label.clone();
      let menu_stack = self.sub_menu_stack.clone();
      let sub_menu_breadcrumbs = self.sub_menu_breadcrumbs.clone();
      let current_items = self.menu_items.clone();
      let dropdown_clone = self.clone();

      let event_controller = gtk::GestureClick::new();
      item_container.add_controller(event_controller.clone());

      event_controller.connect_released(move |_, _, _, _| {
        menu_stack.borrow_mut().push(current_items.borrow().clone());
        sub_menu_breadcrumbs.borrow_mut().push(item_label.clone());
        *current_items.borrow_mut() = submenu_items.clone();
        dropdown_clone.rebuild_menu();
      });
    } else {
      let item_id = item.id.clone();
      let popover = self.popover.clone();
      let callback_ref = self.on_item_selected.clone();

      let event_controller = gtk::GestureClick::new();
      item_container.add_controller(event_controller.clone());

      event_controller.connect_released(move |_, _, _, _| {
        popover.popdown();

        if let Some(callback) = callback_ref.borrow().as_ref() {
          callback(&item_id, false);
        }
      });
    }

    let content_grid = gtk::Grid::builder().column_spacing(12).build();

    let mut col = 0;

    let icon_placeholder = gtk::Box::builder()
      .width_request(16)
      .height_request(16)
      .build();

    if item.is_toggled {
      let checkmark = gtk::Image::from_icon_name("object-select-symbolic");
      checkmark.set_pixel_size(16);
      content_grid.attach(&checkmark, col, 0, 1, 1);
    } else if let Some(icon_name) = &item.icon {
      let icon = gtk::Image::from_icon_name(icon_name);
      icon.set_pixel_size(16);
      content_grid.attach(&icon, col, 0, 1, 1);
    } else {
      content_grid.attach(&icon_placeholder, col, 0, 1, 1);
    }
    col += 1;

    let label = gtk::Label::builder()
      .label(&item.label)
      .halign(gtk::Align::Start)
      .hexpand(true)
      .build();
    content_grid.attach(&label, col, 0, 1, 1);
    col += 1;

    if item.submenu.is_some() {
      let submenu_arrow = gtk::Image::from_icon_name("go-next-symbolic");
      submenu_arrow.set_pixel_size(12);
      submenu_arrow.set_halign(gtk::Align::End);
      content_grid.attach(&submenu_arrow, col, 0, 1, 1);
    } else {
      let arrow_placeholder = gtk::Box::builder().width_request(16).build();
      content_grid.attach(&arrow_placeholder, col, 0, 1, 1);
    }

    item_container.append(&content_grid);

    item_container.upcast()
  }

  fn create_back_button(&self) -> gtk::Widget {
    let item_container = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .css_classes(vec!["dropdown-item".to_string()])
      .build();

    self.setup_hover_styling(&item_container);

    let event_controller = gtk::GestureClick::new();
    item_container.add_controller(event_controller.clone());

    let content_grid = gtk::Grid::builder().column_spacing(12).build();

    let mut col = 0;

    let back_icon = gtk::Image::from_icon_name("go-previous-symbolic");
    back_icon.set_pixel_size(16);
    content_grid.attach(&back_icon, col, 0, 1, 1);
    col += 1;

    let back_label = if let Some(title) = self.sub_menu_breadcrumbs.borrow().last() {
      title.clone()
    } else {
      "Back".to_string()
    };

    let label = gtk::Label::builder()
      .label(&back_label)
      .halign(gtk::Align::Start)
      .hexpand(true)
      .build();
    content_grid.attach(&label, col, 0, 1, 1);
    col += 1;

    let arrow_placeholder = gtk::Box::builder().width_request(16).build();
    content_grid.attach(&arrow_placeholder, col, 0, 1, 1);

    item_container.append(&content_grid);

    let menu_stack = self.sub_menu_stack.clone();
    let sub_menu_breadcrumbs = self.sub_menu_breadcrumbs.clone();
    let current_items = self.menu_items.clone();
    let dropdown_clone = self.clone();

    event_controller.connect_released(move |_, _, _, _| {
      let previous_menu = menu_stack.borrow_mut().pop();
      if let Some(previous_menu) = previous_menu {
        sub_menu_breadcrumbs.borrow_mut().pop();
        *current_items.borrow_mut() = previous_menu;
        dropdown_clone.rebuild_menu();
      }
    });

    item_container.upcast()
  }

  fn any_dropdown_is_open() -> bool {
    DROPDOWN_INSTANCES.with(|instances| {
      instances.borrow().iter().any(|popover| {
        popover.parent().is_some() && popover.is_visible()
      })
    })
  }

  fn close_all_other_dropdowns(current_popover: &gtk::Popover) {
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

  fn update_popover_alignment(popover: &gtk::Popover, button: &gtk::Button) {
    if let Some(surface) = button.native().and_then(|n| n.surface()) {
      let display = surface.display();
      let monitor = display
        .monitor_at_surface(&surface)
        .unwrap_or_else(|| display.monitors().item(0).unwrap().downcast().unwrap());
      let monitor_geometry = monitor.geometry();

      let (button_x, _) = button
        .translate_coordinates(&button.root().unwrap(), 0.0, 0.0)
        .unwrap_or((0.0, 0.0));

      let button_width = button.allocated_width();
      let menu_width = 200;
      let space_right = monitor_geometry.width() - (button_x as i32 + button_width);

      if space_right >= menu_width {
        popover.set_halign(gtk::Align::Start);
      } else {
        popover.set_halign(gtk::Align::End);
      }
    }
  }

  fn setup_hover_styling(&self, item_container: &gtk::Box) {
    let motion_controller = gtk::EventControllerMotion::new();

    let item_container_enter = item_container.clone();
    let focused_index = self.focused_item_index.clone();
    let menu_boxes = self.menu_boxes.clone();
    motion_controller.connect_enter(move |_, _, _| {
      *focused_index.borrow_mut() = None;
      Self::update_visual_focus(&menu_boxes, &[], None);

      let css_provider = gtk::CssProvider::new();
      css_provider.load_from_data(
        "* { background-color: @theme_selected_bg_color; color: @theme_selected_fg_color; }",
      );
      item_container_enter
        .style_context()
        .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1);
    });

    let item_container_leave = item_container.clone();
    motion_controller.connect_leave(move |_| {
      let css_provider = gtk::CssProvider::new();
      css_provider.load_from_data("* { background-color: transparent; }");
      item_container_leave
        .style_context()
        .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1);
    });

    item_container.add_controller(motion_controller);
  }

  pub fn widget(&self) -> &gtk::Button {
    &self.button
  }
}

impl Default for DropdownButton {
  fn default() -> Self {
    Self::new()
  }
}
