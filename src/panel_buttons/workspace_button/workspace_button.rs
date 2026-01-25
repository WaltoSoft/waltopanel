use gtk::prelude::*;
use gtk::Widget;
use std::cell::RefCell;
use std::rc::Rc;

use crate::traits::CompositeWidget;
use crate::widgets::{PanelButton, PanelButtonGroup};

use super::hyprland_service::{HyprlandService, WorkspaceState};

#[derive(Clone, Debug)]
pub struct WorkspaceButton {
    button_group: PanelButtonGroup,
    state: Rc<RefCell<WorkspaceButtonState>>,
}

#[derive(Debug)]
struct WorkspaceButtonState {
    workspace_buttons: Vec<(i32, PanelButton)>, // (workspace_id, button)
    plus_button: PanelButton,
    current_monitor: Option<String>,
}

impl WorkspaceButton {
    pub fn _new() -> Self {
        let button_group = PanelButtonGroup::new();
        let plus_button = PanelButton::from_text("+");

        let state = Rc::new(RefCell::new(WorkspaceButtonState {
            workspace_buttons: Vec::new(),
            plus_button: plus_button.clone(),
            current_monitor: None,
        }));

        let obj = Self {
            button_group: button_group.clone(),
            state: state.clone(),
        };

        // Connect the + button
        let state_clone = state.clone();
        plus_button.connect_button_clicked(move |_| {
            if let Some(monitor_name) = state_clone.borrow().current_monitor.clone() {
                HyprlandService::create_new_workspace_on_monitor(&monitor_name);
            } else {
                eprintln!("Warning: + button clicked but monitor not yet initialized");
            }
        });

        // Add the + button initially
        button_group.add_button(&plus_button);

        // Initialize the service when the widget is realized (attached to a window)
        let obj_clone = obj.clone();
        let button_group_widget: Widget = obj.button_group.clone().upcast();
        button_group_widget.connect_realize(move |widget| {
            if let Some(monitor_name) = Self::_get_monitor_name(widget) {
                obj_clone.initialize_with_monitor(monitor_name);
            } else {
                eprintln!("Warning: Could not determine monitor name for WorkspaceButton");
            }
        });

        obj
    }

    pub fn new_with_monitor(monitor_name: String) -> Self {
        let button_group = PanelButtonGroup::new();
        let plus_button = PanelButton::from_text("+");

        let state = Rc::new(RefCell::new(WorkspaceButtonState {
            workspace_buttons: Vec::new(),
            plus_button: plus_button.clone(),
            current_monitor: Some(monitor_name.clone()),
        }));

        let obj = Self {
            button_group: button_group.clone(),
            state: state.clone(),
        };

        // Connect the + button
        let monitor_name_for_button = monitor_name.clone();
        plus_button.connect_button_clicked(move |_| {
            HyprlandService::create_new_workspace_on_monitor(&monitor_name_for_button);
        });

        // Add the + button initially
        button_group.add_button(&plus_button);

        // Initialize immediately with the provided monitor name
        obj.initialize_with_monitor(monitor_name);

        obj
    }

    /// Initialize the workspace button with a specific monitor
    fn initialize_with_monitor(&self, monitor_name: String) {
        {
            let mut state = self.state.borrow_mut();
            state.current_monitor = Some(monitor_name.clone());
        }

        // Start the Hyprland service
        let initial_state = HyprlandService::start(monitor_name.clone());

        // Update UI with initial state
        self.update_ui(&initial_state);

        // Subscribe to workspace changes for this monitor
        let obj_clone = self.clone();
        HyprlandService::subscribe(monitor_name, move |workspace_state| {
            obj_clone.update_ui(&workspace_state);
        });
    }

    /// Get the monitor name from the widget's window
    fn _get_monitor_name(widget: &Widget) -> Option<String> {
        let native = widget.native()?;
        let surface = native.surface()?;
        let display = surface.display();
        let monitor = display.monitor_at_surface(&surface)?;

        // Try to get connector name (e.g., "DP-1", "HDMI-A-1")
        // This should match Hyprland's monitor names
        monitor.connector().map(|s| s.to_string())
    }

    /// Update the UI based on workspace state
    fn update_ui(&self, workspace_state: &WorkspaceState) {
        let mut state = self.state.borrow_mut();

        // Remove all existing workspace buttons
        for (_, button) in &state.workspace_buttons {
            self.button_group.remove_button(button);
        }
        state.workspace_buttons.clear();

        // Create buttons for each visible workspace
        let mut workspaces = workspace_state.workspaces.clone();
        workspaces.sort_by_key(|ws| ws.id);

        for workspace in workspaces {
            let button = PanelButton::from_text(&workspace.id.to_string());

            // Add CSS class for active workspace (for underline styling)
            if workspace.id == workspace_state.active_workspace_id {
                let button_widget: Widget = button.clone().upcast();
                button_widget.add_css_class("workspace-active");
            }

            // Connect click handler
            let workspace_id = workspace.id;
            button.connect_button_clicked(move |_| {
                HyprlandService::switch_workspace(workspace_id);
            });

            // Add button to the group (before the + button)
            self.button_group.add_button(&button);
            state.workspace_buttons.push((workspace.id, button));
        }

        // Check if + button should be disabled
        // It should be disabled when we're currently on an empty workspace
        let should_disable = workspace_state.workspaces.iter().any(|ws| {
            ws.id == workspace_state.active_workspace_id && ws.windows == 0
        });

        let plus_button_widget: Widget = state.plus_button.clone().upcast();
        if should_disable {
            plus_button_widget.add_css_class("workspace-plus-disabled");
        } else {
            plus_button_widget.remove_css_class("workspace-plus-disabled");
        }

        // Remove and re-add the + button to ensure it's at the end
        self.button_group.remove_button(&state.plus_button);
        self.button_group.add_button(&state.plus_button);
    }
}

impl CompositeWidget for WorkspaceButton {
    fn widget(&self) -> Widget {
        self.button_group.clone().upcast()
    }
}
