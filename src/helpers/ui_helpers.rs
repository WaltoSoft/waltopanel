use gtk::{Box, Image, Separator};
use gtk::{Orientation, Widget, glib::object::Cast};

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