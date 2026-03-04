use gtk::glib;
use gtk::gio;
use gtk::prelude::*;
use gtk::gdk;
use gtk4_layer_shell::LayerShell;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use crate::panel_buttons::workspace_button::hyprland_service::HyprlandService;
use crate::system_panel::SystemPanel;

struct PanelEntry {
  connector: String,
  panel: SystemPanel,
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
  adw::init().unwrap();

  // Load CSS
  let css_provider = gtk::CssProvider::new();
  css_provider.load_from_data(include_str!("../style.css"));
  gtk::style_context_add_provider_for_display(
    &gtk::gdk::Display::default().expect("Could not connect to a display."),
    &css_provider,
    gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
  );

  let app = adw::Application::builder()
    .application_id("com.waltosoft.WaltoPanel")
    .flags(gio::ApplicationFlags::empty())
    .build();

  let panels: Rc<RefCell<Vec<PanelEntry>>> = Rc::new(RefCell::new(Vec::new()));

  {
    let panels = panels.clone();
    app.connect_activate(move |app| {
      let display = gdk::Display::default().expect("Could not get default display");

      // Initial panel creation
      sync_panels(app, &panels);

      // Register monitor change handler exactly once, here on activate
      let panels = panels.clone();
      let app_weak = app.downgrade();
      let sync_pending = Rc::new(Cell::new(false));

      display.monitors().connect_items_changed(move |monitors, _pos, _removed, _added| {
        let active_connectors = list_connectors(monitors);

        // Immediately hide panels whose monitor is gone so they don't
        // migrate to another monitor while we wait for the idle callback.
        for entry in panels.borrow().iter() {
          if !active_connectors.contains(&entry.connector) {
            entry.panel.window.set_visible(false);
          }
        }

        // Defer the actual destroy/create work outside the Wayland dispatch.
        // Guard against queuing multiple syncs for rapid back-to-back events.
        if sync_pending.get() {
          return;
        }
        sync_pending.set(true);

        let sync_pending = sync_pending.clone();
        let panels = panels.clone();
        let app_weak = app_weak.clone();
        glib::idle_add_local_once(move || {
          sync_pending.set(false);
          if let Some(app) = app_weak.upgrade() {
            sync_panels(&app, &panels);
          }
        });
      });
    });
  }

  app.run_with_args(&Vec::<String>::new());
  Ok(())
}

/// Returns the connector name for every monitor currently in the list model.
fn list_connectors(monitors: &gio::ListModel) -> Vec<String> {
  (0..monitors.n_items())
    .filter_map(|i| monitors.item(i)?.downcast::<gdk::Monitor>().ok())
    .enumerate()
    .map(|(i, m)| connector_name(&m, i))
    .collect()
}

fn connector_name(monitor: &gdk::Monitor, index: usize) -> String {
  monitor.connector()
    .map(|s| s.to_string())
    .unwrap_or_else(|| format!("monitor-{}", index))
}

/// Diff the current panel list against the connected monitors:
/// destroy panels for monitors that are gone, create panels for new monitors.
fn sync_panels(app: &adw::Application, panels: &Rc<RefCell<Vec<PanelEntry>>>) {
  let display = match gdk::Display::default() {
    Some(d) => d,
    None => return,
  };
  let monitors = display.monitors();

  // Collect current monitors with their connector names
  let current: Vec<(String, gdk::Monitor)> = (0..monitors.n_items())
    .filter_map(|i| monitors.item(i)?.downcast::<gdk::Monitor>().ok())
    .enumerate()
    .map(|(i, m)| (connector_name(&m, i), m))
    .collect();

  let current_connectors: Vec<&str> = current.iter().map(|(c, _)| c.as_str()).collect();

  // Destroy panels for monitors that are no longer connected.
  let mut any_removed = false;
  panels.borrow_mut().retain(|entry| {
    if current_connectors.contains(&entry.connector.as_str()) {
      true
    } else {
      entry.panel.window.destroy();
      any_removed = true;
      false
    }
  });

  // If monitors were removed and at least one remains, move any workspaces
  // still assigned to the now-gone monitors to the first remaining monitor.
  // Pass the GDK connector list directly — Hyprland's IPC monitor list can
  // lag behind GDK when multiple monitors disconnect simultaneously.
  if any_removed {
    if let Some((fallback, _)) = current.first() {
      let active_connectors: Vec<String> = current.iter().map(|(c, _)| c.clone()).collect();
      HyprlandService::move_orphaned_workspaces_to(fallback, &active_connectors);
    }
  }

  // Create panels for monitors that don't have one yet
  let existing: Vec<String> = panels.borrow().iter().map(|e| e.connector.clone()).collect();
  for (connector, monitor) in &current {
    if existing.contains(connector) {
      continue;
    }
    let index = current.iter().position(|(c, _)| c == connector).unwrap_or(0);
    match create_panel_for_monitor(app, monitor, index) {
      Ok(panel) => {
        panel.present();
        panels.borrow_mut().push(PanelEntry {
          connector: connector.clone(),
          panel,
        });
      }
      Err(e) => eprintln!("Failed to create panel for {}: {}", connector, e),
    }
  }
}

fn create_panel_for_monitor(
  app: &adw::Application,
  monitor: &gdk::Monitor,
  monitor_index: usize,
) -> Result<SystemPanel, Box<dyn std::error::Error>> {
  let monitor_name = connector_name(monitor, monitor_index);
  let panel = SystemPanel::new_with_monitor(app, monitor_name)?;
  panel.window.set_monitor(Some(monitor));
  Ok(panel)
}
