use gtk::Widget;
use gtk::glib::object::Cast;

use crate::traits::CompositeWidget;
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
          let _ = std::process::Command::new("loginctl").arg("lock-session").spawn();
        }
        "logout" => {
          let _ = std::process::Command::new("gnome-session-quit").arg("--logout").arg("--no-prompt").spawn();
        }
        "restart" => {
          let _ = std::process::Command::new("systemctl").arg("reboot").spawn();
        }
        "shutdown" => {
          let _ = std::process::Command::new("systemctl").arg("poweroff").spawn();
        }
        "suspend" => {
          let _ = std::process::Command::new("systemctl").arg("suspend").spawn();
        }
        "hibernate" => {
          let _ = std::process::Command::new("systemctl").arg("hibernate").spawn();
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