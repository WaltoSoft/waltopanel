use gtk::glib;
use serde_json::Value;
use std::cell::RefCell;
use std::env;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::rc::Rc;
use std::thread;

#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceInfo {
    pub id: i32,
    pub name: String,
    pub monitor: String,
    pub windows: i32,
    pub has_fullscreen: bool,
}

#[derive(Debug, Clone)]
pub struct WorkspaceState {
    pub workspaces: Vec<WorkspaceInfo>,
    pub active_workspace_id: i32,
    pub current_monitor: String,
}

type WorkspaceCallback = Box<dyn Fn(WorkspaceState)>;

struct MonitorSubscription {
    monitor_name: String,
    callback: WorkspaceCallback,
}

struct HyprlandServiceState {
    all_workspaces: Vec<WorkspaceInfo>,
    active_workspace_id: i32,
    subscribers: Vec<MonitorSubscription>,
    running: bool,
}

thread_local! {
    static HYPRLAND_SERVICE: RefCell<Option<Rc<RefCell<HyprlandServiceState>>>> = RefCell::new(None);
}

pub struct HyprlandService;

impl HyprlandService {
    /// Start the Hyprland service and return initial workspace state for a monitor
    pub fn start(monitor_name: String) -> WorkspaceState {
        HYPRLAND_SERVICE.with(|service| {
            // If service already running, just return current state for this monitor
            if let Some(state_ref) = service.borrow().as_ref() {
                let state = state_ref.borrow();
                return Self::filter_workspace_state_for_monitor(
                    &state.all_workspaces,
                    state.active_workspace_id,
                    &monitor_name,
                );
            }

            // First time initialization
            let all_workspaces = Self::query_workspaces().unwrap_or_default();
            let active_workspace_id = Self::query_active_workspace().unwrap_or(1);

            let service_state = Rc::new(RefCell::new(HyprlandServiceState {
                all_workspaces: all_workspaces.clone(),
                active_workspace_id,
                subscribers: Vec::new(),
                running: true,
            }));

            *service.borrow_mut() = Some(service_state.clone());

            // Start event listener in a separate thread
            Self::start_event_listener();

            // Return filtered state for this monitor
            Self::filter_workspace_state_for_monitor(
                &all_workspaces,
                active_workspace_id,
                &monitor_name,
            )
        })
    }

    pub fn stop() {
        HYPRLAND_SERVICE.with(|service| {
            if let Some(state) = service.borrow().as_ref() {
                state.borrow_mut().running = false;
            }
            *service.borrow_mut() = None;
        });
    }

    pub fn subscribe<F>(monitor_name: String, callback: F)
    where
        F: Fn(WorkspaceState) + 'static,
    {
        HYPRLAND_SERVICE.with(|service| {
            if let Some(state) = service.borrow().as_ref() {
                state.borrow_mut().subscribers.push(MonitorSubscription {
                    monitor_name,
                    callback: Box::new(callback),
                });
            }
        });
    }

    /// Switch to a workspace by ID
    pub fn switch_workspace(workspace_id: i32) {
        if let Err(e) = Self::send_command(&format!("dispatch workspace {}", workspace_id)) {
            eprintln!("Failed to switch workspace: {}", e);
        }
    }

    /// Create a new workspace and switch to it, or switch to existing empty workspace
    pub fn create_new_workspace_on_monitor(monitor_name: &str) {
        // Check if this monitor already has an empty workspace
        if let Ok(workspaces) = Self::query_workspaces() {
            let empty_workspace = workspaces
                .iter()
                .find(|ws| ws.monitor == monitor_name && ws.windows == 0);

            if let Some(empty_ws) = empty_workspace {
                Self::switch_workspace(empty_ws.id);
                return;
            }
        }

        // No empty workspace on this monitor, create a new one
        // Find the next available workspace ID
        let workspaces = Self::query_workspaces().unwrap_or_default();
        let max_id = workspaces.iter().map(|ws| ws.id).max().unwrap_or(0);
        let new_workspace_id = max_id + 1;

        // First, focus this monitor to ensure the workspace is created on the correct monitor
        if let Err(e) = Self::send_command(&format!("dispatch focusmonitor {}", monitor_name)) {
            eprintln!("Failed to focus monitor: {}", e);
        }

        // Now switch to the new workspace ID (this will create it on the focused monitor)
        if let Err(e) = Self::send_command(&format!("dispatch workspace {}", new_workspace_id)) {
            eprintln!("Failed to create new workspace: {}", e);
        }
    }

