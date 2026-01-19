use adw::subclass::prelude::{ObjectImpl, ObjectImplExt, ObjectSubclass, ObjectSubclassExt};
use gtk::glib::object::ObjectExt;
use gtk::prelude::{BoxExt, WidgetExt};
use gtk::subclass::widget::{WidgetClassExt, WidgetImpl};
use gtk::{BinLayout, Box, Image, Orientation, Widget};
use gtk::glib::{self, ParamSpec, ParamSpecInt, ParamSpecString, Value, object_subclass};
use gtk::glib::value::ToValue;
use std::cell::{OnceCell, RefCell};
use std::sync::OnceLock;

use super::OptionalImage;

pub struct OptionalImageImp {
  icon_name: RefCell<Option<String>>,
  icon_size: RefCell<i32>,
  container: OnceCell<Box>,
}

impl Default for OptionalImageImp {
  fn default() -> Self {
    Self {
      icon_name: RefCell::new(None),
      icon_size: RefCell::new(16),
      container: OnceCell::new(),
    }
  }
}

#[object_subclass]
impl ObjectSubclass for OptionalImageImp {
  const NAME: &'static str = "OptionalImage";
  type Type = OptionalImage;
  type ParentType = Widget;

  fn class_init(klass: &mut Self::Class) {
    klass.set_layout_manager_type::<BinLayout>();
    klass.set_css_name("optionalimage");
  }
}

impl ObjectImpl for OptionalImageImp {
  fn constructed(&self) {
    self.parent_constructed();
    self.initialize();
  }

  fn dispose(&self) {
    self.finalize();
  }

  fn properties() -> &'static [ParamSpec] {
    static PROPERTIES: OnceLock<Vec<ParamSpec>> = OnceLock::new();
    PROPERTIES.get_or_init(|| {
      vec![
        ParamSpecString::builder("icon-name").build(),
        ParamSpecInt::builder("icon-size").build(),
      ]
    })
  }

  fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
    match pspec.name() {
      "icon-name" => self.icon_name.borrow().to_value(),
      "icon-size" => self.icon_size.borrow().to_value(),
      _ => unimplemented!(),
    }
  }

  fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
    match pspec.name() {
      "icon-name" => {
        let icon_name: Option<String> = value.get().expect("type checked upstream");
        self.icon_name.replace(icon_name.clone());
      }
      "icon-size" => {
        let icon_size: i32 = value.get().expect("type checked upstream");
        self.icon_size.replace(icon_size);
      }
      _ => unimplemented!(),
    }
  }
}

impl WidgetImpl for OptionalImageImp {}


impl OptionalImageImp {
  fn initialize(&self) {
    let obj = self.obj();
    let container = Box::new(Orientation::Horizontal, 0);
    let icon_image = Image::new();
    container.append(&icon_image);
    container.set_parent(&*obj);

    obj.bind_property("icon-name", &icon_image, "icon-name").build();
    obj.bind_property("icon-size", &icon_image, "pixel-size").build();
    obj.bind_property("icon-size", &container, "width-request").build();
    obj.bind_property("icon-size", &container, "height-request").build();

    self.container.set(container.clone()).expect("Failed to set container");

  }

  fn finalize(&self) {
    if let Some(container) = self.container.get() {
      container.unparent();
    }
  }
}
