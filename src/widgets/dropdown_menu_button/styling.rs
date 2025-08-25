use gtk::prelude::*;

pub struct DropdownStyling;

impl DropdownStyling {
  /// Apply CSS classes and styling to the dropdown button
  pub fn setup_button_styling(button: &gtk::Button) {
    // Add base CSS classes
    button.add_css_class("menu-button");
  }
    
  /// Apply CSS classes to the popover menu container
  pub fn setup_popover_styling(popover: &gtk::Popover) {
    // CSS classes for the popover itself if needed
  }
    
  /// Create and style a menu container box
  pub fn create_styled_menu_container() -> gtk::Box {
    gtk::Box::builder()
      .orientation(gtk::Orientation::Vertical)
      .spacing(0)
      .css_classes(vec!["dropdown-menu".to_string()])
      .build()
  }
    
  /// Create and style a menu item container
  pub fn create_styled_menu_item() -> gtk::Box {
    gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .css_classes(vec!["dropdown-item".to_string()])
      .build()
  }
    
  /// Create a styled separator
  pub fn create_styled_separator() -> gtk::Separator {
    gtk::Separator::builder()
      .orientation(gtk::Orientation::Horizontal)
      .css_classes(vec!["dropdown-separator".to_string()])
      .build()
  }
  
  /// Set up hover behavior for menu items
  pub fn setup_hover_styling(item_container: &gtk::Box) {
    let motion_controller = gtk::EventControllerMotion::new();
    
    let item_enter = item_container.clone();
    motion_controller.connect_enter(move |_, _, _| {
      item_enter.add_css_class("focused");
    });
    
    let item_leave = item_container.clone();
    motion_controller.connect_leave(move |_| {
      item_leave.remove_css_class("focused");
    });
    
    item_container.add_controller(motion_controller);
  }
  
  /// Update visual focus state
  pub fn update_visual_focus(containers: &[gtk::Box], focused_index: Option<usize>) {
    for (idx, container) in containers.iter().enumerate() {
      if Some(idx) == focused_index {
        container.add_css_class("focused");
      } else {
        container.remove_css_class("focused");
      }
    }
  }
  
  /// Set active state on button
  pub fn set_button_active(button: &gtk::Button, active: bool) {
    if active {
      button.add_css_class("menu-button-active");
    } else {
      button.remove_css_class("menu-button-active");
    }
  }
  
  /// Mark item as toggled
  pub fn set_item_toggled(item_container: &gtk::Box, toggled: bool) {
    if toggled {
      item_container.add_css_class("toggled");
    } else {
      item_container.remove_css_class("toggled");
    }
  }
}