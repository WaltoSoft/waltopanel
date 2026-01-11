use gtk::glib::object::Cast;
use gtk::prelude::{BoxExt, GridExt, WidgetExt};
use gtk::{Box, GestureClick, Grid, Label, Widget}; 
use gtk::{Align, Orientation};

use crate::constants::{ICON_SIZE, PANEL_BUTTON_MENU_ITEM_SPACING};
use crate::models::MenuItemModel;
use crate::traits::{CompositeWidget, WidgetExtensions};

#[derive(Clone, Debug)]
pub struct DropdownMenuItem {
  container: Box,
  model: MenuItemModel,
  click_gesture: GestureClick,
}

impl DropdownMenuItem {
  pub fn new(model: MenuItemModel, menu_has_toggable_items: bool, menu_has_icons: bool, menu_is_submenu: bool) -> Self {

    let mut col = 0;

    let container = 
      Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(vec!["menu-item"])
        .focus_on_click(true)
        .can_focus(true)
        .focusable(true)
        .build();

    let content_grid = Grid::builder()
      .column_spacing(PANEL_BUTTON_MENU_ITEM_SPACING)
      .build();

    let label = Label::builder()
      .label(model.text())
      .halign(Align::Start)
      .hexpand(true)
      .valign(Align::Center)
      .build();    

    let icon_widget = Widget::create_icon_widget(
      model.icon_name(), 
      ICON_SIZE);

    let toggled_icon = if menu_has_toggable_items && model.toggled() {
      Some("object-select-symbolic")
    } else {
      None
    };

    if menu_has_toggable_items {
      let toggled_icon_widget = Widget::create_icon_widget(toggled_icon, ICON_SIZE);
      content_grid.attach(&toggled_icon_widget, col, 0, 1, 1);
      col += 1;
    }

    if menu_has_icons {
      content_grid.attach(&icon_widget, col, 0, 1, 1);
      col += 1;
    }

    if ! menu_has_icons && ! menu_has_toggable_items && menu_is_submenu {
      let blank_icon_widget = Widget::create_icon_widget(None::<String>, ICON_SIZE);
      content_grid.attach(&blank_icon_widget, col, 0, 1, 1);
      col += 1;
    }

    content_grid.attach(&label, col, 0, 1, 1);
    col += 1;

    if model.has_submenu() {
      let arrow_icon_widget = Widget::create_icon_widget(Some("go-next-symbolic"), ICON_SIZE);
      content_grid.attach(&arrow_icon_widget, col, 0, 1, 1);
    } 

    let click_gesture = GestureClick::new();

    container.append(&content_grid);
    container.add_controller(click_gesture.clone());
    
    Self {
      container,
      model,
      click_gesture,
    }
  }

  pub fn connect_clicked<F>(&self, callback: F)
  where
    F: Fn(&MenuItemModel) + 'static,
  {
    let model = self.model.clone();

    self.click_gesture.connect_released(move |_, _, _, _| {
      callback(&model);
    });    
  }   
}

impl CompositeWidget for DropdownMenuItem {
  fn widget(&self) -> Widget {
    self.container.clone().upcast()
  }
}