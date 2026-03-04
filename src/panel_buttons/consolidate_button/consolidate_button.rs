use gtk::{Widget, glib::object::Cast};

use crate::{
    panel_buttons::workspace_button::hyprland_service::HyprlandService,
    traits::CompositeWidget,
    widgets::PanelButton,
};

/// A button that moves all workspaces from external monitors to the internal
/// laptop display (eDP-*). Useful when you want to consolidate everything to
/// the laptop screen.
pub struct ConsolidateButton {
    panel_button: PanelButton,
}

impl ConsolidateButton {
    pub fn new() -> Self {
        let panel_button = PanelButton::from_icon_name("go-home-symbolic");

        panel_button.connect_button_clicked(move |_| {
            HyprlandService::move_all_to_laptop();
        });

        ConsolidateButton { panel_button }
    }
}

impl CompositeWidget for ConsolidateButton {
    fn widget(&self) -> &Widget {
        self.panel_button.upcast_ref()
    }
}
