use crate::types::TypedListStore;
use super::MenuItemModel;

#[derive(Clone)]
pub struct MenuBuilder {
  items: TypedListStore<MenuItemModel>,
}

impl MenuBuilder {
  pub fn new() -> Self {
    Self {
      items: TypedListStore::new(),
    }
  }

  pub fn item(self, id: impl Into<String>, text: impl Into<String>) -> MenuItemBuilder {
    MenuItemBuilder {
      parent: self,
      item: Some(MenuItemModel::new(&id.into(), &text.into())),
    }
  }

  pub fn item_if(self, condition: bool, id: impl Into<String>, text: impl Into<String>) -> MenuItemBuilder {
    if condition {
      self.item(id, text)
    } else {
      MenuItemBuilder {
        parent: self,
        item: None,
      }
    }
  }

  pub fn build(self) -> TypedListStore<MenuItemModel> {
    self.items
  }
}

#[derive(Clone)]
pub struct MenuItemBuilder {
  parent: MenuBuilder,
  item: Option<MenuItemModel>,
}

impl MenuItemBuilder {
  pub fn icon(self, icon_name: impl Into<String>) -> Self {
    if let Some(ref item) = self.item {
      item.set_icon_name(Some(&icon_name.into()));
    }
    self
  }

  pub fn allow_toggle(self) -> Self {
    if let Some(ref item) = self.item {
      item.set_allow_toggle(true);
    }
    self
  }

  pub fn toggled(self, toggled: bool) -> Self {
    if let Some(ref item) = self.item {
      item.set_toggled(toggled);
      item.set_allow_toggle(true);
    }
    self
  }

  pub fn toggled_on(self) -> Self {
    if let Some(ref item) = self.item {
      item.set_toggled(true);
      item.set_allow_toggle(true);
    }
    self
  }

  pub fn separator(self) -> Self {
    if let Some(ref item) = self.item {
      item.set_separator_after(true);
    }
    self
  }

  pub fn disabled(self) -> Self {
    if let Some(ref item) = self.item {
      item.set_disabled(true);
    }
    self
  }

  pub fn disabled_if(self, condition: bool) -> Self {
    if condition {
      if let Some(ref item) = self.item {
        item.set_disabled(true);
      }
    }
    self
  }

  pub fn submenu<F>(self, builder_fn: F) -> Self
  where
    F: FnOnce(MenuBuilder) -> TypedListStore<MenuItemModel>,
  {
    if let Some(ref item) = self.item {
      let submenu_builder = MenuBuilder::new();
      let submenu = builder_fn(submenu_builder);
      item.set_submenu(submenu.as_list_store().clone());
    }
    self
  }

  pub fn item(self, id: impl Into<String>, text: impl Into<String>) -> MenuItemBuilder {
    if let Some(item) = self.item {
      self.parent.items.append(item);
    }
    self.parent.item(id, text)
  }

  pub fn item_if(self, condition: bool, id: impl Into<String>, text: impl Into<String>) -> MenuItemBuilder {
    if let Some(item) = self.item {
      self.parent.items.append(item);
    }
    if condition {
      self.parent.item(id, text)
    } else {
      MenuItemBuilder {
        parent: self.parent,
        item: None,
      }
    }
  }

  pub fn build(self) -> TypedListStore<MenuItemModel> {
    if let Some(item) = self.item {
      self.parent.items.append(item);
    }
    self.parent.items
  }
}
