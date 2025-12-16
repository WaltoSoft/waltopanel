use adw::subclass::prelude::{ObjectImpl, ObjectImplExt, ObjectSubclass, ObjectSubclassExt};
use gtk::gio::ListStore;
use gtk::glib::value::ToValue;
use gtk::subclass::widget::{WidgetClassExt, WidgetImpl};
use gtk::{BinLayout, Orientation, Widget};
use gtk::glib::{self, ParamSpec, ParamSpecObject, ParamSpecString, Value, object_subclass};
use std::cell::{OnceCell, RefCell};

use crate::models::MenuItemModel;
use crate::traits::CompositeWidget;
use crate::types::TypedListStore;

use super::PanelButton;
use super::components::Button;
use super::components::Menu;

pub struct PanelButtonImp {
  text: RefCell<String>,
  icon_name: RefCell<Option<String>>,
  button: OnceCell::<Button>,
  menu: OnceCell::<Menu>
}

impl Default for PanelButtonImp {
  fn default() -> Self {
    Self {
      text: RefCell::new(String::new()),
      icon_name: RefCell::new(None),
      button: OnceCell::new(),
      menu: OnceCell::new()
    }
  }
}

#[object_subclass]
impl ObjectSubclass for PanelButtonImp {
  const NAME: &'static str = "PanelButton";
  type Type = PanelButton;
  type ParentType = Widget;

  fn class_init(klass: &mut Self::Class) {
    klass.set_layout_manager_type::<BinLayout>();
  }
}

impl ObjectImpl for PanelButtonImp {
  fn constructed(&self) {
    self.parent_constructed();
    self.initialize();
  }

  fn dispose(&self) {
    self.finalize();
  }

  fn properties() -> &'static [ParamSpec] {
    use std::sync::OnceLock;
    static PROPERTIES: OnceLock<Vec<ParamSpec>> = OnceLock::new();
    PROPERTIES.get_or_init(|| {
      vec![
        ParamSpecString::builder("text").build(),
        ParamSpecString::builder("icon-name").build(),
        ParamSpecObject::builder::<ListStore>("menu").build(),
      ]
    })  
  }

  fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
    match pspec.name() {
      "text" => self.text.borrow().to_value(),
      "icon-name" => self.icon_name.borrow().to_value(),
      _ => unimplemented!(),
    }
  }

  fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
    match pspec.name() {
      "text" => {
          let text = value.get().expect("type checked upstream");
          self.text.replace(text);
      }
      "icon-name" => {
          let icon_name = value.get().expect("type checked upstream");
          self.icon_name.replace(icon_name);
      }
      "menu" => {
          let list_store: ListStore = value.get().expect("type checked upstream");
          let typed_store: TypedListStore<MenuItemModel> = TypedListStore::from_list_store(list_store);
          
          if let Some(menu)= self.menu.get() {
            menu.set_menu(typed_store);
          }
      }
      _ => unimplemented!(),
    }
  }
}

impl WidgetImpl for PanelButtonImp {
  fn measure(&self, orientation: Orientation, for_size: i32) -> (i32, i32, i32, i32) {
    if let Some(button) = self.button.get() {
      button.measure(orientation, for_size)
    } else {
      (0, 0, -1, -1)
    }
  }
  
  fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
    if let Some(button) = self.button.get() {
      button.size_allocate(width, height, baseline);
    }
  }  
}

impl PanelButtonImp {
  fn initialize(&self) {
    let obj = self.obj();
    let button = Button::new(&*obj);
    let menu = Menu::new();

    menu.set_parent(&button);

    let menu_clone = menu.clone();
    button.connect_clicked(move || {
      menu_clone.toggle_visibility();
    });

    self.button.set(button.clone()).expect("Failed to set button");
    self.menu.set(menu.clone()).expect("Failed to set menu");
  }

  fn finalize(&self) {
    if let Some(button) = self.button.get() {
      button.unparent();
    }

    if let Some(menu) = self.menu.get() {
      menu.unparent();
    }
  }
}