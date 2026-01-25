use gtk::glib::object::Cast;
use gtk::prelude::{PopoverExt, WidgetExt};
use gtk::{Popover, Widget};
use gtk::{PositionType, StateFlags};

use crate::widgets::PanelButton;
use crate::traits::{CompositeWidget, PopoverExtensions};

#[derive(Clone)]
pub struct DropdownComponent {
  parent_panel_button: PanelButton,
  container: Popover,
}

// Public API--------------------------------------------------------------------------------------
impl DropdownComponent {
  pub fn new(parent: &PanelButton) -> Self {
    let popover = Popover::builder()
      .autohide(true)
      .has_arrow(false)
      .position(PositionType::Bottom)
      .can_focus(true)
      .focusable(true)
      .build();

    let dropdown = Self {
      parent_panel_button: parent.clone(),
      container: popover,
    };

    dropdown.setup_popover_handlers();
    dropdown
  }

  pub fn set_widget(&self, widget: Option<&Widget>) {
    self.container.set_child(widget);
  }

  pub fn show(&self) {
    self.container.update_popover_alignment();
    self.container.popup();
  }

  pub fn hide(&self) {
    if self.container.is_visible() {
      self.container.popdown();
    }
  }

  pub fn toggle_visibility(&self) {
    if self.container.is_visible() {
      self.hide();
    } else {
      self.show();
    }
  }

  pub fn _is_visible(&self) -> bool {
    self.container.is_visible()
  }
}
// End Public API----------------------------------------------------------------------------------

// Event Handler Methods---------------------------------------------------------------------------
impl DropdownComponent {
  fn setup_popover_handlers(&self) {
    let panel_button_show = self.parent_panel_button.clone();
    let panel_button_hide = self.parent_panel_button.clone();

    self.container.connect_show(move |_| {
      panel_button_show.set_state_flags(StateFlags::ACTIVE, false);
    });

    self.container.connect_hide(move |_| {
      panel_button_hide.unset_state_flags(StateFlags::ACTIVE);
    });
  }
}
// End Event Handler Methods-----------------------------------------------------------------------

impl CompositeWidget for DropdownComponent {
  fn widget(&self) -> Widget {
    self.container.clone().upcast()
  }
}

impl std::fmt::Debug for DropdownComponent {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("DropdownComponent")
      .field("parent_panel_button", &self.parent_panel_button)
      .field("popover", &self.container)
      .finish()
  }
}
