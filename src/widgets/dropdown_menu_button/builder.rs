use crate::models::MenuItem;
use super::DropdownMenuButton;

pub struct DropdownMenuButtonBuilder {
  text: Option<String>,
  icon: Option<String>,
  menu_items: Vec<MenuItem>,
}

pub struct SubmenuBuilder<P> {
  parent: P,
  id: String,
  label: String,
  icon: Option<String>,
  submenu_items: Vec<MenuItem>,
}

impl DropdownMenuButtonBuilder {
  pub fn new() -> Self {
    Self {
      text: None,
      icon: None,
      menu_items: Vec::new(),
    }
  }

  pub fn with_text<S: Into<String>>(mut self, text: S) -> Self {
    self.text = Some(text.into());
    self
  }

  pub fn with_icon<S: Into<String>>(mut self, icon_name: S) -> Self {
    self.icon = Some(icon_name.into());
    self
  }

  pub fn with_icon_and_text<S1: Into<String>, S2: Into<String>>(
    mut self, 
    icon_name: S1, 
    text: S2
  ) -> Self {
    self.icon = Some(icon_name.into());
    self.text = Some(text.into());
    self
  }

  pub fn with_menu_items(mut self, items: Vec<MenuItem>) -> Self {
    self.menu_items = items;
    self
  }

  pub fn add_menu_item(mut self, item: MenuItem) -> Self {
    self.menu_items.push(item);
    self
  }

  pub fn add_menu_items(mut self, mut items: Vec<MenuItem>) -> Self {
    self.menu_items.append(&mut items);
    self
  }

  pub fn add_menu_with_submenu<S1: Into<String>, S2: Into<String>>(
    mut self, 
    id: S1, 
    label: S2, 
    submenu_items: Vec<MenuItem>
  ) -> Self {
    let item = MenuItem {
      id: id.into(),
      label: label.into(),
      submenu: Some(submenu_items),
      icon: None,
      is_separator: false,
      is_toggled: false,
      is_toggleable: false,
    };
    self.menu_items.push(item);
    self
  }

  pub fn start_submenu<S1: Into<String>, S2: Into<String>>(
    self, 
    id: S1, 
    label: S2
  ) -> SubmenuBuilder<Self> {
    SubmenuBuilder::new(self, id.into(), label.into())
  }

  pub fn add_separator(mut self) -> Self {
    let separator = MenuItem {
      id: String::new(),
      label: String::new(),
      submenu: None,
      icon: None,
      is_separator: true,
      is_toggled: false,
      is_toggleable: false,
    };
    self.menu_items.push(separator);
    self
  }


  pub fn build(self) -> DropdownMenuButton {
    let dropdown = DropdownMenuButton::new();

    match (self.icon.as_ref(), self.text.as_ref()) {
      (Some(icon), Some(text)) => dropdown.set_icon_and_text(icon, text),
      (Some(icon), None) => dropdown.set_icon(icon),
      (None, Some(text)) => dropdown.set_text(text),
      (None, None) => {},
    }

    if !self.menu_items.is_empty() {
      dropdown.set_menu_items(self.menu_items);
    }

    dropdown
  }
}

impl Default for DropdownMenuButtonBuilder {
  fn default() -> Self {
    Self::new()
  }
}

impl<P> SubmenuBuilder<P> 
where 
  P: HasMenuItems,
{
  fn new(parent: P, id: String, label: String) -> Self {
    Self {
      parent,
      id,
      label,
      icon: None,
      submenu_items: Vec::new(),
    }
  }

  pub fn with_icon<S: Into<String>>(mut self, icon_name: S) -> Self {
    self.icon = Some(icon_name.into());
    self
  }

  pub fn add_item(mut self, item: MenuItem) -> Self {
    self.submenu_items.push(item);
    self
  }

  pub fn add_menu_item<S1: Into<String>, S2: Into<String>>(
    mut self, 
    id: S1, 
    label: S2
  ) -> Self {
    let item = MenuItem {
      id: id.into(),
      label: label.into(),
      submenu: None,
      icon: None,
      is_separator: false,
      is_toggled: false,
      is_toggleable: false,
    };
    self.submenu_items.push(item);
    self
  }

  pub fn add_menu_item_with_icon<S1: Into<String>, S2: Into<String>, S3: Into<String>>(
    mut self, 
    id: S1, 
    label: S2,
    icon_name: S3
  ) -> Self {
    let item = MenuItem {
      id: id.into(),
      label: label.into(),
      submenu: None,
      icon: Some(icon_name.into()),
      is_separator: false,
      is_toggled: false,
      is_toggleable: false,
    };
    self.submenu_items.push(item);
    self
  }

  pub fn start_submenu<S1: Into<String>, S2: Into<String>>(
    self, 
    id: S1, 
    label: S2
  ) -> SubmenuBuilder<Self> {
    SubmenuBuilder::new(self, id.into(), label.into())
  }

  pub fn add_separator(mut self) -> Self {
    let separator = MenuItem {
      id: String::new(),
      label: String::new(),
      submenu: None,
      icon: None,
      is_separator: true,
      is_toggled: false,
      is_toggleable: false,
    };
    self.submenu_items.push(separator);
    self
  }

  pub fn end_submenu(mut self) -> P {
    let submenu_item = MenuItem {
      id: self.id,
      label: self.label,
      submenu: Some(self.submenu_items),
      icon: self.icon,
      is_separator: false,
      is_toggled: false,
      is_toggleable: false,
    };
    
    self.parent.add_menu_item_internal(submenu_item)
  }
}

pub trait HasMenuItems {
  fn add_menu_item_internal(self, item: MenuItem) -> Self;
}

impl HasMenuItems for DropdownMenuButtonBuilder {
  fn add_menu_item_internal(mut self, item: MenuItem) -> Self {
    self.menu_items.push(item);
    self
  }
}

impl<P> HasMenuItems for SubmenuBuilder<P> 
where 
  P: HasMenuItems,
{
  fn add_menu_item_internal(mut self, item: MenuItem) -> Self {
    self.submenu_items.push(item);
    self
  }
}

impl DropdownMenuButton {
  pub fn builder() -> DropdownMenuButtonBuilder {
    DropdownMenuButtonBuilder::new()
  }
}