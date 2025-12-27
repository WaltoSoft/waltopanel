use gtk::glib::object::Cast;
use gtk::prelude::{BoxExt, EventControllerExt, WidgetExt};
use gtk::{Box, GestureClick, Grid, Label, Widget}; 
use gtk::{Align, Orientation, prelude::GridExt};

use crate::models::MenuItemModel;
use crate::helpers::ui_helpers;
use crate::traits::CompositeWidget;

#[derive(Clone)]
pub struct MenuItem {
  container: Box,
  model: MenuItemModel,
  click_gesture: GestureClick
}

impl MenuItem {
  pub fn new(model: MenuItemModel, menu_has_toggable_items: bool, menu_has_icons: bool, menu_is_submenu: bool) -> Self {
    let mut col = 0;
    let icon_size = 16;
    let column_spacing = 12;

    let container = ui_helpers::create_styled_box(Orientation::Horizontal, 0, vec!["menu-item".to_string()]);
    container.set_focusable(true);
    container.set_can_focus(true);

    let content_grid = Grid::builder().column_spacing(column_spacing).build();
    let icon_widget = ui_helpers::create_icon_widget(model.icon_name(), icon_size);

    let toggled_icon = if menu_has_toggable_items && model.toggled() {
      Some("object-select-symbolic".to_string())
    } else {
      None
    };

    if menu_has_toggable_items {
      let toggled_icon_widget = ui_helpers::create_icon_widget(toggled_icon, icon_size);  
      content_grid.attach(&toggled_icon_widget, col, 0, 1, 1);
      col += 1;
    }

    if menu_has_icons {
      content_grid.attach(&icon_widget, col, 0, 1, 1);
      col += 1;
    }

    if ! menu_has_icons && ! menu_has_toggable_items && menu_is_submenu {
      let blank_icon_widget = ui_helpers::create_icon_widget(None, icon_size);
      content_grid.attach(&blank_icon_widget, col, 0, 1, 1);
      col += 1;
    }

    let label = Label::builder()
      .label(model.text())
      .halign(Align::Start)
      .hexpand(true)
      .build();    

    content_grid.attach(&label, col, 0, 1, 1);
    col += 1;

    if model.has_submenu() {
      let sub_menu_icon = Some("go-next-symbolic".to_string());
      let arrow_icon_widget = ui_helpers::create_icon_widget(sub_menu_icon,icon_size);
      content_grid.attach(&arrow_icon_widget, col, 0, 1,1);
    } 

    container.append(&content_grid);
    let click_gesture = GestureClick::new();
    container.add_controller(click_gesture.clone());
    
    Self {
      container,
      model,
      click_gesture
    }
  }

  pub fn connect_clicked<F>(&self, callback: F)
  where
    F: Fn(&MenuItemModel) + 'static,
  {
    let model = self.model.clone();

    self.click_gesture.connect_released(move |gesture, _, _, _| {
      if let Some(widget) = gesture.widget() {
        widget.grab_focus();
      }
      
      callback(&model);
    });    

  }   
}

impl CompositeWidget for MenuItem {
  fn widget(&self) -> Widget {
    self.container.clone().upcast()
  }
}