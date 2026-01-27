use gtk::prelude::*;
use gtk::{IconTheme, Widget};
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
    app_icon_buttons: Vec<(String, PanelButton)>, // (window_address, button)
    plus_button: PanelButton,
    current_monitor: Option<String>,
}

impl WorkspaceButton {
    pub fn _new() -> Self {
        let button_group = PanelButtonGroup::new();
        let plus_button = PanelButton::from_text("+");

        let state = Rc::new(RefCell::new(WorkspaceButtonState {
            workspace_buttons: Vec::new(),
            app_icon_buttons: Vec::new(),
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
            app_icon_buttons: Vec::new(),
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

        // Remove all existing buttons (workspaces, app icons, and + button)
        for (_, button) in &state.workspace_buttons {
            self.button_group.remove_button(button);
        }
        state.workspace_buttons.clear();

        for (_, button) in &state.app_icon_buttons {
            self.button_group.remove_button(button);
        }
        state.app_icon_buttons.clear();

        // Remove the + button
        self.button_group.remove_button(&state.plus_button);

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

            // Add button to the group
            self.button_group.add_button(&button);
            state.workspace_buttons.push((workspace.id, button));
        }

        // Add the + button after workspace buttons
        // Check if + button should be disabled
        let should_disable = workspace_state.workspaces.iter().any(|ws| {
            ws.id == workspace_state.active_workspace_id && ws.windows == 0
        });

        let plus_button_widget: Widget = state.plus_button.clone().upcast();
        if should_disable {
            plus_button_widget.add_css_class("workspace-plus-disabled");
        } else {
            plus_button_widget.remove_css_class("workspace-plus-disabled");
        }

        self.button_group.add_button(&state.plus_button);

        // Create app icon buttons for each window
        // Windows are already filtered to this monitor's workspaces
        let mut windows = workspace_state.windows.clone();
        // Sort by workspace_id first, then by address (as a proxy for creation time)
        windows.sort_by_key(|w| (w.workspace_id, w.address.clone()));

        for window in windows {
            let icon_name = get_icon_for_app(&window.class);
            let button = PanelButton::from_icon_name(&icon_name);

            // Add CSS class for active window
            if let Some(ref active_address) = workspace_state.active_window_address {
                if &window.address == active_address {
                    let button_widget: Widget = button.clone().upcast();
                    button_widget.add_css_class("workspace-active");
                }
            }

            // Connect click handler
            let window_address = window.address.clone();
            let workspace_id = window.workspace_id;
            button.connect_button_clicked(move |_| {
                // First switch to the workspace
                HyprlandService::switch_workspace(workspace_id);
                // Then focus the window
                HyprlandService::focus_window(&window_address);
            });

            // Add button to the group
            self.button_group.add_button(&button);
            state.app_icon_buttons.push((window.address.clone(), button));
        }
    }
}

impl CompositeWidget for WorkspaceButton {
    fn widget(&self) -> &Widget {
        self.button_group.upcast_ref()
    }
}

/// Helper function to get an icon name for an application class
fn get_icon_for_app(app_class: &str) -> String {
    let display = match gtk::gdk::Display::default() {
        Some(d) => d,
        None => {
            eprintln!("No display available for icon theme");
            return "application-x-executable".to_string();
        }
    };

    let icon_theme = IconTheme::for_display(&display);

    // Try the app class as-is (often works for common apps like "firefox", "kitty", etc.)
    if icon_theme.has_icon(app_class) {
        return app_class.to_string();
    }

    // Try lowercase version
    let lowercase = app_class.to_lowercase();
    if icon_theme.has_icon(&lowercase) {
        return lowercase;
    }

    // Try common icon name patterns
    let patterns = vec![
        app_class.to_string(),
        lowercase.clone(),
        format!("{}-icon", lowercase),
        format!("application-{}", lowercase),
    ];

    for pattern in patterns {
        if icon_theme.has_icon(&pattern) {
            return pattern;
        }
    }

    // Try to find icon from desktop files
    if let Some(icon) = find_icon_from_desktop_files(app_class) {
        if icon_theme.has_icon(&icon) {
            return icon;
        }
    }

    // Fallback to generic application icon
    "application-x-executable".to_string()
}

/// Search desktop files for an application and extract its icon
fn find_icon_from_desktop_files(app_class: &str) -> Option<String> {
    use std::fs;
    use std::path::Path;

    let home_dir = std::env::var("HOME").unwrap_or_default();
    let desktop_dirs = vec![
        "/usr/share/applications".to_string(),
        "/usr/local/share/applications".to_string(),
        format!("{}/.local/share/applications", home_dir),
    ];

    let lowercase_class = app_class.to_lowercase();

    for dir in desktop_dirs {
        let dir_path = Path::new(&dir);
        if !dir_path.exists() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("desktop") {
                    continue;
                }

                let filename = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let filename_lower = filename.to_lowercase();

                // Check if filename matches the app class
                if filename_lower.contains(&lowercase_class) || lowercase_class.contains(&filename_lower) {
                    if let Ok(contents) = fs::read_to_string(&path) {
                        // Parse the desktop file for Icon= line
                        for line in contents.lines() {
                            if line.starts_with("Icon=") {
                                let icon = line.trim_start_matches("Icon=").trim().to_string();
                                return Some(icon);
                            }
                        }
                    }
                }
            }
        }
    }

    None
}
