use std::cell::{RefCell, Cell};
use crate::models::MenuItem;

pub struct MenuState {
  pub menu_items: RefCell<Vec<MenuItem>>,
  pub menu_boxes: RefCell<Vec<gtk::Box>>,
  pub focused_item_index: RefCell<Option<usize>>,
  pub sub_menu_stack: RefCell<Vec<Vec<MenuItem>>>,
  pub sub_menu_breadcrumbs: RefCell<Vec<String>>,
  pub is_open: Cell<bool>,
  pub is_active: Cell<bool>,
}

impl MenuState {
  pub fn new() -> Self {
    Self {
      menu_items: RefCell::new(Vec::new()),
      menu_boxes: RefCell::new(Vec::new()),
      focused_item_index: RefCell::new(None),
      sub_menu_stack: RefCell::new(Vec::new()),
      sub_menu_breadcrumbs: RefCell::new(Vec::new()),
      is_open: Cell::new(false),
      is_active: Cell::new(false),
    }
  }
    
  pub fn reset_navigation(&self) {
    *self.focused_item_index.borrow_mut() = None;
    self.sub_menu_stack.borrow_mut().clear();
    self.sub_menu_breadcrumbs.borrow_mut().clear();
  }
  
  pub fn is_in_submenu(&self) -> bool {
    !self.sub_menu_stack.borrow().is_empty()
  }
}

