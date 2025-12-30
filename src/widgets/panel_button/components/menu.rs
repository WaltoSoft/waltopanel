use gtk::gdk::Key;
use gtk::glib::{object::Cast, Propagation};
use gtk::prelude::{BoxExt, ListBoxRowExt, PopoverExt, WidgetExt};
use gtk::{Box, ListBox, ListBoxRow,  Popover, PositionType, Widget};
use gtk::{EventControllerKey, EventControllerMotion, Orientation, SelectionMode};
use std::{cell::{OnceCell, RefCell}, rc::Rc};
use std::boxed::Box as StdBox;

use crate::{helpers::ui_helpers, models::MenuItemModel};
use crate::traits::CompositeWidget;
use crate::types::TypedListStore;
use super::menu_item::MenuItem;
use super::back_button::BackButton;
use super::super::panel_button_api::PanelButton;

#[derive(Clone)]
pub struct Menu {
  parent_panel_button: PanelButton,
  container: Popover,
  menu_data: Rc<OnceCell<TypedListStore<MenuItemModel>>>,
  current_menu: Rc<RefCell<TypedListStore<MenuItemModel>>>,
  menu_stack: Rc<RefCell<Vec<TypedListStore<MenuItemModel>>>>,
  breadcrumbs: Rc<RefCell<Vec<String>>>,
  before_menu_show_callback: Rc<OnceCell<StdBox<dyn Fn()>>>,
  menu_clicked_callback: Rc<OnceCell<StdBox<dyn Fn(&MenuItemModel)>>>
}

// Public API--------------------------------------------------------------------------------------
impl Menu {
  pub fn new(parent: &PanelButton) -> Self {
    let popover = Popover::builder()
      .autohide(true)
      .has_arrow(false)
      .position(PositionType::Bottom)
      .can_focus(true)
      .focusable(true)
      .build();

    popover.connect_show(move |popover| handle_popover_show(popover));
    popover.connect_hide(move |popover| handle_popover_hide(popover));

    let menu = Self {
      parent_panel_button: parent.clone(),
      container: popover.clone(),
      menu_data: Rc::new(OnceCell::new()),
      current_menu: Rc::new(RefCell::new(TypedListStore::new())),
      menu_stack: Rc::new(RefCell::new(Vec::new())),
      breadcrumbs: Rc::new(RefCell::new(Vec::new())),
      before_menu_show_callback: Rc::new(OnceCell::new()),
      menu_clicked_callback: Rc::new(OnceCell::new()),
    };

    let key_controller = menu.get_menu_keypress_controller();
    popover.add_controller(key_controller);

    menu
  }

  pub fn set_menu(&self, menu: TypedListStore<MenuItemModel>) {
    self.menu_data.set(menu).expect("Menu can only be set once");
  }

  pub fn show_menu(&self) {
    let self_clone = self.clone();

    if let Some(callback) = self_clone.before_menu_show_callback.get() {
      callback();
    }

    ui_helpers::update_popover_alignment(&self.container);
    self.reset_menu();
    self.container.popup();
  }

  pub fn hide_menu(&self) {
    if self.container.is_visible() {
      self.container.popdown();
    }
  }

