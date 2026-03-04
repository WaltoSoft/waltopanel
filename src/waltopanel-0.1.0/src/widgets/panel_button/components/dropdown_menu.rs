use gtk::gdk::Key;
use gtk::glib::{object::Cast, Propagation, SignalHandlerId};
use gtk::prelude::{BoxExt, ListBoxRowExt, ListModelExt, ObjectExt, PopoverExt, WidgetExt};
use gtk::{Box, ListBox, ListBoxRow,  Popover, Widget};
use gtk::{Orientation, PositionType, SelectionMode, StateFlags};
use std::{cell::{Cell, OnceCell, RefCell}, rc::Rc};
use std::boxed::Box as StdBox;

use crate::widgets::PanelButton;
use crate::models::MenuItemModel;
use crate::traits::{CompositeWidget, ListBoxExtensions, PopoverExtensions, WidgetExtensions};
use crate::types::TypedListStore;
use super::dropdown_menu_item::DropdownMenuItem;
use super::back_button::BackButton;

#[derive(Clone)]
pub struct DropdownMenu {
  parent_panel_button: PanelButton,
  container: Popover,
  menu_data: Rc<OnceCell<TypedListStore<MenuItemModel>>>,
  current_menu: Rc<RefCell<TypedListStore<MenuItemModel>>>,
  menu_stack: Rc<RefCell<Vec<TypedListStore<MenuItemModel>>>>,
  breadcrumbs: Rc<RefCell<Vec<String>>>,
  menu_clicked_callback: Rc<OnceCell<StdBox<dyn Fn(&MenuItemModel)>>>,
  items_changed_handler: Rc<Cell<Option<SignalHandlerId>>>,
  transitioning: Rc<Cell<bool>>,
}

// Public API--------------------------------------------------------------------------------------
impl DropdownMenu {
  pub fn new(parent: &PanelButton) -> Self {
    let popover = Popover::builder()
      .autohide(true)
      .has_arrow(false)
      .position(PositionType::Bottom)
      .can_focus(true)
      .focusable(true)
      .build();

    let menu = Self {
      parent_panel_button: parent.clone(),
      container: popover,
      menu_data: Rc::new(OnceCell::new()),
      current_menu: Rc::new(RefCell::new(TypedListStore::new())),
      menu_stack: Rc::new(RefCell::new(Vec::new())),
      breadcrumbs: Rc::new(RefCell::new(Vec::new())),
      menu_clicked_callback: Rc::new(OnceCell::new()),
      items_changed_handler: Rc::new(Cell::new(None)),
      transitioning: Rc::new(Cell::new(false)),
    };

    menu.setup_popover_handlers();
    menu
  }

  pub fn set_menu(&self, menu: TypedListStore<MenuItemModel>) {
    self.menu_data.set(menu).expect("Menu can only be set once");
  }

  pub fn show_menu(&self) {
    self.reset_menu();
    self.container.update_popover_alignment();
    self.container.popup();
  }

  pub fn hide_menu(&self) {
    if self.container.is_visible() {
      self.container.popdown();
    }
  }

  pub fn toggle_visibility(&self) {
    // Guard against reentrancy during popover transitions
    if self.transitioning.get() {
      return;
    }

    self.transitioning.set(true);

    if self.container.is_visible() {
      self.hide_menu();
    } else {
      self.show_menu();
    }

    self.transitioning.set(false);
  }

  pub fn connect_menu_clicked<F>(&self, callback: F)
  where
    F: Fn(&MenuItemModel) + 'static,
  {
    self.menu_clicked_callback.set(StdBox::new(callback)).ok().expect("Menu clicked callback can only be set once");
  }
}
// End Public API----------------------------------------------------------------------------------

// Behavior Methods--------------------------------------------------------------------------------
impl DropdownMenu {
  fn reset_menu(&self) {
    if let Some(menu_data) = self.menu_data.get() {
      self.set_current_menu(menu_data.clone());
    }

    self.menu_stack.borrow_mut().clear();
    self.breadcrumbs.borrow_mut().clear();

    self.rebuild_menu();
  }

  fn set_current_menu(&self, menu: TypedListStore<MenuItemModel>) {
    // Disconnect previous handler if any
    if let Some(handler_id) = self.items_changed_handler.take() {
      self.current_menu.borrow().as_list_store().disconnect(handler_id);
    }

    // Set the new current menu
    *self.current_menu.borrow_mut() = menu.clone();

    // Connect to items_changed on the new menu
    let menu_clone = self.clone();
    let handler_id = menu.as_list_store().connect_items_changed(move |_, position, removed, added| {
      menu_clone.handle_items_changed(position, removed, added);
    });
    self.items_changed_handler.set(Some(handler_id));
  }

