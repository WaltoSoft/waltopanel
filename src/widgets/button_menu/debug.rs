use adw::subclass::prelude::{ObjectSubclassExt, ObjectSubclassIsExt};
use gtk::{prelude::WidgetExt, GestureClick};

use super::ButtonMenuPrivate;

impl ButtonMenuPrivate {
  pub fn debug_current_menu(&self) {
      let menu = self.current_menu.borrow();
      println!("=== Current Menu ({} items) ===", menu.count());
      for (i, item) in menu.iter().enumerate() {
          println!("  [{}] text: {:?}, icon: {:?}, toggled: {}, has_submenu: {}", 
              i,
              item.text(),
              item.icon_name(),
              item.allow_toggle() && item.toggled(),
              item.has_submenu()
          );
      }
      println!("===================");
  }
}