use gtk::prelude::*;
use crate::CurtainBar;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    gtk::init()?;

    let app = gtk::Application::builder()
        .application_id("com.example.CurtainBar")
        .build();

    app.connect_activate(|app| {
        if let Err(e) = build_panel(app) {
            eprintln!("Failed to build panel: {}", e);
            std::process::exit(1);
        }
    });
    
    app.run();
    Ok(())
}

fn build_panel(app: &gtk::Application) -> Result<(), Box<dyn std::error::Error>> {
    let panel = CurtainBar::new(app)?;
    panel.present();
    Ok(())
}