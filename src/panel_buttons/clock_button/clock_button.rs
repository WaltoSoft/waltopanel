use chrono::Local;
use gtk::glib;
use gtk::Widget;
use gtk::glib::object::Cast;


use crate::{traits::CompositeWidget, widgets::{PanelButton, PanelButtonBuilder}};

#[derive(Clone, Debug)]
pub struct ClockButton{
  panel_button: PanelButton,
}

impl ClockButton {
  pub fn new() -> Self {
    let time = Self::get_time(); 
    let panel_button = PanelButtonBuilder::new()
      .text(&time)
      .build();

    let obj = Self {
      panel_button,
    };  

    let obj_clone = obj.clone();

    glib::timeout_add_seconds_local(1, move || {
      let time = Self::get_time();
      obj_clone.panel_button.set_text(&time);
      glib::ControlFlow::Continue
    });

    obj
  }

  fn get_time() -> String {
    let now = Local::now();
    now.format("%b %-d, %Y %-I:%M %p").to_string()
  }
}

impl CompositeWidget for ClockButton {
  fn widget(&self) -> Widget {
    self.panel_button.clone().upcast()
  }
}