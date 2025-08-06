use gtk::prelude::*;
use gtk::gio;
use crate::CurtainBar;

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

    app.connect_activate(|app| {
        if let Err(e) = build_panel(app) {
            eprintln!("Failed to build panel: {}", e);
            std::process::exit(1);
        }
    });
    
    app.run_with_args(&Vec::<String>::new());
    Ok(())
}

fn build_panel(app: &adw::Application) -> Result<(), Box<dyn std::error::Error>> {
    let mut panel = CurtainBar::new(app)?;
    panel.add_sample_menus();
    panel.present();
    Ok(())
}