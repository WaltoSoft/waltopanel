mod builder;
mod handlers;
mod instances;
mod navigation;
mod properties;
mod signals;
mod state;
mod styling;
mod ui;
mod utils;

use gtk::{glib::{self, subclass::Signal}, prelude::*, subclass::prelude::* };
use gtk::{BinLayout, Button, Popover, Widget};
use gtk::{Orientation};
use std::cell::OnceCell;

use state::MenuState;

pub struct DropdownMenuButtonPrivate {
  dropdown_menu_button: OnceCell<Button>,
  popover: OnceCell<Popover>,
  state: MenuState,
}

impl Default for DropdownMenuButtonPrivate {
  fn default() -> Self {
    Self {
      dropdown_menu_button: OnceCell::new(),
      popover: OnceCell::new(),
      state: MenuState::new(),
    }
  }
}

#[glib::object_subclass]
impl ObjectSubclass for DropdownMenuButtonPrivate {
  const NAME: &'static str = "DropdownMenuButton";
  type Type = super::DropdownMenuButton;
  type ParentType = Widget;
  
  fn class_init(klass: &mut Self::Class) {
    klass.set_layout_manager_type::<BinLayout>();
  }
}

impl ObjectImpl for DropdownMenuButtonPrivate {
  fn constructed(&self) {
    self.parent_constructed();
    self.initialize();
  }
    
  fn dispose(&self) {
    self.finalize();
  }
  
  fn signals() -> &'static [Signal] {
    Self::setup_signals()
  }
}

impl WidgetImpl for DropdownMenuButtonPrivate {
  fn measure(&self, orientation: Orientation, for_size: i32) -> (i32, i32, i32, i32) {
    if let Some(button) = self.dropdown_menu_button.get() {
      button.measure(orientation, for_size)
    } else {
      (0, 0, -1, -1)
    }
  }
  
  fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
    if let Some(button) = self.dropdown_menu_button.get() {
      button.allocate(width, height, baseline, None);
    }
  }
}