    /// Get the socket path for Hyprland IPC
    fn get_socket_path() -> Result<PathBuf, String> {
        let runtime_dir = env::var("XDG_RUNTIME_DIR")
            .map_err(|_| "XDG_RUNTIME_DIR not set".to_string())?;

        let instance_sig = env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .map_err(|_| "HYPRLAND_INSTANCE_SIGNATURE not set".to_string())?;

        Ok(PathBuf::from(runtime_dir)
            .join("hypr")
            .join(instance_sig)
            .join(".socket.sock"))
    }

    /// Get the event socket path for Hyprland IPC
    fn get_event_socket_path() -> Result<PathBuf, String> {
        let runtime_dir = env::var("XDG_RUNTIME_DIR")
            .map_err(|_| "XDG_RUNTIME_DIR not set".to_string())?;

        let instance_sig = env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .map_err(|_| "HYPRLAND_INSTANCE_SIGNATURE not set".to_string())?;

        Ok(PathBuf::from(runtime_dir)
            .join("hypr")
            .join(instance_sig)
            .join(".socket2.sock"))
    }

    /// Send a command to Hyprland
    fn send_command(command: &str) -> Result<String, String> {
        let socket_path = Self::get_socket_path()?;

        let mut stream = UnixStream::connect(&socket_path)
            .map_err(|e| format!("Failed to connect to Hyprland socket: {}", e))?;

        stream
            .write_all(command.as_bytes())
            .map_err(|e| format!("Failed to write to socket: {}", e))?;

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .map_err(|e| format!("Failed to read response: {}", e))?;

        Ok(response)
    }

    /// Query workspace information from Hyprland
    fn query_workspaces() -> Result<Vec<WorkspaceInfo>, String> {
        let response = Self::send_command("j/workspaces")?;
        let workspaces: Value = serde_json::from_str(&response)
            .map_err(|e| format!("Failed to parse JSON: {}. Response was: {}", e, response))?;

        let mut result = Vec::new();

        if let Some(ws_array) = workspaces.as_array() {
            for ws in ws_array {
                if let (Some(id), Some(name), Some(monitor), Some(windows)) = (
                    ws["id"].as_i64(),
                    ws["name"].as_str(),
                    ws["monitor"].as_str(),
                    ws["windows"].as_i64(),
                ) {
                    result.push(WorkspaceInfo {
                        id: id as i32,
                        name: name.to_string(),
                        monitor: monitor.to_string(),
                        windows: windows as i32,
                        has_fullscreen: ws["hasfullscreen"].as_bool().unwrap_or(false),
                    });
                }
            }
        }

        Ok(result)
    }

    /// Get the active workspace ID (globally focused)
    fn query_active_workspace() -> Result<i32, String> {
        let response = Self::send_command("j/activeworkspace")?;
        let workspace: Value = serde_json::from_str(&response)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        workspace["id"]
            .as_i64()
            .map(|id| id as i32)
            .ok_or_else(|| "Failed to get active workspace ID".to_string())
    }

    /// Get the active workspace ID for a specific monitor
    fn query_active_workspace_for_monitor(monitor_name: &str) -> Result<i32, String> {
        let response = Self::send_command("j/monitors")?;
        let monitors: Value = serde_json::from_str(&response)
            .map_err(|e| format!("Failed to parse monitors JSON: {}", e))?;

        if let Some(monitors_array) = monitors.as_array() {
            for monitor in monitors_array {
                if let Some(name) = monitor["name"].as_str() {
                    if name == monitor_name {
                        if let Some(workspace_id) = monitor["activeWorkspace"]["id"].as_i64() {
                            return Ok(workspace_id as i32);
                        }
                    }
                }
            }
        }

        Err(format!("Could not find active workspace for monitor {}", monitor_name))
    }

