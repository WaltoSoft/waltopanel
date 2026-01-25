use gtk::{Box, Image, Label, Widget};
use gtk::{GestureClick, Orientation};
use gtk::glib::object::{Cast, IsA, ObjectExt};
use gtk::prelude::{BoxExt, WidgetExt};

use crate::traits::CompositeWidget;
use crate::constants::*;

#[derive(Clone, Debug)]
pub struct Button {
  container: Box,
  widget_container: Box,
  click_gesture: GestureClick,
}

impl Button {
  pub fn new(parent: &impl IsA<Widget>) -> Self {
    let container = Box::builder()
      .orientation(Orientation::Horizontal)
      .spacing(PANEL_BUTTON_ICON_LABEL_SPACING)
      .build();

    let widget_container = Box::builder().visible(false).build();
    let icon_image = Image::builder().visible(false).build();
    let text_label = Label::builder().visible(false).build();

    // Set up initial custom widget if present
    let custom_widget = parent.property::<Option<Widget>>("custom-widget");
    if let Some(widget) = custom_widget {
      widget_container.append(&widget);
      widget_container.set_visible(true);
    }

    // Bind to custom-widget property changes
    let widget_container_clone = widget_container.clone();
    parent.connect_notify_local(Some("custom-widget"), move |obj, _| {
      let custom_widget = obj.property::<Option<Widget>>("custom-widget");

      // Remove any existing child
      while let Some(child) = widget_container_clone.first_child() {
        widget_container_clone.remove(&child);
      }

      // Add new widget if provided
      if let Some(widget) = custom_widget {
        widget_container_clone.append(&widget);
        widget_container_clone.set_visible(true);
      } else {
        widget_container_clone.set_visible(false);
      }
    });

    parent.bind_property("icon-name", &icon_image, "icon-name").build();
    parent.bind_property("text", &text_label, "label").build();

    parent
      .bind_property("icon-name", &icon_image, "visible")
      .transform_to(|_, icon_name: Option<String>| {
        Some(icon_name.is_some())
      })
      .build();

    parent
      .bind_property("text", &text_label, "visible")
      .transform_to(|_, text: String| {
        Some(!text.is_empty())
      })
      .build();

    let click_gesture = GestureClick::new();

    // Order: text, custom widget, icon
    container.append(&text_label);
    container.append(&widget_container);
    container.append(&icon_image);
    container.add_controller(click_gesture.clone());
    container.set_parent(parent);

    Self {
      container,
      widget_container,
      click_gesture,
    }
  }

  pub fn measure(&self, orientation: Orientation, for_size: i32) -> (i32, i32, i32, i32) {
    self.container.measure(orientation, for_size)
  } 

  pub fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
    self.container.allocate(width, height, baseline, None);
  }

  pub fn connect_clicked<F>(&self, callback: F)
  where
    F: Fn() + 'static,
  {
    self.click_gesture.connect_released(move |_, _, _, _| {
      callback();
    });
  } 
}

impl CompositeWidget for Button {
  fn widget(&self) -> Widget {
    self.container.clone().upcast()
  }
}