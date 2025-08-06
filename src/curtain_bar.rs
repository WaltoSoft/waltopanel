use gtk::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use crate::widgets::{DropdownButton, MenuItem};

#[derive(Debug, Clone)]
pub struct CurtainBarConfig {
    pub height: i32,
    pub layer: Layer,
    pub margins: Margins,
    pub spacing: i32,
}

#[derive(Debug, Clone)]
pub struct Margins {
    pub top: i32,
    pub bottom: i32,
    pub left: i32,
    pub right: i32,
}

impl Default for CurtainBarConfig {
    fn default() -> Self {
        Self {
            height: 40,
            layer: Layer::Top,
            margins: Margins {
                top: 4,
                bottom: 4,
                left: 8,
                right: 8,
            },
            spacing: 8,
        }
    }
}

pub struct CurtainBarBuilder {
    config: CurtainBarConfig,
}

impl CurtainBarBuilder {
    pub fn new() -> Self {
        Self {
            config: CurtainBarConfig::default(),
        }
    }

    pub fn height(mut self, height: i32) -> Self {
        self.config.height = height;
        self
    }

    pub fn layer(mut self, layer: Layer) -> Self {
        self.config.layer = layer;
        self
    }

    pub fn margins(mut self, top: i32, bottom: i32, left: i32, right: i32) -> Self {
        self.config.margins = Margins { top, bottom, left, right };
        self
    }

    pub fn spacing(mut self, spacing: i32) -> Self {
        self.config.spacing = spacing;
        self
    }

    pub fn build(self, app: &adw::Application) -> Result<CurtainBar, Box<dyn std::error::Error>> {
        CurtainBar::with_config(app, self.config)
    }
}

pub struct CurtainBar {
    window: gtk::ApplicationWindow,
    left_box: gtk::Box,
    center_box: gtk::Box,
    right_box: gtk::Box,
    _dropdowns: Vec<DropdownButton>, // Keep dropdown buttons alive
}

impl CurtainBar {
    pub fn builder() -> CurtainBarBuilder {
        CurtainBarBuilder::new()
    }