  pub fn toggle_visibility(&self) {
    if self.container.is_visible() {
      self.hide_menu();
    } else {
      self.show_menu();
    }
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
impl Menu {
  fn reset_menu(&self) {
    if let Some(menu_data) = self.menu_data.get() {
      *self.current_menu.borrow_mut() = menu_data.clone();
    }

    self.menu_stack.borrow_mut().clear();
    self.breadcrumbs.borrow_mut().clear();

    self.rebuild_menu();
  }

  fn rebuild_menu(&self) {
    if let Some(_) = self.container.child() {
      self.container.set_child(Widget::NONE);
    }

    if self.current_menu().is_empty() {
      return;
    }

    let menu_box  = self.create_menu();
    self.container.set_child(Some(&menu_box));
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
    
    let mut menu_items = Vec::new();

    if self.in_submenu() {
      let back_button_row = self.build_back_button_row(&list_box);
      list_box.append(&back_button_row);
    }  

    for menu_item in &self.current_menu() {
      let (menu_item_row, menu_item_widget) = self.build_menu_item_row(&list_box, &menu_item);

      list_box.append(&menu_item_row);
      menu_items.push(menu_item_widget);
    }    
    
    menu_box.append(&list_box);
    ui_helpers::defer_listbox_selection(&list_box);
    menu_box
  }

  fn show_submenu(&self, menu_item: &MenuItemModel) {
    let submenu_items = menu_item.submenu();
    let sub_menu_label = menu_item.text().clone();
    let current_menu = self.current_menu.borrow().clone();

    self.menu_stack.borrow_mut().push(current_menu);
    self.breadcrumbs.borrow_mut().push(sub_menu_label);    
    *self.current_menu.borrow_mut() = submenu_items;

    self.rebuild_menu();
  }

  fn show_submenu_parent(&self) {
    let mut stack  = self.menu_stack.borrow_mut();

    if let Some(previous_menu) = stack.pop() {
      drop(stack);

      self.breadcrumbs.borrow_mut().pop();
      *self.current_menu.borrow_mut() = previous_menu;

      self.rebuild_menu();
    }
  }
}
// End Behavior Methods----------------------------------------------------------------------------

// Helper Methods----------------------------------------------------------------------------------
impl Menu {
  fn current_menu(&self) -> TypedListStore<MenuItemModel> {
    self.current_menu.borrow().clone()
  }

  fn in_submenu(&self) -> bool {
    ! self.menu_stack.borrow().is_empty()
  }
 
  fn has_toggable_items(&self) -> bool {
    self.current_menu().iter().any(|item| item.allow_toggle())
  }

  fn has_icons(&self) -> bool {
    self.current_menu().iter().any(|item| item.icon_name().is_some())
  }

  fn get_back_button_label(&self) -> String {
    self.breadcrumbs.borrow().last().cloned().unwrap_or_else(|| "Back".to_string())
  }
  
  fn get_selected_menu_item_row(&self) -> Option<ListBoxRow> {
    let Some(list_box) = ui_helpers::get_sub_widget::<ListBox>(&self.container.upcast_ref()) else {
      return None;
    };

    list_box.selected_row()
  }

  fn get_menu_model_from_row(&self, row: &ListBoxRow) -> Option<MenuItemModel> {
    let index = if self.in_submenu() {
      (row.index() + 1) as u32  // Adjust for back button row
    } else {
      row.index() as u32
    };

    self.current_menu().get(index)
  }

  fn is_back_button_row(&self, row: &ListBoxRow) -> bool {
    self.in_submenu() && row.index() == 0
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

  fn build_back_button_row(&self, list_box: &ListBox) -> ListBoxRow {
    let back_button_row = ListBoxRow::new();
    let back_button = BackButton::new(self.get_back_button_label());
    let menu_clone = self.clone();

    back_button.connect_clicked(move || menu_clone.show_submenu_parent());
    back_button_row.set_child(Some(&back_button.widget()));
    back_button_row.add_css_class("menu-back-row");
    back_button_row.add_css_class("has-separator-after");  // Add separator after back button
    
    let motion = EventControllerMotion::new();
    let back_row_clone = back_button_row.clone();
    let list_box_clone = list_box.clone();

    motion.connect_enter(move |_, _, _| select_row(&list_box_clone, &back_row_clone));
    back_button_row.add_controller(motion);
    
    let menu_clone = self.clone();
    back_button_row.connect_activate(move |_row| {
      menu_clone.show_submenu_parent();
    });

    back_button_row
  }

  fn build_menu_item_row(&self, list_box: &ListBox, menu_item: &MenuItemModel) -> (ListBoxRow, MenuItem) {
    let menu_clone = self.clone();
    let menu_item_row = ListBoxRow::new();
    let model_clone = menu_item.clone();
    let menu_item_widget = MenuItem::new(
      model_clone, 
      self.has_toggable_items(), 
      self.has_icons(), 
      self.in_submenu());

    menu_item_widget.connect_clicked(move |model|{
      if model.has_submenu() {
        menu_clone.show_submenu(model);
      } else {
        if let Some(callback) = menu_clone.menu_clicked_callback.get() {
          callback(model);
        }
        menu_clone.hide_menu();
      }
    });

    menu_item_row.set_child(Some(&menu_item_widget.widget()));
    
    // Focus and select row on hover
    let motion = EventControllerMotion::new();
    let row_clone = menu_item_row.clone();
    let list_box_clone = list_box.clone();
    motion.connect_enter(move |_, _, _| {
      list_box_clone.select_row(Some(&row_clone));
      row_clone.grab_focus();
    });
    menu_item_row.add_controller(motion);
    
    // Connect row activation (Enter key)
    let model_clone = menu_item.clone();
    let menu_clone = self.clone();
    menu_item_row.connect_activate(move |_row| {
      if model_clone.has_submenu() {
        menu_clone.show_submenu(&model_clone);
      } else {
        if let Some(callback) = menu_clone.menu_clicked_callback.get() {
          callback(&model_clone);
        }
        menu_clone.hide_menu();
      }
    });

    if menu_item.separator_after() {
      menu_item_row.add_css_class("has-separator-after");
    }

    (menu_item_row, menu_item_widget)
  }

}
// End Helper Methods------------------------------------------------------------------------------

// Event Handler Methods---------------------------------------------------------------------------
impl Menu {
  fn handle_arrow_key_press(&self, keyval: Key) -> Propagation {
    let Some(selected_row) = self.get_selected_menu_item_row() else {
      return Propagation::Proceed;
    };
 
    match keyval {
      Key::Right => {
        self.handle_right_arrow_key(&selected_row);
        return Propagation::Stop;
      }
      Key::Left => {
        self.handle_left_arrow_key(&selected_row);
        return Propagation::Stop;
      }
      _ => {}
    }

    Propagation::Proceed
  }

  fn get_menu_keypress_controller(&self) -> EventControllerKey {
    let key_controller = EventControllerKey::new();
    let menu_clone = self.clone();
   
    key_controller.connect_key_pressed(move |_, keyval, _, _| {
      menu_clone.handle_arrow_key_press(keyval)
    });

    key_controller
  }

  fn handle_right_arrow_key(&self, selected_row: &ListBoxRow) {
    if ! self.is_back_button_row(selected_row) {
      if let Some(model) = self.get_menu_model_from_row(&selected_row) {
        if model.has_submenu() {
          self.show_submenu(&model);
        } else {
          self.navigate_to_next_panel_button();
        }
      }
    }
  }

  fn handle_left_arrow_key(&self, selected_row: &ListBoxRow) {
    if self.is_back_button_row(selected_row) {
      self.show_submenu_parent();
    } else {
      self.navigate_to_previous_panel_button();
    }
  }
}
// End Event Handler Methods-----------------------------------------------------------------------

// standalone functions----------------------------------------------------------------------------
  fn handle_popover_show(popover: &Popover) {
    if let Some(panel_button) = popover.parent().and_then(|b| b.parent()) {
      panel_button.set_state_flags(gtk::StateFlags::ACTIVE, false);
    }
  }

  fn handle_popover_hide(popover: &Popover) {
    if let Some(panel_button) = popover.parent().and_then(|b| b.parent()) {
      panel_button.unset_state_flags(gtk::StateFlags::ACTIVE);
    }
  }

  fn select_row(list_box: &ListBox, row: &ListBoxRow) {
    list_box.select_row(Some(row));
    row.grab_focus();
  }

// End standalone functions------------------------------------------------------------------------


impl CompositeWidget for Menu {
  fn widget(&self) -> Widget {
    self.container.clone().upcast()
  }
}

impl std::fmt::Debug for Menu {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Menu")
      .field("parent_panel_button", &self.parent_panel_button)
      .field("popover", &self.container)
      .field("menu_data", &self.menu_data)
      .field("current_menu", &self.current_menu)
      .field("menu_stack", &self.menu_stack)
      .field("breadcrumbs", &self.breadcrumbs)
      .field("before_menu_show_callback", &"<callback>")
      .field("menu_clicked_callback", &"<callback>")
      .finish()
  }
}