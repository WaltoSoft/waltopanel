use gtk::{Box, Grid, Label, Orientation, Widget};
use gtk::{Align, glib::object::Cast, GestureClick, prelude::{BoxExt, GridExt, WidgetExt}};

use crate::helpers::ui_helpers;
use crate::traits::CompositeWidget; 

#[derive(Clone, Debug)]
pub struct BackButton {
  container: Box,
  click_gesture: GestureClick,
}

impl BackButton {
  pub fn new(text: String) -> Self{
    let icon_size = 16;
    let column_spacing = 12;

    let container = Box::builder()
      .orientation(Orientation::Horizontal)
      .css_classes(vec!["back-button".to_string()])
      .build();
    
    let content_grid = Grid::builder()
      .column_spacing(column_spacing)
      .build();
    
    let back_icon_widget = ui_helpers::create_icon_widget(
      Some("go-previous-symbolic".to_string()), 
      icon_size);

    let label = Label::builder()
      .label(text)
      .halign(Align::Start)
      .hexpand(true)
      .build();

    let click_gesture = GestureClick::new();
      
    content_grid.attach(&back_icon_widget, 0, 0, 1, 1);
    content_grid.attach(&label, 1, 0, 1, 1);

    container.append(&content_grid);
    container.add_controller(click_gesture.clone());
    
    Self {
      container,
      click_gesture,
    }
  } 

  pub fn connect_clicked<F>(&self, callback: F)
  where
    F: Fn() + 'static,
  {
    self.click_gesture.connect_released(move |_, _, _, _| {
      callback();
    });
  } 
}

impl CompositeWidget for BackButton {
  fn widget(&self) -> Widget {
    self.container.clone().upcast()
  }
}