  fn handle_items_changed(&self, position: u32, removed: u32, added: u32) {
    // Skip updates if menu is transitioning or not visible
    if self.transitioning.get() || !self.container.is_visible() {
      return;
    }

    let Some(list_box) = self.widget().get_sub_widget::<ListBox>() else {
      return;
    };

    // Adjust position for back button if in submenu
    let row_offset = if self.in_submenu() { 1 } else { 0 };

    // Remove rows
    for _ in 0..removed {
      let row_index = (position as i32) + row_offset;
      if let Some(row) = list_box.row_at_index(row_index) {
        list_box.remove(&row);
      }
    }

    // Add new rows
    for i in 0..added {
      let model_index = position + i;
      if let Some(model) = self.current_menu().get(model_index) {
        let menu_item_row = self.build_menu_item_row(&model);
        let insert_index = (position + i) as i32 + row_offset;

        // Insert at the correct position
        if list_box.row_at_index(insert_index).is_some() {
          list_box.insert(&menu_item_row, insert_index);
        } else {
          list_box.append(&menu_item_row);
        }
      }
    }
  }

  fn rebuild_menu(&self) {
    if self.container.child().is_some() {
      self.container.set_child(Widget::NONE);
    }

    if ! self.current_menu().is_empty() {
      let menu_box  = self.create_menu();
      self.container.set_child(Some(&menu_box));
    }
  }

  fn create_menu(&self) -> Box {
    let menu_box = Box::builder()
      .orientation(Orientation::Vertical)
      .css_classes(vec!["menu"])
      .build();

    let list_box = ListBox::builder()
      .selection_mode(SelectionMode::None) // Start with no selection to prevent flash
      .css_classes(vec!["menu-list"])
      .build();
    
    if self.in_submenu() {
      let back_button_row = self.build_back_button_row();
      list_box.append_with_hover(&back_button_row);
    }  

    for menu_item in &self.current_menu() {
      let menu_item_row = self.build_menu_item_row(&menu_item);
      list_box.append_with_hover(&menu_item_row);
    }

    list_box.set_selection_mode_deferred(SelectionMode::Browse);
    menu_box.append(&list_box);

    menu_box
  }

  fn show_submenu(&self, menu_item: &MenuItemModel) {
    let submenu_items = menu_item.submenu();
    let sub_menu_label = menu_item.text().clone();
    let current_menu = self.current_menu.borrow().clone();

    self.menu_stack.borrow_mut().push(current_menu);
    self.breadcrumbs.borrow_mut().push(sub_menu_label);
    self.set_current_menu(submenu_items);

    self.rebuild_menu();
  }

  fn show_submenu_parent(&self) {
    let mut stack  = self.menu_stack.borrow_mut();

    if let Some(previous_menu) = stack.pop() {
      drop(stack);

      self.breadcrumbs.borrow_mut().pop();
      self.set_current_menu(previous_menu);

      self.rebuild_menu();
    }
  }

  fn take_menu_action(&self, model: &MenuItemModel) {
    if model.has_submenu() {
      self.show_submenu(&model);
    } else {
      if let Some(callback) = self.menu_clicked_callback.get() {
        callback(&model);
      }

      self.hide_menu();
    }
  }

  fn navigate_to_previous_panel_button(&self) {
    if let Some(prev_panel_button) = self.parent_panel_button.get_previous_instance() {
      self.hide_menu();
      prev_panel_button.show_menu();
    }
  }

  fn navigate_to_next_panel_button(&self) {
    if let Some(next_panel_button) = self.parent_panel_button.get_next_instance() {
      self.hide_menu();
      next_panel_button.show_menu();
    }
  }
}
// End Behavior Methods----------------------------------------------------------------------------

// Helper Methods----------------------------------------------------------------------------------
impl DropdownMenu {
  fn current_menu(&self) -> TypedListStore<MenuItemModel> {
    self.current_menu.borrow().clone()
  }

  fn in_submenu(&self) -> bool {
    ! self.menu_stack.borrow().is_empty()
  }
 
  fn get_back_button_label(&self) -> String {
    self.breadcrumbs.borrow().last().cloned().unwrap_or_else(|| String::from("Back"))
  }
  
  fn get_selected_menu_item_row(&self) -> Option<ListBoxRow> {
    let Some(list_box) = self.widget().get_sub_widget::<ListBox>() else {
      return None;
    };

    list_box.selected_row()
  }

