use gtk::prelude::WidgetExt;
use uuid::Uuid;
use std::{cell::RefCell, collections::HashMap};

use crate::widgets::PanelButton;

thread_local! {
  static INSTANCES: RefCell<HashMap<Uuid, PanelButton>> = RefCell::new(HashMap::new());
  static ACTIVE_INSTANCE: RefCell<Option<PanelButton>> = RefCell::new(None);
}

impl PanelButton {
  pub(super) fn register_instance(instance: &PanelButton) {
    INSTANCES.with(|instances| {
      instances.borrow_mut().insert(instance.id(), instance.clone());
    });

    let instance_for_cleanup = instance.clone();
    instance.connect_destroy(move |_| {
      INSTANCES.with(|instances| {
        instances.borrow_mut().remove(&instance_for_cleanup.id());
      });
    });
  }

  pub(super) fn set_active_instance(instance: &PanelButton) {
   ACTIVE_INSTANCE.with(|active_instance| {
      *active_instance.borrow_mut() = Some(instance.clone());
    });

    PanelButton::close_other_instances(instance);
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
}