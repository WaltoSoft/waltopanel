use gtk::{Box, Image, Separator};
use gtk::{Orientation, Widget, glib::object::Cast};

pub fn create_styled_box(orientation: Orientation, spacing: i32, css_classes: Vec<String>) -> Box {
  Box::builder()
    .orientation(orientation)
    .spacing(spacing)
    .css_classes(css_classes)
    .build()
}

pub fn create_icon_widget(icon_name: Option<String>, icon_size: i32) -> Widget {
  if let Some(name) = icon_name {
    let image = Image::from_icon_name(&name);
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

pub fn create_menu_separator() -> gtk::Separator {
  Separator::builder()
    .orientation(Orientation::Horizontal)
    .css_classes(vec!["dropdown-separator".to_string()])
    .build()
}