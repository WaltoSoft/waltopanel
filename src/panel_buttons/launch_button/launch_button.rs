use gtk::{Widget, glib::object::Cast};

use crate::{traits::CompositeWidget, util::process, widgets::PanelButton};

pub struct LaunchButton{
  panel_button: PanelButton,
}

impl LaunchButton {
  pub fn from_icon_name<T: Into<String>>(icon_name: &str, launch_command: T) -> Self {
    let panel_button = PanelButton::from_icon_name(icon_name);
    let command = launch_command.into();

    panel_button.connect_button_clicked(move |_| {
      process::spawn_detached(&command);
    });

    LaunchButton{
      panel_button,
    }
  }
}

impl CompositeWidget for LaunchButton {
  fn widget(&self) -> Widget {
    self.panel_button.clone().upcast()
  }
}