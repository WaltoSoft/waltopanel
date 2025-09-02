use gtk::prelude::*;

use super::DropdownMenuButtonPrivate;

impl DropdownMenuButtonPrivate {
  pub fn create_styled_box(orientation: gtk::Orientation, spacing: i32, css_classes: Vec<String>) -> gtk::Box {
    gtk::Box::builder()
      .orientation(orientation)
      .spacing(spacing)
      .css_classes(css_classes)
      .build()
  }

  pub fn create_styled_separator() -> gtk::Separator {
    gtk::Separator::builder()
      .orientation(gtk::Orientation::Horizontal)
      .css_classes(vec!["dropdown-separator".to_string()])
      .build()
  }

  pub fn set_item_toggled(item_container: &gtk::Box, toggled: bool) {
    if toggled {
      item_container.add_css_class("toggled");
    } else {
      item_container.remove_css_class("toggled");
    }
  }
}