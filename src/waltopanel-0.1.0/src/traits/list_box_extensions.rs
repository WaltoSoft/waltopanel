use gtk::glib;
use gtk::{EventControllerMotion, ListBox, ListBoxRow, SelectionMode};
use gtk::prelude::WidgetExt;

pub trait ListBoxExtensions {
  fn append_with_hover(&self, row: &ListBoxRow);
  fn set_selection_mode_deferred(&self, selection_mode: SelectionMode);
}

impl ListBoxExtensions for ListBox {
  fn append_with_hover(&self, row: &ListBoxRow) {
    self.append(row);

    let motion = EventControllerMotion::new();
    let self_clone = self.clone();
    let row_clone = row.clone();

    motion.connect_enter(move |_, _, _| {
      self_clone.select_row(Some(&row_clone));
      row_clone.grab_focus();
    });

    row.add_controller(motion);
  }

  fn set_selection_mode_deferred(&self, selection_mode: SelectionMode) {
    let list_box_clone = self.clone();

    glib::idle_add_local_once(move || {
      list_box_clone.set_selection_mode(selection_mode);
      list_box_clone.unselect_all();
      list_box_clone.grab_focus();
    
      if let Some(first_row) = list_box_clone.row_at_index(0) {
        list_box_clone.select_row(Some(&first_row));
        first_row.grab_focus();
      }
    });
  }  
}