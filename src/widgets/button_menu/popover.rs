use gtk::{gdk::prelude::{DisplayExt, MonitorExt, SurfaceExt}, gio::prelude::ListModelExt, glib::object::Cast, prelude::{NativeExt, PopoverExt, WidgetExt}, Align, Box, Popover};

use super::ButtonMenuPrivate;

impl ButtonMenuPrivate {
  pub fn toggle_popover(&self) {
    if let (Some(button_menu_box), Some(popover)) = (self.button_menu_box.get(), self.popover.get()) {
      if popover.is_visible() {
        popover.popdown();
      } else {
        Self::close_other_button_menus(popover);
        Self::update_popover_alignment(popover, button_menu_box);
        popover.popup();
      }
    }
  }

  pub fn update_popover_alignment(popover: &Popover, button_menu_box: &Box) {
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
      let menu_width = 200;
      let space_right = monitor_geometry.width() - (button_x as i32 + button_width);

      if space_right >= menu_width {
        popover.set_halign(Align::Start);
      } else {
        popover.set_halign(Align::End);
      }
    }
  }  
}