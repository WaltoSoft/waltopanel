use adw::subclass::prelude::{ObjectImpl, ObjectImplExt, ObjectSubclass, ObjectSubclassExt};
use gtk::prelude::*;
use gtk::subclass::widget::{WidgetClassExt, WidgetImpl};
use gtk::{BinLayout, Orientation, Widget};
use gtk::glib::{self, object_subclass};
use std::cell::RefCell;

use super::PanelButtonGroup;
use crate::widgets::PanelButton;

pub struct PanelButtonGroupImp {
  pub container: gtk::Box,
  pub buttons: RefCell<Vec<PanelButton>>,
}

impl Default for PanelButtonGroupImp {
  fn default() -> Self {
    Self {
      container: gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(0)
        .build(),
      buttons: RefCell::new(Vec::new()),
    }
  }
}

#[object_subclass]
impl ObjectSubclass for PanelButtonGroupImp {
  const NAME: &'static str = "PanelButtonGroup";
  type Type = PanelButtonGroup;
  type ParentType = Widget;

  fn class_init(klass: &mut Self::Class) {
    klass.set_layout_manager_type::<BinLayout>();
    klass.set_css_name("panelbuttongroup");
  }
}

impl ObjectImpl for PanelButtonGroupImp {
  fn constructed(&self) {
    self.parent_constructed();
    self.initialize();
  }

  fn dispose(&self) {
    self.finalize();
  }
}

impl WidgetImpl for PanelButtonGroupImp {
  fn measure(&self, orientation: Orientation, for_size: i32) -> (i32, i32, i32, i32) {
    self.container.measure(orientation, for_size)
  }

  fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
    self.container.allocate(width, height, baseline, None);
  }
}

impl PanelButtonGroupImp {
  fn initialize(&self) {
    let obj = self.obj();

    self.container.set_parent(&*obj);

    obj.add_css_class("panelbuttongroup");
  }

  fn finalize(&self) {
    self.container.unparent();
  }

  pub fn add_button(&self, button: &PanelButton) {
    self.container.append(button);
    self.buttons.borrow_mut().push(button.clone());
  }

  pub fn remove_button(&self, button: &PanelButton) {
    self.container.remove(button);
    self.buttons.borrow_mut().retain(|b| b != button);
  }

  pub fn clear(&self) {
    let buttons = self.buttons.borrow().clone();
    for button in buttons {
      self.container.remove(&button);
    }
    self.buttons.borrow_mut().clear();
  }

  pub fn get_buttons(&self) -> Vec<PanelButton> {
    self.buttons.borrow().clone()
  }
}
