use gtk::gio;
use gtk::prelude::*;
use gtk::gdk;
use gtk4_layer_shell::LayerShell;
use std::cell::RefCell;
use std::rc::Rc;

use crate::curtain_bar::CurtainBar;


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
    .application_id("com.example.CurtainBar")
    .flags(gio::ApplicationFlags::empty())
    .build();

  let panels = Rc::new(RefCell::new(Vec::<CurtainBar>::new()));
  
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
  panels: &Rc<RefCell<Vec<CurtainBar>>>
) -> Result<(), Box<dyn std::error::Error>> {
  let display = gdk::Display::default().ok_or("Could not get default display")?;
  
  // Clear existing panels
  panels.borrow_mut().clear();
  
  // Create panel for each monitor
  let n_monitors = display.monitors().n_items();
  println!("Found {} monitors", n_monitors);
  
  for i in 0..n_monitors {
    if let Some(monitor_obj) = display.monitors().item(i) {
      let monitor = monitor_obj.downcast::<gdk::Monitor>()
        .map_err(|_| "Failed to downcast monitor")?;
      
      println!("Creating panel for monitor {}: {:?}", i, monitor.model());
      
      let mut panel = create_panel_for_monitor(app, &monitor, i as usize)?;
      panel.add_sample_menus_for_monitor(i as usize);
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
      println!("Monitors changed, recreating panels");
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
) -> Result<CurtainBar, Box<dyn std::error::Error>> {
  let geometry = monitor.geometry();
  println!("Monitor geometry: {}x{} at ({}, {})", 
           geometry.width(), geometry.height(), geometry.x(), geometry.y());
  
  // Create the panel
  let panel = CurtainBar::new(app)?;
  
  // Explicitly assign this panel to the specific monitor
  panel.window.set_monitor(Some(monitor));
  
  Ok(panel)
}
