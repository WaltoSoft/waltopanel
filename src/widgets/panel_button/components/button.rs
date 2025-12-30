use gtk::{Box, Image, Label, Widget};
use gtk::{GestureClick, Orientation};
use gtk::glib::object::{Cast, IsA, ObjectExt};
use gtk::prelude::{BoxExt, WidgetExt};

use crate::traits::CompositeWidget;

#[derive(Clone, Debug)]
pub struct Button {
  container: Box,
  click_gesture: GestureClick,
}

impl Button {
  pub fn new(parent: &impl IsA<Widget>) -> Self {
    let container = Box::builder()
      .orientation(Orientation::Horizontal)
      .spacing(10)
      .build();

    let icon_image = Image::new();
    let text_label = Label::new(None);

    parent.bind_property("icon-name", &icon_image, "icon-name").build();
    parent.bind_property("text", &text_label, "label").build();

    let click_gesture = GestureClick::new();

    container.append(&icon_image);
    container.append(&text_label);
    container.add_controller(click_gesture.clone());
    container.set_parent(parent);

    Self {
      container,
      click_gesture,
    }
  }

  pub fn measure(&self, orientation: Orientation, for_size: i32) -> (i32, i32, i32, i32) {
    self.container.measure(orientation, for_size)
  } 

  pub fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
    self.container.allocate(width, height, baseline, None);
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

impl CompositeWidget for Button {
  fn widget(&self) -> Widget {
    self.container.clone().upcast()
  }
}