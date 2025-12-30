use gtk::{Box, Image, Widget, glib::object::{Cast, IsA}, prelude::WidgetExt};

pub trait WidgetExtensions {
  fn create_icon_widget(icon_name: Option<impl Into<String>>, icon_size: i32) -> Widget;
  fn get_sub_widget<T: IsA<Widget> + Clone>(&self) -> Option<T>;
}

impl<W: IsA<Widget>> WidgetExtensions for W {
  fn get_sub_widget<T: IsA<Widget> + Clone>(&self) -> Option<T> {
    let widget: &Widget = self.as_ref();

    if let Some(found) = widget.downcast_ref::<T>() {
      return Some(found.clone());
    }

    let mut child_opt = widget.first_child();
    while let Some(child) = child_opt {
      if let Some(found) = child.get_sub_widget::<T>() {
        return Some(found);
      }
      child_opt = child.next_sibling();
    }
    None
  }

  fn create_icon_widget(icon_name: Option<impl Into<String>>, icon_size: i32) -> Widget {
    if let Some(name) = icon_name {
      let image = Image::from_icon_name(&name.into());
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