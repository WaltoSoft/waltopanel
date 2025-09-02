use std::cell::RefCell;
use crate::models::MenuItem;

#[derive(Clone)]
pub struct MenuState {
  pub menu_items: RefCell<Vec<MenuItem>>,
  pub menu_boxes: RefCell<Vec<gtk::Box>>,
  pub sub_menu_stack: RefCell<Vec<Vec<MenuItem>>>,
  pub sub_menu_breadcrumbs: RefCell<Vec<String>>,
}

impl MenuState {
  pub fn new() -> Self {
    Self {
      menu_items: RefCell::new(Vec::new()),
      menu_boxes: RefCell::new(Vec::new()),
      sub_menu_stack: RefCell::new(Vec::new()),
      sub_menu_breadcrumbs: RefCell::new(Vec::new()),
    }
  }
    
  pub fn reset_navigation(&self) {
    self.sub_menu_stack.borrow_mut().clear();
    self.sub_menu_breadcrumbs.borrow_mut().clear();
  }
  
  pub fn is_in_submenu(&self) -> bool {
    !self.sub_menu_stack.borrow().is_empty()
  }
}

