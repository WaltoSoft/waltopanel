use gtk::glib::object::Cast;
use gtk::prelude::BoxExt;
use gtk::{Box, Grid, Label, Widget};
use gtk::{Align, prelude::GridExt};

use crate::helpers::ui_helpers;
use crate::traits::CompositeWidget; 

pub struct BackButton {
  container: Box
}

impl BackButton {
  pub fn new(text: String) -> Self{
    let icon_size = 16;
    let column_spacing = 12;
    let menu_item_box = ui_helpers::create_styled_box(gtk::Orientation::Horizontal, 0, vec!["dropdown-item".to_string()]);
    let content_grid = Grid::builder().column_spacing(column_spacing).build();

    let back_icon_widget = ui_helpers::create_icon_widget(Some("go-previous-symbolic".to_string()), icon_size);
    content_grid.attach(&back_icon_widget, 0, 0, 1, 1);

    let label = Label::builder()
      .label(text)
      .halign(Align::Start)
      .hexpand(true)
      .build();
    
    content_grid.attach(&label, 1, 0, 1, 1);

    menu_item_box.append(&content_grid);
    
    Self {
      container: menu_item_box
    }
  } 
}

impl CompositeWidget for BackButton {
  fn widget(&self) -> Widget {
    self.container.clone().upcast()
  }
}