    /// Filter workspace state for a specific monitor
    fn filter_workspace_state_for_monitor(
        all_workspaces: &[WorkspaceInfo],
        _global_active_workspace_id: i32,
        monitor_name: &str,
    ) -> WorkspaceState {
        // Filter workspaces for this monitor and apply visibility rules
        let monitor_workspaces = Self::filter_visible_workspaces(all_workspaces, monitor_name);

        // Get the active workspace for THIS monitor specifically
        let active_workspace_id = Self::query_active_workspace_for_monitor(monitor_name)
            .unwrap_or(_global_active_workspace_id);

        WorkspaceState {
            workspaces: monitor_workspaces,
            active_workspace_id,
            current_monitor: monitor_name.to_string(),
        }
    }

    /// Filter workspaces according to the visibility rules:
    /// - Show workspaces with windows
    /// - Keep only one empty workspace per monitor
    fn filter_visible_workspaces(
        workspaces: &[WorkspaceInfo],
        monitor_name: &str,
    ) -> Vec<WorkspaceInfo> {
        let mut monitor_workspaces: Vec<WorkspaceInfo> = workspaces
            .iter()
            .filter(|ws| ws.monitor == monitor_name)
            .cloned()
            .collect();

        // Sort by ID
        monitor_workspaces.sort_by_key(|ws| ws.id);

        // Count empty workspaces
        let empty_workspaces: Vec<&WorkspaceInfo> = monitor_workspaces
            .iter()
            .filter(|ws| ws.windows == 0)
            .collect();

        // If there are multiple empty workspaces, keep only the first one
        if empty_workspaces.len() > 1 {
            let first_empty_id = empty_workspaces[0].id;
            monitor_workspaces.retain(|ws| ws.windows > 0 || ws.id == first_empty_id);
        }

        monitor_workspaces
    }

    /// Notify all subscribers with updated workspace states
    fn notify_subscribers() {
        HYPRLAND_SERVICE.with(|service| {
            if let Some(state_ref) = service.borrow().as_ref() {
                let state = state_ref.borrow();
                let all_workspaces = state.all_workspaces.clone();
                let active_workspace_id = state.active_workspace_id;

                // Notify each subscriber with their monitor-specific state
                for sub in &state.subscribers {
                    let monitor_state = Self::filter_workspace_state_for_monitor(
                        &all_workspaces,
                        active_workspace_id,
                        &sub.monitor_name,
                    );
                    (sub.callback)(monitor_state);
                }
            }
        });
    }

    /// Start listening to Hyprland events
    fn start_event_listener() {
        thread::spawn(move || {
            let event_socket_path = match Self::get_event_socket_path() {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("Failed to get event socket path: {}", e);
                    return;
                }
            };

            let stream = match UnixStream::connect(&event_socket_path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to connect to Hyprland event socket: {}", e);
                    return;
                }
            };

            let reader = BufReader::new(stream);

            for line in reader.lines() {
                if let Ok(event) = line {
                    // Check if this is a workspace-related event
                    if event.starts_with("workspace>>")
                        || event.starts_with("createworkspace>>")
                        || event.starts_with("destroyworkspace>>")
                        || event.starts_with("moveworkspace>>")
                        || event.starts_with("openwindow>>")
                        || event.starts_with("closewindow>>")
                        || event.starts_with("movewindow>>")
                    {
                        // Schedule the state update on the main thread
                        glib::idle_add_once(move || {
                            // Update global workspace state
                            let all_workspaces = Self::query_workspaces().unwrap_or_default();
                            let active_workspace_id =
                                Self::query_active_workspace().unwrap_or(1);

                            HYPRLAND_SERVICE.with(|service| {
                                if let Some(state_ref) = service.borrow().as_ref() {
                                    let mut state = state_ref.borrow_mut();
                                    state.all_workspaces = all_workspaces;
                                    state.active_workspace_id = active_workspace_id;
                                }
                            });

                            // Notify all subscribers
                            Self::notify_subscribers();
                        });
                    }
                }
            }
        });
    }
}
