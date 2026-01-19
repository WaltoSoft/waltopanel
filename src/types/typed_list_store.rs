use gtk::{gio::ListStore, glib, prelude::*};
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct TypedListStore<T> {
  inner: ListStore,
  _phantom: PhantomData<T>,
}

impl<T: IsA<glib::Object>> TypedListStore<T> {
  pub fn new() -> Self {
    Self {
      inner: ListStore::new::<T>(),
      _phantom: PhantomData,
    }
  }

  pub fn from_list_store(list_store: ListStore) -> Self {
    Self {
      inner: list_store,
      _phantom: PhantomData,
    }
  }

  pub fn get(&self, index: u32) -> Option<T> {
    self.inner.item(index)?.downcast().ok()
  }

  pub fn append(&self, item: T) {
    let end = self.inner.n_items();
    self.inner.splice(end, 0, &[item.upcast::<glib::Object>()]);
  }

  pub fn insert(&self, index: u32, item: &T) {
    self.inner.splice(index, 0, &[item.clone().upcast::<glib::Object>()]);
  }

  pub fn remove(&self, index: u32) {
    self.inner.remove(index);
  }

  pub fn count(&self) -> u32 {
    self.inner.n_items()
  }

  pub fn is_empty(&self) -> bool {
    self.inner.n_items() <= 0
  }

  pub fn iter(&self) -> TypedListStoreIter<'_, T> {
    TypedListStoreIter {
      store: &self,
      index: 0,
      len: self.count(),
    }
  }

  pub fn connect_items_changed<F>(&self, callback: F)
  where
    F: Fn(&Self, u32, u32, u32) + 'static,
  {
    self.inner.connect_items_changed(move |store, position, removed, added| {
      let typed_store = TypedListStore::from_list_store(store.clone());
      callback(&typed_store, position, removed, added);
    });
  }

  pub fn as_list_store(&self) -> &ListStore {
    &self.inner
  }
}

pub struct TypedListStoreIter<'a, T> {
  store: &'a TypedListStore<T>,
  index: u32,
  len: u32,
}

impl<'a, T: IsA<glib::Object>> Iterator for TypedListStoreIter<'a, T> {
  type Item = T;

  fn next(&mut self) -> Option<T> {
    if self.index >= self.len {
      None
    } else {
      let i = self.index;
      self.index += 1;
      self.store.get(i)
    }
  }
}

// allow `for item in &store { ... }`
impl<'a, T: IsA<glib::Object>> IntoIterator for &'a TypedListStore<T> {
  type Item = T;
  type IntoIter = TypedListStoreIter<'a, T>;

  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}

