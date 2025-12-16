use adw::subclass::prelude::{ObjectImpl, ObjectImplExt, ObjectSubclass, ObjectSubclassExt};
use gtk::{Orientation, PositionType, glib::{self, object_subclass}, prelude::{BoxExt, WidgetExt}, subclass::widget::{WidgetClassExt, WidgetImpl}};
use gtk::{BinLayout, Box, Image, Label, Popover, Widget};
use std::cell::{OnceCell,RefCell};

use crate::{models::MenuItemModel, types::TypedListStore};
use super::ButtonMenu;

pub struct ButtonMenuPrivate {
  //Component State
  pub (super) icon_name: RefCell<Option<String>>,
  pub (super) text: RefCell<Option<String>>,

  //Menu State
  pub (super) menu_data: OnceCell<TypedListStore<MenuItemModel>>,
  pub (super) current_menu: RefCell<TypedListStore<MenuItemModel>>,
  pub (super) sub_menu_stack: RefCell<Vec<TypedListStore<MenuItemModel>>>,
  pub (super) breadcrumbs: RefCell<Vec<String>>,

  //Widget references
  pub (super) button_menu_box: OnceCell<Box>,
  pub (super) popover: OnceCell<Popover>,
  pub (super) icon_image: OnceCell<Image>,
  pub (super) text_label: OnceCell<Label>,
}

impl Default for ButtonMenuPrivate {
  fn default() -> Self {
    Self {
      icon_name: RefCell::new(None),
      text: RefCell::new(None),

      menu_data: OnceCell::new(),
      current_menu: RefCell::new(TypedListStore::new()),
      sub_menu_stack: RefCell::new(Vec::new()),
      breadcrumbs: RefCell::new(Vec::new()),

      button_menu_box: OnceCell::new(),
      popover: OnceCell::new(),
      icon_image: OnceCell::new(),
      text_label: OnceCell::new(),
   
    }
  }
}

#[object_subclass]
impl ObjectSubclass for ButtonMenuPrivate {
  const NAME: &'static str = "ButtonMenu";
  type Type = ButtonMenu;
  type ParentType = Widget;

  fn class_init(klass: &mut Self::Class) {
    klass.set_layout_manager_type::<BinLayout>();
  }
}

impl ObjectImpl for ButtonMenuPrivate {
  fn constructed(&self) {
    self.parent_constructed();
    self.initialize();
  }

  fn dispose(&self) {
    self.finalize();
  }
}

impl WidgetImpl for ButtonMenuPrivate {
  fn measure(&self, orientation: Orientation, for_size: i32) -> (i32, i32, i32, i32) {
    if let Some(button_menu_box) = self.button_menu_box.get() {
      button_menu_box.measure(orientation, for_size)
    } else {
      (0, 0, -1, -1)
    }
  }
  
  fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
    if let Some(button_menu_box) = self.button_menu_box.get() {
      button_menu_box.allocate(width, height, baseline, None);
    }
  }
}

impl ButtonMenuPrivate {
  fn initialize(&self) {
    let obj = self.obj();
    let button_menu_box = Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .spacing(10)
      .build();

    let image = Image::new();
    self.icon_image.set(image.clone()).expect("Failed to set icon_image");
    self.refresh_icon_image();

    let label = Label::new(None);
    self.text_label.set(label.clone()).expect("Failed to set text_label");
    self.refresh_text_label();

    button_menu_box.append(&image);
    button_menu_box.append(&label);
    button_menu_box.set_parent(&*obj);

    let popover = Popover::builder()
      .autohide(true)
      .has_arrow(false)
      .position(PositionType::Bottom)
      .can_focus(true)
      .focusable(true)
      .build();

    popover.set_parent(&button_menu_box);

    self.button_menu_box.set(button_menu_box.clone()).expect("Failed to set button_menu_box");
    self.popover.set(popover).expect("Failed to set popover");
    self.attach_button_menu_handlers();

    Self::register_instance(&*obj);

  }

  fn finalize(&self) {
    if let Some(menu_box) = self.button_menu_box.get() {
      menu_box.unparent();
    }

    if let Some(popover) = self.popover.get() {
      popover.unparent();
    } 
  }

  pub fn refresh_icon_image(&self) {
    if let Some(image) = self.icon_image.get() {
      if let Some(icon_name) = self.icon_name.borrow().as_ref() {
        image.set_icon_name(Some(icon_name));
        image.set_visible(true);
      } else {
          image.set_visible(false);
      } 
    }
  }

  pub fn refresh_text_label(&self) {
    if let Some(label) = self.text_label.get() {
      if let Some(text) = self.text.borrow().as_ref() {
        label.set_text(text);
        label.set_visible(true);
      } else {
          label.set_visible(false);
      } 
    }
  }  
}
