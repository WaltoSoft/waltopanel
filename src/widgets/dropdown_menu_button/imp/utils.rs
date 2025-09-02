use gtk::{glib::object::Cast, prelude::GridExt};
use gtk::{Align, Box, Grid, Label, Widget};

use super::DropdownMenuButtonPrivate;

impl DropdownMenuButtonPrivate {
  /// Creates a GTK4 `Grid` representing a menu item.  A Grid is used to
  /// allow for consistent spacing between icon, label, and sub-menu indicator.
  ///
  /// # Parameters
  /// - `icon_name`: The name of the icon to display (if any).
  /// - `icon_size`: The size of the icon.
  /// - `label`: The text label for the menu item.
  /// - `has_sub_menu`: Whether the menu item has a sub-menu.
  /// # Returns
  /// A GTK4 `Grid` containing the menu item.
  pub fn create_menu_item_grid(icon_name: Option<&str>, icon_size: i32, label: &str, has_sub_menu: bool) -> Grid {
    let mut col = 0;
    let mut sub_menu_icon: Option<&str> = None;

    let content_grid = Grid::builder().column_spacing(12).build();
    let icon_widget = Self::create_icon_widget(icon_name, icon_size);

    content_grid.attach(&icon_widget, col, 0, 1, 1);
    col += 1;

    let label = Label::builder()
      .label(label)
      .halign(Align::Start)
      .hexpand(true)
      .build();

    content_grid.attach(&label, col, 0, 1, 1);
    col += 1;


    if has_sub_menu {
      sub_menu_icon = Some("go-next-symbolic");
    }

    let arrow_icon_widget = Self::create_icon_widget(sub_menu_icon,icon_size);
    content_grid.attach(&arrow_icon_widget, col, 0, 1,1);

    content_grid
  }

  /// Creates an icon widget for the menu item.
  /// If an icon name is provided, an image widget is created; otherwise, a placeholder is used.
  ///
  /// # Parameters
  /// - `icon_name`: The name of the icon to display (if any).
  /// - `icon_size`: The size of the icon.
  /// # Returns
  /// A GTK4 `Widget` representing the icon.
  pub fn create_icon_widget(icon_name: Option<&str>, icon_size: i32) -> Widget {
    if let Some(name) = icon_name {
      let image = gtk::Image::from_icon_name(name);
      image.set_pixel_size(icon_size);
      image.upcast::<Widget>()
    
    } else {
      let placeholder = Box::builder()
        .width_request(icon_size)
        .height_request(icon_size)
        .build();
    
      placeholder.upcast()    
    }
  }
}