use gtk::prelude::*;
use std::cell::RefCell;
use crate::models::MenuItem;

thread_local! {
  static DROPDOWN_INSTANCES: RefCell<Vec<gtk::Popover>> = RefCell::new(Vec::new());
}

pub struct DropdownUtils;

impl DropdownUtils {
  pub fn update_popover_alignment(popover: &gtk::Popover, button: &gtk::Button) {
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

  pub fn get_non_separator_items(items: &[MenuItem]) -> Vec<(usize, &MenuItem)> {
    items
      .iter()
      .enumerate()
      .filter(|(_, item)| !item.is_separator)
      .collect()
  }

  pub fn create_menu_icon(item: &MenuItem) -> gtk::Widget {
    if item.is_toggled {
      let checkmark = gtk::Image::from_icon_name("object-select-symbolic");
      checkmark.set_pixel_size(16);
      checkmark.upcast()
    } else if let Some(icon_name) = &item.icon {
      let icon = gtk::Image::from_icon_name(icon_name);
      icon.set_pixel_size(16);
      icon.upcast()
    } else {
      let placeholder = gtk::Box::builder()
        .width_request(16)
        .height_request(16)
        .build();
      placeholder.upcast()
    }
  }

  pub fn create_submenu_indicator(has_submenu: bool) -> gtk::Widget {
    if has_submenu {
      let arrow = gtk::Image::from_icon_name("go-next-symbolic");
      arrow.set_pixel_size(12);
      arrow.set_halign(gtk::Align::End);
      arrow.upcast()
    } else {
      let placeholder = gtk::Box::builder().width_request(16).build();
      placeholder.upcast()
    }
  }

  pub fn create_back_icon() -> gtk::Image {
    let icon = gtk::Image::from_icon_name("go-previous-symbolic");
    icon.set_pixel_size(16);
    icon
  }


  pub fn get_back_button_label(breadcrumbs: &[String]) -> String {
    breadcrumbs.last().cloned().unwrap_or_else(|| "Back".to_string())
  }

  pub fn register_popover_instance(popover: &gtk::Popover) {
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

  pub fn close_all_other_dropdowns(current_popover: &gtk::Popover) {
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

  pub fn close_all_dropdowns() {
    DROPDOWN_INSTANCES.with(|instances| {
      let mut instances = instances.borrow_mut();
      instances.retain(|popover| {
        if let Some(_parent) = popover.parent() {
          if popover.is_visible() {
            popover.popdown();
          }
          true
        } else {
          false
        }
      });
    });
  }

  pub fn any_dropdown_is_open() -> bool {
    DROPDOWN_INSTANCES.with(|instances| {
      instances
        .borrow()
        .iter()
        .any(|popover| popover.parent().is_some() && popover.is_visible())
    })
  }

  pub fn clear_all_active_states() {
    DROPDOWN_INSTANCES.with(|instances| {
      let instances = instances.borrow();
      for popover in instances.iter() {
        if let Some(button) = popover
          .parent()
          .and_then(|p| p.downcast::<gtk::Button>().ok())
        {
          button.remove_css_class("menu-button-active");
        }
      }
    });
  }
}