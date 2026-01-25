use gtk::gio;
use gtk::prelude::*;
use gtk::gdk;
use gtk4_layer_shell::LayerShell;
use std::cell::RefCell;
use std::rc::Rc;

use crate::system_panel::SystemPanel;

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

  let panels = Rc::new(RefCell::new(Vec::<SystemPanel>::new()));

  {
    let panels = panels.clone();
    app.connect_activate(move |app| {
      if let Err(e) = build_panels_for_all_monitors(app, &panels) {
        eprintln!("Failed to build panels: {}", e);
        std::process::exit(1);
      }
    });
  }

  app.run_with_args(&Vec::<String>::new());
  Ok(())
}

fn build_panels_for_all_monitors(
  app: &adw::Application,
  panels: &Rc<RefCell<Vec<SystemPanel>>>
) -> Result<(), Box<dyn std::error::Error>> {
  let display = gdk::Display::default().ok_or("Could not get default display")?;

  // Clear existing panels
  panels.borrow_mut().clear();

  // Create panel for each monitor
  let n_monitors = display.monitors().n_items();
  
  for i in 0..n_monitors {
    if let Some(monitor_obj) = display.monitors().item(i) {
      let monitor = monitor_obj.downcast::<gdk::Monitor>()
        .map_err(|_| "Failed to downcast monitor")?;
     
      let panel = create_panel_for_monitor(app, &monitor, i as usize)?;
      panel.present();
      
      panels.borrow_mut().push(panel);
    }
  }
  
  // Set up monitor change handling
  {
    let panels = panels.clone();
    let app_weak = app.downgrade();
    let monitors = display.monitors();
    monitors.connect_items_changed(move |_, _, _, _| {
      if let Some(app) = app_weak.upgrade() {
        if let Err(e) = build_panels_for_all_monitors(&app, &panels) {
          eprintln!("Failed to recreate panels: {}", e);
        }
      }
    });
  }
  
  Ok(())
}

fn create_panel_for_monitor(
  app: &adw::Application,
  monitor: &gdk::Monitor,
  monitor_index: usize
) -> Result<SystemPanel, Box<dyn std::error::Error>> {
  // Get the monitor connector name (e.g., "eDP-1", "DP-1")
  let monitor_name = monitor.connector()
    .map(|s| s.to_string())
    .unwrap_or_else(|| format!("monitor-{}", monitor_index));

  // Create the panel with the monitor name
  let panel = SystemPanel::new_with_monitor(app, monitor_name)?;

  // Explicitly assign this panel to the specific monitor
  panel.window.set_monitor(Some(monitor));

  Ok(panel)
}
