use adw::subclass::prelude::{ObjectImpl, ObjectImplExt, ObjectSubclass, ObjectSubclassExt, ObjectSubclassIsExt};
use gtk::gio::ListStore;
use gtk::glib::object::ObjectExt;
use gtk::glib::subclass::Signal;
use gtk::glib::types::StaticType;
use gtk::glib::value::ToValue;
use gtk::prelude::WidgetExt;
use gtk::subclass::widget::{WidgetClassExt, WidgetImpl};
use gtk::{BinLayout, Orientation, Widget};
use gtk::glib::{self, ParamSpec, ParamSpecObject, ParamSpecString, Value, object_subclass};
use std::cell::{OnceCell, RefCell};
use std::sync::OnceLock;
use uuid::Uuid;

use crate::models::MenuItemModel;
use crate::traits::CompositeWidget;
use crate::types::TypedListStore;
use super::PanelButton;
use super::components::Button;
use super::components::DropdownComponent;
use super::components::DropdownMenu;

pub struct PanelButtonImp {
  pub id: Uuid,
  text: RefCell<String>,
  icon_name: RefCell<Option<String>>,
  custom_widget: RefCell<Option<Widget>>,
  dropdown_widget: RefCell<Option<Widget>>,
  button: OnceCell<Button>,
  menu: OnceCell<DropdownMenu>,
  dropdown_component: OnceCell<DropdownComponent>,
}

impl Default for PanelButtonImp {
  fn default() -> Self {
    Self {
      id: Uuid::new_v4(),
      text: RefCell::new(String::new()),
      icon_name: RefCell::new(None),
      custom_widget: RefCell::new(None),
      dropdown_widget: RefCell::new(None),
      button: OnceCell::new(),
      menu: OnceCell::new(),
      dropdown_component: OnceCell::new(),
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
    klass.set_css_name("panelbutton");
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
    static PROPERTIES: OnceLock<Vec<ParamSpec>> = OnceLock::new();
    PROPERTIES.get_or_init(|| {
      vec![
        ParamSpecString::builder("text").build(),
        ParamSpecString::builder("icon-name").build(),
        ParamSpecObject::builder::<Widget>("custom-widget").build(),
        ParamSpecObject::builder::<Widget>("dropdown-widget").build(),
        ParamSpecObject::builder::<ListStore>("menu").build(),
      ]
    })
  }

  fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
    match pspec.name() {
      "text" => self.text.borrow().to_value(),
      "icon-name" => self.icon_name.borrow().to_value(),
      "custom-widget" => self.custom_widget.borrow().to_value(),
      "dropdown-widget" => self.dropdown_widget.borrow().to_value(),
      "menu" => {
        // Return an empty ListStore since menu is write-only
        ListStore::new::<MenuItemModel>().to_value()
      }
      _ => unimplemented!(),
    }
  }

  fn signals() -> &'static [Signal] {
    Self::setup_signals()
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
      "custom-widget" => {
          let widget: Option<Widget> = value.get().expect("type checked upstream");
          self.custom_widget.replace(widget.clone());
      }
      "dropdown-widget" => {
          let widget: Option<Widget> = value.get().expect("type checked upstream");
          self.dropdown_widget.replace(widget.clone());

          // Create dropdown component if it doesn't exist yet
          if self.dropdown_component.get().is_none() {
            self.create_dropdown_component();
          }

          // Set the widget in the dropdown component
          if let Some(dropdown) = self.dropdown_component.get() {
            dropdown.set_widget(widget.as_ref());
          }
      }
      "menu" => {
          let list_store: ListStore = value.get().expect("type checked upstream");
          let typed_store: TypedListStore<MenuItemModel> = TypedListStore::from_list_store(list_store);

          // Create menu if it doesn't exist yet
          if self.menu.get().is_none() {
            self.create_menu();
          }

          if let Some(menu) = self.menu.get() {
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

    obj.add_css_class("panelbutton");

    button.connect_clicked(glib::clone!(@weak obj => move || {
      let imp = obj.imp();
      if let Some(menu) = imp.menu.get() {
        PanelButton::close_other_instances(&obj);
        menu.toggle_visibility();
      } else if let Some(dropdown) = imp.dropdown_component.get() {
        PanelButton::close_other_instances(&obj);
        dropdown.toggle_visibility();
      } else {
        obj.emit_by_name::<()>("button-clicked", &[]);
      }
    }));

    self.button.set(button.clone()).expect("Failed to set button");

    PanelButton::register_instance(&*obj);
  }

  fn create_menu(&self) {
    let obj = self.obj();
    let menu = DropdownMenu::new(&obj);

    if let Some(button) = self.button.get() {
      menu.set_parent(button);
    }

    let obj_clone = obj.clone();
    menu.connect_menu_clicked(move |model| {
      println!("Menu item clicked: {}", model.text());
      obj_clone.emit_by_name::<()>("menu-item-clicked", &[&model]);
    });

    self.menu.set(menu).expect("Failed to set menu");
  }

  fn create_dropdown_component(&self) {
    let obj = self.obj();
    let dropdown = DropdownComponent::new(&obj);

    if let Some(button) = self.button.get() {
      dropdown.set_parent(button);
    }

    self.dropdown_component.set(dropdown).expect("Failed to set dropdown component");
  }

  fn finalize(&self) {
    if let Some(button) = self.button.get() {
      button.unparent();
    }
  }

  pub fn show_menu(&self) {
    if let Some(menu) = self.menu.get() {
      PanelButton::close_other_instances(&self.obj());
      menu.show_menu();
    } else if let Some(dropdown) = self.dropdown_component.get() {
      PanelButton::close_other_instances(&self.obj());
      dropdown.show();
    }
  }

  pub fn hide_menu(&self) {
    if let Some(menu) = self.menu.get() {
      menu.hide_menu();
    }
    if let Some(dropdown) = self.dropdown_component.get() {
      dropdown.hide();
    }
  }

  fn setup_signals() -> &'static [Signal] {
    static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
    SIGNALS.get_or_init(|| {
      vec![
        Signal::builder("button-clicked")
          .build(),
        Signal::builder("menu-item-clicked")
          .param_types([MenuItemModel::static_type()])
          .build(),
      ]
    })
  }
}