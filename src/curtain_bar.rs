use gtk::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

use crate::models::{CurtainBarConfig, Margins, MenuItem};
use crate::widgets::dropdown_menu_button::DropdownMenuButton;

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
    self.config.margins = Margins {
      top,
      bottom,
      left,
      right,
    };
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
  dropdowns: Rc<RefCell<Vec<DropdownMenuButton>>>,
}

impl CurtainBar {
  pub fn builder() -> CurtainBarBuilder {
    CurtainBarBuilder::new()
  }

  pub fn new(app: &adw::Application) -> Result<Self, Box<dyn std::error::Error>> {
    Self::with_config(app, CurtainBarConfig::default())
  }

  fn with_config(
    app: &adw::Application,
    config: CurtainBarConfig,
  ) -> Result<Self, Box<dyn std::error::Error>> {
    let window = gtk::ApplicationWindow::builder()
      .application(app)
      .title("Curtain Bar")
      .build();

    Self::configure_layer_shell(&window, &config)?;

    let (left_box, center_box, right_box) = Self::create_layout(config.spacing);
    let panel_box =
      Self::create_panel_container(&left_box, &center_box, &right_box, &config.margins);

    window.set_child(Some(&panel_box));

    let curtain_bar = Self {
      window: window.clone(),
      left_box,
      center_box,
      right_box,
      dropdowns: Rc::new(RefCell::new(Vec::new())),
    };

    Ok(curtain_bar)
  }

  fn configure_layer_shell(
    window: &gtk::ApplicationWindow,
    config: &CurtainBarConfig,
  ) -> Result<(), Box<dyn std::error::Error>> {
    window.init_layer_shell();
    window.set_layer(config.layer);
    window.auto_exclusive_zone_enable();

    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.set_anchor(Edge::Bottom, false);

    // Enable keyboard interactivity for the layer shell window
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);

    // Make the window focusable and able to receive input
    window.set_can_focus(true);
    window.set_focusable(true);

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

  fn setup_dropdown(&self, dropdown: DropdownMenuButton) -> DropdownMenuButton {
    dropdown
  }

