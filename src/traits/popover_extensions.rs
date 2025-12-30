use gtk::{Align, EventControllerKey, Popover, gdk::{Key, prelude::{DisplayExt, MonitorExt, SurfaceExt}}, gio::prelude::ListModelExt, glib::object::Cast, prelude::{NativeExt, WidgetExt}};

pub trait PopoverExtensions {
  fn connect_key_pressed<F: 'static + Fn(Key) -> gtk::glib::Propagation>(&self, callback: F);
  fn update_popover_alignment(&self);
}

impl PopoverExtensions for Popover {
  fn connect_key_pressed<F>(&self, callback: F) 
  where
    F: Fn(Key) -> gtk::glib::Propagation + 'static,
  {
    let key_controller = EventControllerKey::new();
    key_controller.connect_key_pressed(move |_, keyval, _, _| callback(keyval));
  
    self.add_controller(key_controller);
  }

  fn update_popover_alignment(&self) {
    let Some(button_menu_box) = self.parent() else {
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
      let menu_width = 200;  // TODO: Magic number needs to be fixed.
      let space_right = monitor_geometry.width() - (button_x as i32 + button_width);

      if space_right >= menu_width {
        self.set_halign(Align::Start);
      } else {
        self.set_halign(Align::End);
      }
    }
  }
}