  fn get_menu_model_from_row(&self, row: &ListBoxRow) -> Option<MenuItemModel> {
    let index = if self.in_submenu() {
      (row.index() - 1) as u32  // Adjust for back button row
    } else {
      row.index() as u32
    };

    self.current_menu().get(index)
  }

  fn is_back_button_row(&self, row: &ListBoxRow) -> bool {
    self.in_submenu() && row.index() == 0
  }

  fn build_back_button_row(&self) -> ListBoxRow {
    let menu_clone = self.clone();
    let back_button = BackButton::new(self.get_back_button_label());

    back_button.connect_clicked(move || menu_clone.show_submenu_parent());

    let back_button_row = ListBoxRow::builder()
      .child(back_button.widget())
      .css_classes(vec!["menu-back-row", "has-separator-after"])
      .build();

    back_button_row
  }

  fn build_menu_item_row(&self, model: &MenuItemModel) -> ListBoxRow {
    let menu_item_row = ListBoxRow::builder()
      .activatable(!model.disabled())
      .build();

    let menu_clone = self.clone();
    let model_clone = model.clone();
    let menu_item = DropdownMenuItem::new(
      model_clone);

    menu_item.connect_clicked(move |model| menu_clone.take_menu_action(model));
    menu_item_row.set_child(Some(menu_item.widget()));

    if model.separator_after() {
      menu_item_row.add_css_class("has-separator-after");
    }

    menu_item_row
  }

}
// End Helper Methods------------------------------------------------------------------------------

// Event Handler Methods---------------------------------------------------------------------------
impl DropdownMenu {
  fn handle_key_press(&self, keyval: Key) -> Propagation {
    let Some(selected_row) = self.get_selected_menu_item_row() else {
      return Propagation::Proceed;
    };
 
    match keyval {
      Key::Return | Key::KP_Enter => {
        self.handle_row_enter_key_pressed(&selected_row);
        return Propagation::Stop;
      }
      Key::Right => {
        self.handle_row_right_arrow_key_pressed(&selected_row);
        return Propagation::Stop;
      }
      Key::Left => {
        self.handle_row_left_arrow_key_pressed(&selected_row);
        return Propagation::Stop;
      }
      _ => {}
    }

    Propagation::Proceed
  }

  fn handle_row_enter_key_pressed(&self, selected_row: &ListBoxRow) {
    if self.is_back_button_row(selected_row) {
      self.show_submenu_parent();
    } else {
      if let Some(model) = self.get_menu_model_from_row(&selected_row) {
        if !model.disabled() {
          self.take_menu_action(&model);
        }
      }
    }
  }

  fn handle_row_right_arrow_key_pressed(&self, selected_row: &ListBoxRow) {
    if ! self.is_back_button_row(selected_row) {
      if let Some(model) = self.get_menu_model_from_row(&selected_row) {
        if !model.disabled() {
          if model.has_submenu() {
            self.show_submenu(&model);
          } else {
            self.navigate_to_next_panel_button();
          }
        }
      }
    }
  }

  fn handle_row_left_arrow_key_pressed(&self, selected_row: &ListBoxRow) {
    if self.is_back_button_row(selected_row) {
      self.show_submenu_parent();
    } else {
      self.navigate_to_previous_panel_button();
    }
  }

  fn setup_popover_handlers(&self) {
    let panel_button_show = self.parent_panel_button.clone();
    let panel_button_hide = self.parent_panel_button.clone();
    let menu_clone = self.clone();

    self.container.connect_show(move |_| panel_button_show.set_state_flags(StateFlags::ACTIVE, false));
    self.container.connect_hide(move |_| panel_button_hide.unset_state_flags(StateFlags::ACTIVE));
    self.container.connect_key_pressed(move |keyval| menu_clone.handle_key_press(keyval));
  }
}
// End Event Handler Methods-----------------------------------------------------------------------

impl CompositeWidget for DropdownMenu {
  fn widget(&self) -> &Widget {
    self.container.upcast_ref()
  }
}

impl std::fmt::Debug for DropdownMenu {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Menu")
      .field("parent_panel_button", &self.parent_panel_button)
      .field("popover", &self.container)
      .field("menu_data", &self.menu_data)
      .field("current_menu", &self.current_menu)
      .field("menu_stack", &self.menu_stack)
      .field("breadcrumbs", &self.breadcrumbs)
      .field("menu_clicked_callback", &"<callback>")
      .finish()
  }
}