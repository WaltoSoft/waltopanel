use adw::subclass::prelude::ObjectSubclassExt;
use gtk::{gdk::prelude::{DisplayExt, MonitorExt, SurfaceExt}, gio::prelude::ListModelExt, glib::object::Cast, prelude::{BoxExt, NativeExt, PopoverExt, WidgetExt}, Align, Box, Button, Orientation, Popover, PositionType, Widget};

use crate::models::MenuItem;
use super::DropdownMenuButtonPrivate;

impl DropdownMenuButtonPrivate {
  pub fn initialize(&self){ 
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

    self.dropdown_menu_button.set(button.clone()).expect("Button should only be set once during construction");
    self.popover.set(popover.clone()).expect("Popover should only be set once during construction");
    self.attach_dropdown_menu_button_handlers();

    Self::register_instance(&*obj);
  }

  pub fn finalize(&self) {
    if let Some(button) = self.dropdown_menu_button.get() {
      button.unparent();
    }
    if let Some(popover) = self.popover.get() {
      popover.unparent();
    }
  }

  /// Creates a GTK4 Box for the back button in the submenu.
  ///
  /// # Parameters
  /// - `breadcrumbs`: The breadcrumb trail for the current submenu.
  ///
  /// # Returns
  /// A GTK4 Box containing the back button.
  fn create_back_button(&self, breadcrumbs: &[String]) -> Widget {
    let menu_item_container = Self::create_styled_box(gtk::Orientation::Horizontal, 0, vec!["dropdown-menu".to_string()]);

    let content_grid = Self::create_menu_item_grid(
      Some("go-previous-symbolic"),
      16,
      &Self::get_back_button_label(breadcrumbs),
      false
    );

    self.attach_submenu_back_button_handler(&menu_item_container);

    menu_item_container.append(&content_grid);
    menu_item_container.upcast()
  }

  /// Generates the GTK4 Box from the given list of MenuItems
  /// If `is_submenu` is true, it will include a back button whose label
  /// is the last element of the `breadcrumbs` slice.
  /// # Parameters
  /// - `items`: The list of menu items to display.
  /// - `is_submenu`: Whether the menu is a submenu.
  /// - `breadcrumbs`: The breadcrumb trail for the current submenu.
  ///
  /// # Returns
  /// A GTK4 Box containing the menu items.
  fn create_menu(&self, items: &[MenuItem], is_submenu: bool, breadcrumbs: &[String]) -> Widget {
    let menu_box = Self::create_styled_box(Orientation::Vertical, 0, vec!["dropdown-menu".to_string()]);

    let mut containers = Vec::new();

    if is_submenu {
      let back_item = self.create_back_button(breadcrumbs);
      if let Some(container) = back_item.downcast_ref::<Box>() {
        containers.push(container.clone());
      }
      menu_box.append(&back_item);

      let separator = Self::create_styled_separator();
      menu_box.append(&separator);
    }

    for item in items {
      if item.is_separator {
        let separator = Self::create_styled_separator();
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


  /// Creates a GTK4 Box for the given `MenuItem`.
  /// The box contains a 1 row grid with the following columns:
  /// - Icon (optional, if a MenuItem is toggled, a checkmark icon is shown)
  /// - Label
  /// - Arrow (if submenu exists)
  ///
  /// # Parameters
  /// - `menu_item`: The `MenuItem` to create the box for.
  ///
  /// # Returns
  /// A GTK4 Box containing the menu item.
  fn create_menu_item(&self, menu_item: &MenuItem) -> Widget {
    let menu_item_container = Self::create_styled_box(gtk::Orientation::Horizontal, 0, vec!["dropdown-item".to_string()]);

    Self::set_item_toggled(&menu_item_container, menu_item.is_toggled);

    let icon_to_show = if menu_item.is_toggleable && menu_item.is_toggled {
      Some("object-select-symbolic")
    } else {
      menu_item.icon.as_deref()
    };

    let content_grid = Self::create_menu_item_grid(
      icon_to_show,
      16,
      &menu_item.label,
      menu_item.submenu.is_some()
    );

    self.attach_menu_item_handlers(&menu_item_container, menu_item);

    menu_item_container.append(&content_grid);
    menu_item_container.upcast()
  }

  pub fn get_back_button_label(breadcrumbs: &[String]) -> String {
    breadcrumbs.last().cloned().unwrap_or_else(|| "Back".to_string())
  }


  pub fn rebuild_menu(&self) {
    if let Some(popover) = self.popover.get() {
      if let Some(_child) = popover.child() {
        popover.set_child(Widget::NONE);
      }

      let items = self.state.menu_items.borrow().clone();
      let is_submenu = !self.state.sub_menu_stack.borrow().is_empty();
      let breadcrumbs = self.state.sub_menu_breadcrumbs.borrow().clone();
      
      if items.is_empty() {
        return;
      }

      let menu_box = self.create_menu(&items, is_submenu, &breadcrumbs);
      popover.set_child(Some(&menu_box));
    }
  }

  pub fn update_popover_alignment(popover: &Popover, button: &Button) {
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



}