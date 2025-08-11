#[derive(Clone)]
pub struct MenuItem {
  pub id: String,
  pub label: String,
  pub icon: Option<String>,
  pub is_toggled: bool,
  pub is_separator: bool,
  pub submenu: Option<Vec<MenuItem>>,
}

impl MenuItem {
  pub fn new(id: &str, label: &str) -> Self {
    Self {
      id: id.to_string(),
      label: label.to_string(),
      icon: None,
      is_toggled: false,
      is_separator: false,
      submenu: None,
    }
  }

  pub fn with_icon(mut self, icon: &str) -> Self {
    self.icon = Some(icon.to_string());
    self
  }

  pub fn toggled(mut self) -> Self {
    self.is_toggled = true;
    self
  }

  pub fn with_submenu(mut self, submenu: Vec<MenuItem>) -> Self {
    self.submenu = Some(submenu);
    self
  }

  pub fn separator() -> Self {
    Self {
      id: String::new(),
      label: String::new(),
      icon: None,
      is_toggled: false,
      is_separator: true,
      submenu: None,
    }
  }
}