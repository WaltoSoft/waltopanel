use adw::subclass::prelude::ObjectSubclassIsExt;
use gtk::prelude::WidgetExt;
use indexmap::IndexMap;
use uuid::Uuid;
use std::cell::RefCell;

use crate::widgets::PanelButton;

thread_local! {
  static INSTANCES: RefCell<IndexMap<Uuid, PanelButton>> = RefCell::new(IndexMap::new());
}

impl PanelButton {
  pub(super) fn register_instance(instance: &PanelButton) {
    INSTANCES.with(|instances| {
      instances.borrow_mut().insert(instance.id(), instance.clone());
    });

    let instance_for_cleanup = instance.clone();
    instance.connect_destroy(move |_| {
      INSTANCES.with(|instances| {
        instances.borrow_mut().shift_remove(&instance_for_cleanup.id());
      });
    });
  }

 pub(super) fn close_other_instances(current_panel_button: &PanelButton) {
    INSTANCES.with(|instances| {
      instances.borrow().values().for_each(|panel_button| {
        if panel_button != current_panel_button {
          panel_button.hide_menu();
        }
      });
    });
  }

  pub fn get_next_instance(&self) -> Option<PanelButton> {
    INSTANCES.with(|instances| {
      let instances = instances.borrow();
      let current_index = instances.get_index_of(&self.id())?;
      let next_index = (current_index + 1) % instances.len();
      instances.get_index(next_index).map(|(_, pb)| pb.clone())
    })
  }

  pub fn get_previous_instance(&self) -> Option<PanelButton> {
    INSTANCES.with(|instances| {
      let instances = instances.borrow();
      let current_index = instances.get_index_of(&self.id())?;
      let prev_index = if current_index == 0 { instances.len() - 1 } else { current_index - 1 };
      instances.get_index(prev_index).map(|(_, pb)| pb.clone())
    })
  }
}