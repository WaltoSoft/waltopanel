use gtk::glib::object::Cast;
use gtk::prelude::BoxExt;
use gtk::{Box, Grid, Label, Widget}; 
use gtk::{Align, Orientation, prelude::GridExt};

use crate::models::MenuItemModel;
use crate::helpers::ui_helpers;
use crate::traits::CompositeWidget;

pub struct MenuItem {
  container: Box,
}

impl MenuItem {
  pub fn new(model: MenuItemModel, menu_has_toggable_items: bool, menu_has_icons: bool) -> Self {
    let mut col = 0;
    let icon_size = 16;
    let column_spacing = 12;

    let menu_item_box = ui_helpers::create_styled_box(Orientation::Horizontal, 0, vec!["dropdown-item".to_string()]);
    let content_grid = Grid::builder().column_spacing(column_spacing).build();
    let icon_widget = ui_helpers::create_icon_widget(model.icon_name(), icon_size);

    let toggled_icon = if menu_has_toggable_items && model.toggled() {
      Some("object-select-symbolic".to_string())
    } else {
      None
    };

    if menu_has_toggable_items {
      let toggled_icon_widget = ui_helpers::create_icon_widget(toggled_icon, icon_size);  

      content_grid.attach(&toggled_icon_widget, col, 0, 1, 1);
      col += 1;
    }

    if menu_has_icons {
      content_grid.attach(&icon_widget, col, 0, 1, 1);
      col += 1;
    }

    let label = Label::builder()
      .label(model.text())
      .halign(Align::Start)
      .hexpand(true)
      .build();    

    content_grid.attach(&label, col, 0, 1, 1);
    col += 1;

    if model.has_submenu() {
      let sub_menu_icon = Some("go-next-symbolic".to_string());
      let arrow_icon_widget = ui_helpers::create_icon_widget(sub_menu_icon,icon_size);
      content_grid.attach(&arrow_icon_widget, col, 0, 1,1);
    } 

    menu_item_box.append(&content_grid);

    Self {
      container: menu_item_box,
    }
  }
}

impl CompositeWidget for MenuItem {
  fn widget(&self) -> Widget {
    self.container.clone().upcast()
  }
}