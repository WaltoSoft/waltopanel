use gtk::glib::{self, object::ObjectExt};

use super::imp::OptionalImageImp;

glib::wrapper! {
  pub struct OptionalImage(ObjectSubclass<OptionalImageImp>)
    @extends gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl OptionalImage {
  pub fn new(icon_name: Option<&str>, icon_size: i32) -> Self {
    let obj: OptionalImage = glib::Object::new();
    obj.set_icon_name(icon_name);
    obj.set_icon_size(icon_size);
    obj
  }

  pub fn set_icon_name(&self, icon_name: Option<&str>) {
    self.set_property("icon-name", icon_name);
  }

  pub fn icon_name(&self) -> Option<String> {
    self.property("icon-name")
  }

  pub fn set_icon_size(&self, icon_size: i32) {
    self.set_property("icon-size", icon_size);
  }

  pub fn icon_size(&self) -> i32 {
    self.property("icon-size")
  }
}