  pub fn add_sample_menus(&mut self) {
    // File menu
    let file_menu = DropdownMenuButton::builder()
      .with_text("File")
      .build();

    let file_items = vec![
      MenuItem::new("new", "New File").with_icon("document-new-symbolic"),
      MenuItem::new("open", "Open..."),
      MenuItem::new("recent", "Recent Files")
        .with_icon("document-open-recent-symbolic")
        .with_submenu(vec![
          MenuItem::new("recent1", "config.toml"),
          MenuItem::new("recent2", "main.rs"),
          MenuItem::new("recent3", "style.css"),
        ]),
      MenuItem::separator(),
      MenuItem::new("save", "Save").toggled(),
      MenuItem::new("save_as", "Save As..."),
      MenuItem::new("export", "Export"),
      MenuItem::separator(),
      MenuItem::new("quit", "Quit").with_icon("application-exit-symbolic"),
    ];

    file_menu.connect_item_selected(|_, item_id| {
      println!("File item selected: {}", item_id);
    });

    file_menu.connect_item_toggled(|_, item_id, toggled_state| {
      println!("File item toggled: {} ({})", item_id, if toggled_state { "ON" } else { "OFF" });
    });

    let file_menu_setup = self.setup_dropdown(file_menu);
    file_menu_setup.set_menu_items(file_items);
    self.add_widget_to_left(&file_menu_setup);
    self.dropdowns.borrow_mut().push(file_menu_setup);

    // View menu
    let view_menu = DropdownMenuButton::builder()
      .with_text("View")
      .build();
    
    view_menu.connect_item_selected(|_, item_id| {
      println!("View item selected: {}", item_id);
    });

    view_menu.connect_item_toggled(|_, item_id, toggled_state| {
      println!("View item toggled: {} ({})", item_id, if toggled_state { "ON" } else { "OFF" });
    });
    
    let view_menu = self.setup_dropdown(view_menu);

    let view_items = vec![
      MenuItem::new("fullscreen", "Fullscreen").with_icon("view-fullscreen-symbolic"),
      MenuItem::new("zoom_in", "Zoom In").with_icon("zoom-in-symbolic"),
      MenuItem::new("zoom_out", "Zoom Out").with_icon("zoom-out-symbolic"),
      MenuItem::new("zoom_reset", "Reset Zoom").with_icon("zoom-original-symbolic"),
      MenuItem::separator(),
      MenuItem::new("dark_mode", "Dark Mode")
        .with_icon("weather-clear-night-symbolic")
        .toggled(),
      MenuItem::new("show_sidebar", "Show Sidebar")
        .with_icon("view-dual-symbolic")
        .toggled(),
    ];

    view_menu.set_menu_items(view_items);
    self.add_widget_to_left(&view_menu);
    self.dropdowns.borrow_mut().push(view_menu);

    // Settings menu with icon
    let settings_menu = DropdownMenuButton::builder()
      .with_icon_and_text("preferences-system-symbolic", "Settings")
      .build();
    
    settings_menu.connect_item_selected(|_, item_id| {
      println!("Settings item selected: {}", item_id);
    });

    settings_menu.connect_item_toggled(|_, item_id, toggled_state| {
      println!("Settings item toggled: {} ({})", item_id, if toggled_state { "ON" } else { "OFF" });
    });
    
    let settings_menu = self.setup_dropdown(settings_menu);

    let settings_items = vec![
      MenuItem::new("preferences", "Preferences...").with_icon("preferences-system-symbolic"),
      MenuItem::new("keyboard", "Keyboard Shortcuts").with_icon("input-keyboard-symbolic"),
      MenuItem::separator(),
      MenuItem::new("plugins", "Plugins").with_icon("application-x-addon-symbolic"),
      MenuItem::new("themes", "Themes")
        .with_icon("preferences-desktop-theme-symbolic")
        .with_submenu(vec![
          MenuItem::new("theme_light", "Light Theme"),
          MenuItem::new("theme_dark", "Dark Theme").toggled(),
          MenuItem::new("theme_auto", "Auto (System)"),
          MenuItem::new("theme_custom", "Custom Themes").with_submenu(vec![
            MenuItem::new("theme_blue", "Blue Theme"),
            MenuItem::new("theme_green", "Green Theme"),
            MenuItem::new("theme_advanced", "Advanced").with_submenu(vec![
              MenuItem::new("theme_editor", "Theme Editor"),
              MenuItem::new("theme_import", "Import Theme"),
              MenuItem::new("theme_export", "Export Theme"),
            ]),
          ]),
        ]),
    ];

    settings_menu.set_menu_items(settings_items);
    self.add_widget_to_right(&settings_menu);
    self.dropdowns.borrow_mut().push(settings_menu);

    // User menu (icon only)
    let user_menu = DropdownMenuButton::builder()
      .with_icon("avatar-default-symbolic")
      .build();
    
    user_menu.connect_item_selected(|_, item_id| {
      println!("User item selected: {}", item_id);
    });

    user_menu.connect_item_toggled(|_, item_id, toggled_state| {
      println!("User item toggled: {} ({})", item_id, if toggled_state { "ON" } else { "OFF" });
    });
    
    let user_menu = self.setup_dropdown(user_menu);

    let user_items = vec![
      MenuItem::new("profile", "Profile").with_icon("user-info-symbolic"),
      MenuItem::new("account", "Account Settings").with_icon("system-users-symbolic"),
      MenuItem::separator(),
      MenuItem::new("help", "Help & Support").with_icon("help-browser-symbolic"),
      MenuItem::new("about", "About").with_icon("help-about-symbolic"),
      MenuItem::separator(),
      MenuItem::new("logout", "Log Out").with_icon("system-log-out-symbolic"),
    ];

    user_menu.set_menu_items(user_items);
    self.add_widget_to_right(&user_menu);
    self.dropdowns.borrow_mut().push(user_menu);
  }
}
