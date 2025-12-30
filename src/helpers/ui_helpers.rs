use gtk::gdk::prelude::{DisplayExt, MonitorExt, SurfaceExt};
use gtk::gio::prelude::ListModelExt;
use gtk::glib::object::{IsA, ObjectExt};
use gtk::prelude::{NativeExt, WidgetExt};
use gtk::{Box, Image, ListBox, Popover};
use gtk::{glib, Align, SelectionMode, Widget, glib::object::Cast};

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

pub fn update_popover_alignment(popover: &Popover) {
  let Some(button_menu_box) = popover.parent() else {
    return;
  };

  if let Some(surface) = button_menu_box.native().and_then(|n| n.surface()) {
    let display = surface.display();
    
    let monitor = display
      .monitor_at_surface(&surface)
      .or_else(|| {
        display.monitors().item(0)?.downcast().ok()
      });
    
    let Some(monitor) = monitor else {
      return;
    };
    
    let monitor_geometry = monitor.geometry();

    let (button_x, _) = button_menu_box
      .root()
      .and_then(|root| button_menu_box.translate_coordinates(&root, 0.0, 0.0))
      .unwrap_or((0.0, 0.0));

    let button_width = button_menu_box.allocated_width();
    let menu_width = 200;  //TODO: Magic number needs to be fixed.
    let space_right = monitor_geometry.width() - (button_x as i32 + button_width);

    if space_right >= menu_width {
      popover.set_halign(Align::Start);
    } else {
      popover.set_halign(Align::End);
    }
  }
}

pub fn get_sub_widget<T: IsA<Widget> + Clone>(parent: &Widget) -> Option<T> {
  parent
    .first_child()
    .and_then(|child| child.downcast::<Widget>().ok())
    .and_then(|widget| {
      // check if the widget is of type T
      if widget.is::<T>() {
        widget.downcast::<T>().ok()
      } else {
        get_sub_widget::<T>(&widget)
      }
    })
}