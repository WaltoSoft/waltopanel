use gtk::glib::{self, closure,object::{Cast, ObjectExt}};
use gtk::prelude::{BoxExt, GObjectPropertyExpressionExt, GridExt, ListModelExt, WidgetExt};
use gtk::{Box, ClosureExpression, GestureClick, Grid, Label, Widget};
use gtk::{Align, Orientation};
use gtk::gio::ListStore;

use crate::{constants::{ICON_SIZE, PANEL_BUTTON_MENU_ITEM_SPACING}, widgets::OptionalImage};
use crate::models::MenuItemModel;
use crate::traits::CompositeWidget;

#[derive(Clone, Debug)]
pub struct DropdownMenuItem {
  container: Box,
  model: MenuItemModel,
  click_gesture: GestureClick,
}

impl DropdownMenuItem {
  pub fn new(model: MenuItemModel) -> Self {
    let mut col = 0;

    let mut css_classes = vec!["menu-item"];
    if model.disabled() {
      css_classes.push("disabled");
    }

    let container =
      Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(css_classes)
        .focus_on_click(!model.disabled())
        .can_focus(!model.disabled())
        .focusable(!model.disabled())
        .sensitive(!model.disabled())
        .build();

    let content_grid = Grid::builder()
      .column_spacing(PANEL_BUTTON_MENU_ITEM_SPACING)
      .build();

    let label = Label::builder()
      .halign(Align::Start)
      .valign(Align::Center)
      .label(&model.text())
      .build();

    let post_label_icon = OptionalImage::new(model.post_label_icon_name().as_deref(), ICON_SIZE);

    // Wrap label and post-label icon in a horizontal box
    let label_container = Box::builder()
      .orientation(Orientation::Horizontal)
      .spacing(4)
      .hexpand(true)
      .build();
    label_container.append(&label);
    label_container.append(&post_label_icon);

    let icon_name = DropdownMenuItem::get_icon_name(model.allow_toggle(), model.toggled(), model.icon_name());
    let icon_image = OptionalImage::new(icon_name.as_deref(), ICON_SIZE);
    let submenu_icon_name = DropdownMenuItem::get_submenu_icon_name(model.submenu().count());
    let submenu_icon_image = OptionalImage::new(submenu_icon_name.as_deref(), ICON_SIZE);

    model.bind_property("text", &label, "label").build();
    model.bind_property("post-label-icon-name", &post_label_icon, "icon-name").build();

    let allowed_toggle_expression = model.property_expression("allow-toggle");
    let toggled_expression = model.property_expression("toggled");
    let icon_name_expression = model.property_expression("icon-name");

    let combined = ClosureExpression::new::<Option<String>>(
      [allowed_toggle_expression, toggled_expression, icon_name_expression],
      closure!(|_: glib::Object, allow_toggle: bool, toggled: bool, icon_name: Option<String>| -> Option<String> {
        DropdownMenuItem::get_icon_name(allow_toggle, toggled, icon_name)
      }),
    );

    combined.bind(&icon_image, "icon-name", Some(&model));

    model
      .bind_property("submenu", &submenu_icon_image, "icon-name")
      .transform_to(|_, submenu: ListStore| {
        DropdownMenuItem::get_submenu_icon_name(submenu.n_items())
      })
      .build();


    content_grid.attach(&icon_image, col, 0, 1, 1);
    col += 1;

    content_grid.attach(&label_container, col, 0, 1, 1);
    col += 1;

    content_grid.attach(&submenu_icon_image, col, 0, 1, 1);

    let click_gesture = GestureClick::new();

    container.append(&content_grid);
    container.add_controller(click_gesture.clone());

    Self {
      container,
      model,
      click_gesture,
    }
  }

  fn get_icon_name(allow_toggle: bool, toggled: bool, icon_name: Option<String>) -> Option<String> {
    if allow_toggle {
      if toggled {
        Some("object-select-symbolic".to_string())
      } else {
        None
      }
    } else {
      icon_name
    }
  }

  fn get_submenu_icon_name(submenu_count: u32) -> Option<String> {
    if submenu_count > 0 {
      Some("go-next-symbolic".to_string())
    } else {
      None
    }
  }


  pub fn connect_clicked<F>(&self, callback: F)
  where
    F: Fn(&MenuItemModel) + 'static,
  {
    let model = self.model.clone();

    self.click_gesture.connect_released(move |_, _, _, _| {
      if !model.disabled() {
        callback(&model);
      }
    });
  }   
}

impl CompositeWidget for DropdownMenuItem {
  fn widget(&self) -> Widget {
    self.container.clone().upcast()
  }
}