    pub fn new(app: &adw::Application) -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_config(app, CurtainBarConfig::default())
    }

    fn with_config(app: &adw::Application, config: CurtainBarConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("Curtain Bar")
            .build();

        Self::configure_layer_shell(&window, &config)?;
        
        let (left_box, center_box, right_box) = Self::create_layout(config.spacing);
        let panel_box = Self::create_panel_container(&left_box, &center_box, &right_box, &config.margins);
        
        window.set_child(Some(&panel_box));

        let curtain_bar = Self {
            window: window.clone(),
            left_box,
            center_box,
            right_box,
            _dropdowns: Vec::new(),
        };

        Ok(curtain_bar)
    }

    fn configure_layer_shell(window: &gtk::ApplicationWindow, config: &CurtainBarConfig) -> Result<(), Box<dyn std::error::Error>> {
        window.init_layer_shell();
        window.set_layer(config.layer);
        window.auto_exclusive_zone_enable();
        
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);
        window.set_anchor(Edge::Bottom, false);
        
        window.set_size_request(-1, config.height);
        
        Ok(())
    }

    fn create_layout(spacing: i32) -> (gtk::Box, gtk::Box, gtk::Box) {
        let left_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(spacing)
            .hexpand(true)
            .halign(gtk::Align::Start)
            .build();

        let center_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(spacing)
            .halign(gtk::Align::Center)
            .build();

        let right_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(spacing)
            .hexpand(true)
            .halign(gtk::Align::End)
            .build();

        (left_box, center_box, right_box)
    }

    fn create_panel_container(
        left_box: &gtk::Box,
        center_box: &gtk::Box,
        right_box: &gtk::Box,
        margins: &Margins,
    ) -> gtk::Box {
        let panel_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .margin_start(margins.left)
            .margin_end(margins.right)
            .margin_top(margins.top)
            .margin_bottom(margins.bottom)
            .build();

        panel_box.append(left_box);
        panel_box.append(center_box);
        panel_box.append(right_box);

        panel_box
    }

    pub fn add_widget_to_left(&self, widget: &impl IsA<gtk::Widget>) {
        self.left_box.append(widget);
    }

    pub fn add_widget_to_center(&self, widget: &impl IsA<gtk::Widget>) {
        self.center_box.append(widget);
    }

    pub fn add_widget_to_right(&self, widget: &impl IsA<gtk::Widget>) {
        self.right_box.append(widget);
    }

    pub fn present(&self) {
        self.window.present();
    }
    

    pub fn add_sample_menus(&mut self) {
        // File menu
        let file_menu = DropdownButton::new()
            .with_text("File");

        let file_items = vec![
            MenuItem::new("new", "New File")
                .with_icon("document-new-symbolic"),
            MenuItem::new("open", "Open..."),
            MenuItem::new("recent", "Recent Files")
                .with_icon("document-open-recent-symbolic")
                .with_submenu(vec![
                    MenuItem::new("recent1", "config.toml"),
                    MenuItem::new("recent2", "main.rs"),
                    MenuItem::new("recent3", "style.css"),
                ]),
            MenuItem::separator(),
            MenuItem::new("save", "Save")
                .toggled(),
            MenuItem::new("save_as", "Save As..."),
            MenuItem::new("export", "Export"),
            MenuItem::separator(),
            MenuItem::new("quit", "Quit")
                .with_icon("application-exit-symbolic"),
        ];

        let file_menu_clone = file_menu.clone();
        let file_menu_with_callback = file_menu.on_item_toggled(move |item_id, _| {
            println!("MAIN CALLBACK TRIGGERED for item: {}", item_id);
            
            // Check if this is a submenu item - if so, ignore the callback
            if item_id == "recent" {
                println!("Ignoring callback for submenu item: {}", item_id);
                return;
            }
            
            if item_id == "save" {
                let new_state = file_menu_clone.toggle_item(item_id);
                println!("Toggled: {} ({})", item_id, if new_state { "ON" } else { "OFF" });
            } else {
                println!("Selected: {}", item_id);
            }
        });
        
        file_menu_with_callback.set_menu_items(file_items);
        self.add_widget_to_left(file_menu_with_callback.widget());
        self._dropdowns.push(file_menu_with_callback);

        // View menu
        let view_menu = DropdownButton::new()
            .with_text("View")
            .on_item_toggled(|item_id, is_toggled| {
                println!("View toggled: {} ({})", item_id, if is_toggled { "ON" } else { "OFF" });
            });

        let view_items = vec![
            MenuItem::new("fullscreen", "Fullscreen")
                .with_icon("view-fullscreen-symbolic"),
            MenuItem::new("zoom_in", "Zoom In")
                .with_icon("zoom-in-symbolic"),
            MenuItem::new("zoom_out", "Zoom Out")
                .with_icon("zoom-out-symbolic"),
            MenuItem::new("zoom_reset", "Reset Zoom")
                .with_icon("zoom-original-symbolic"),
            MenuItem::separator(),
            MenuItem::new("dark_mode", "Dark Mode")
                .with_icon("weather-clear-night-symbolic")
                .toggled(),
            MenuItem::new("show_sidebar", "Show Sidebar")
                .with_icon("view-dual-symbolic")
                .toggled(),
        ];

        view_menu.set_menu_items(view_items);
        self.add_widget_to_left(view_menu.widget());
        self._dropdowns.push(view_menu);

        // Settings menu with icon
        let settings_menu = DropdownButton::new()
            .with_icon_and_text("preferences-system-symbolic", "Settings")
            .on_item_toggled(|item_id, is_toggled| {
                println!("Settings toggled: {} ({})", item_id, if is_toggled { "ON" } else { "OFF" });
            });

        let settings_items = vec![
            MenuItem::new("preferences", "Preferences...")
                .with_icon("preferences-system-symbolic"),
            MenuItem::new("keyboard", "Keyboard Shortcuts")
                .with_icon("input-keyboard-symbolic"),
            MenuItem::separator(),
            MenuItem::new("plugins", "Plugins")
                .with_icon("application-x-addon-symbolic"),
            MenuItem::new("themes", "Themes")
                .with_icon("preferences-desktop-theme-symbolic")
                .with_submenu(vec![
                    MenuItem::new("theme_light", "Light Theme"),
                    MenuItem::new("theme_dark", "Dark Theme")
                        .toggled(),
                    MenuItem::new("theme_auto", "Auto (System)"),
                    MenuItem::new("theme_custom", "Custom Themes")
                        .with_submenu(vec![
                            MenuItem::new("theme_blue", "Blue Theme"),
                            MenuItem::new("theme_green", "Green Theme"),
                            MenuItem::new("theme_advanced", "Advanced")
                                .with_submenu(vec![
                                    MenuItem::new("theme_editor", "Theme Editor"),
                                    MenuItem::new("theme_import", "Import Theme"),
                                    MenuItem::new("theme_export", "Export Theme"),
                                ]),
                        ]),
                ]),
        ];

        settings_menu.set_menu_items(settings_items);
        self.add_widget_to_right(settings_menu.widget());
        self._dropdowns.push(settings_menu);

        // User menu (icon only)
        let user_menu = DropdownButton::new()
            .with_icon("avatar-default-symbolic")
            .on_item_toggled(|item_id, is_toggled| {
                println!("User toggled: {} ({})", item_id, if is_toggled { "ON" } else { "OFF" });
            });

        let user_items = vec![
            MenuItem::new("profile", "Profile")
                .with_icon("user-info-symbolic"),
            MenuItem::new("account", "Account Settings")
                .with_icon("system-users-symbolic"),
            MenuItem::separator(),
            MenuItem::new("help", "Help & Support")
                .with_icon("help-browser-symbolic"),
            MenuItem::new("about", "About")
                .with_icon("help-about-symbolic"),
            MenuItem::separator(),
            MenuItem::new("logout", "Log Out")
                .with_icon("system-log-out-symbolic"),
        ];

        user_menu.set_menu_items(user_items);
        self.add_widget_to_right(user_menu.widget());
        self._dropdowns.push(user_menu);
    }
}