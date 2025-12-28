use gtk::prelude::WidgetExt;
use gtk::{glib, Box, Image, ListBox};
use gtk::{SelectionMode, Widget, glib::object::Cast};

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

pub fn defer_listbox_selection(list_box: &ListBox) {
  let list_box_clone = list_box.clone();
  glib::idle_add_local_once(move || {
    list_box_clone.set_selection_mode(SelectionMode::Browse);
    list_box_clone.unselect_all();
    list_box_clone.grab_focus();
    if let Some(first_row) = list_box_clone.row_at_index(0) {
      list_box_clone.select_row(Some(&first_row));
      first_row.grab_focus();
    }
  });
}