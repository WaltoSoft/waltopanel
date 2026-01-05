use gtk::Widget;
use gtk::glib::object::Cast;

use crate::traits::CompositeWidget;
use crate::util::process;
use crate::widgets::PanelButton;
use crate::models::MenuBuilder;

pub struct SystemButton {
  panel_button: PanelButton,
}

impl SystemButton {
  pub fn new() -> Self {
    let menu = MenuBuilder::new()
      .item("lock", "Lock").icon("system-lock-screen-symbolic")
      .item("logout", "Log Out").icon("system-log-out-symbolic")
      .item("restart", "Restart").icon("system-restart-symbolic")
      .item("shutdown", "Shut Down").icon("system-shutdown-symbolic")
      .item("suspend", "Suspend").icon("system-suspend-symbolic")
      .item("hibernate", "Hibernate").icon("system-suspend-hibernate-symbolic")
      .build();

    let panel_button = PanelButton::from_icon_name("system-shutdown-symbolic");
    panel_button.set_menu(menu);
    
    panel_button.connect_menu_item_clicked(move |_, menu_item| {
      match menu_item.id().as_str() {
        "lock" => {
          process::spawn_detached("loginctl lock-session");
        }
        "logout" => {
          process::spawn_detached("gnome-session-quit --logout --no-prompt");
        }
        "restart" => {
          process::spawn_detached("systemctl reboot");
        }
        "shutdown" => {
          process::spawn_detached("systemctl poweroff");
        }
        "suspend" => {
          process::spawn_detached("systemctl suspend");
        }
        "hibernate" => {
          process::spawn_detached("systemctl hibernate");
        }
        _ => {}
      }
    });

    Self { panel_button  }
  }
}

impl CompositeWidget for SystemButton {
  fn widget(&self) -> Widget {
    self.panel_button.clone().upcast()
  }
}