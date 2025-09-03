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
  pub window: gtk::ApplicationWindow,
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
    self.add_sample_menus_for_monitor(0);
  }

  pub fn add_sample_menus_for_monitor(&mut self, monitor_index: usize) {
    match monitor_index {
      0 => self.add_primary_monitor_menus(),
      1 => self.add_secondary_monitor_menus(),
      _ => self.add_additional_monitor_menus(monitor_index),
    }
  }

  fn add_primary_monitor_menus(&mut self) {
    println!("Adding primary monitor menus (File, Edit, View)");
    
    // File menu
    let file_menu = DropdownMenuButton::builder()
      .with_text("File")
      .build();

    file_menu.connect_item_selected(|_, item_id| {
      println!("Primary File item selected: {}", item_id);
    });

    file_menu.connect_item_toggled(|_, item_id, toggled_state| {
      println!("Primary File item toggled: {} ({})", item_id, if toggled_state { "ON" } else { "OFF" });
    });

    let file_menu = self.setup_dropdown(file_menu);
    let file_items = vec![
      MenuItem::new("new", "New File").with_icon("document-new-symbolic"),
      MenuItem::new("open", "Open...").with_icon("document-open-symbolic"),
      MenuItem::new("recent", "Recent Files")
        .with_icon("document-open-recent-symbolic")
        .with_submenu(vec![
          MenuItem::new("recent1", "config.toml"),
          MenuItem::new("recent2", "main.rs"),
          MenuItem::new("recent3", "style.css"),
        ]),
      MenuItem::separator(),
      MenuItem::new("save", "Save").with_icon("document-save-symbolic"),
      MenuItem::new("save_as", "Save As...").with_icon("document-save-as-symbolic"),
      MenuItem::separator(),
      MenuItem::new("quit", "Quit").with_icon("application-exit-symbolic"),
    ];
    file_menu.set_menu_items(file_items);
    self.add_widget_to_left(&file_menu);
    self.dropdowns.borrow_mut().push(file_menu);

    // Edit menu
    let edit_menu = DropdownMenuButton::builder()
      .with_text("Edit")
      .build();

    edit_menu.connect_item_selected(|_, item_id| {
      println!("Primary Edit item selected: {}", item_id);
    });

    let edit_menu = self.setup_dropdown(edit_menu);
    let edit_items = vec![
      MenuItem::new("undo", "Undo").with_icon("edit-undo-symbolic"),
      MenuItem::new("redo", "Redo").with_icon("edit-redo-symbolic"),
      MenuItem::separator(),
      MenuItem::new("cut", "Cut").with_icon("edit-cut-symbolic"),
      MenuItem::new("copy", "Copy").with_icon("edit-copy-symbolic"),
      MenuItem::new("paste", "Paste").with_icon("edit-paste-symbolic"),
      MenuItem::separator(),
      MenuItem::new("find", "Find").with_icon("edit-find-symbolic"),
      MenuItem::new("replace", "Replace").with_icon("edit-find-replace-symbolic"),
    ];
    edit_menu.set_menu_items(edit_items);
    self.add_widget_to_left(&edit_menu);
    self.dropdowns.borrow_mut().push(edit_menu);

    // View menu
    let view_menu = DropdownMenuButton::builder()
      .with_text("View")
      .build();

    view_menu.connect_item_selected(|_, item_id| {
      println!("Primary View item selected: {}", item_id);
    });

    view_menu.connect_item_toggled(|_, item_id, toggled_state| {
      println!("Primary View item toggled: {} ({})", item_id, if toggled_state { "ON" } else { "OFF" });
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
  }

  fn add_secondary_monitor_menus(&mut self) {
    println!("Adding secondary monitor menus (Tools, Debug, Help)");

    // Tools menu
    let tools_menu = DropdownMenuButton::builder()
      .with_icon_and_text("applications-development-symbolic", "Tools")
      .build();

    tools_menu.connect_item_selected(|_, item_id| {
      println!("Secondary Tools item selected: {}", item_id);
    });

    let tools_menu = self.setup_dropdown(tools_menu);
    let tools_items = vec![
      MenuItem::new("terminal", "Terminal").with_icon("utilities-terminal-symbolic"),
      MenuItem::new("calculator", "Calculator").with_icon("accessories-calculator-symbolic"),
      MenuItem::new("text_editor", "Text Editor").with_icon("accessories-text-editor-symbolic"),
      MenuItem::separator(),
      MenuItem::new("system_monitor", "System Monitor").with_icon("utilities-system-monitor-symbolic"),
      MenuItem::new("disk_usage", "Disk Usage").with_icon("baobab-symbolic"),
      MenuItem::new("network", "Network Tools")
        .with_icon("network-wired-symbolic")
        .with_submenu(vec![
          MenuItem::new("ping", "Ping"),
          MenuItem::new("traceroute", "Traceroute"),
          MenuItem::new("netstat", "Network Status"),
        ]),
    ];
    tools_menu.set_menu_items(tools_items);
    self.add_widget_to_left(&tools_menu);
    self.dropdowns.borrow_mut().push(tools_menu);

    // Debug menu
    let debug_menu = DropdownMenuButton::builder()
      .with_icon_and_text("applications-debugging-symbolic", "Debug")
      .build();

    debug_menu.connect_item_selected(|_, item_id| {
      println!("Secondary Debug item selected: {}", item_id);
    });

    debug_menu.connect_item_toggled(|_, item_id, toggled_state| {
      println!("Secondary Debug item toggled: {} ({})", item_id, if toggled_state { "ON" } else { "OFF" });
    });

    let debug_menu = self.setup_dropdown(debug_menu);
    let debug_items = vec![
      MenuItem::new("start_debug", "Start Debugging").with_icon("media-playback-start-symbolic"),
      MenuItem::new("stop_debug", "Stop Debugging").with_icon("media-playback-stop-symbolic"),
      MenuItem::separator(),
      MenuItem::new("breakpoint", "Toggle Breakpoint").with_icon("media-record-symbolic"),
      MenuItem::new("step_over", "Step Over").with_icon("go-next-symbolic"),
      MenuItem::new("step_into", "Step Into").with_icon("go-down-symbolic"),
      MenuItem::separator(),
      MenuItem::new("verbose_logging", "Verbose Logging")
        .with_icon("text-x-generic-symbolic")
        .toggled(),
    ];
    debug_menu.set_menu_items(debug_items);
    self.add_widget_to_left(&debug_menu);
    self.dropdowns.borrow_mut().push(debug_menu);

    // Help menu on the right
    let help_menu = DropdownMenuButton::builder()
      .with_icon("help-browser-symbolic")
      .build();

    help_menu.connect_item_selected(|_, item_id| {
      println!("Secondary Help item selected: {}", item_id);
    });

    let help_menu = self.setup_dropdown(help_menu);
    let help_items = vec![
      MenuItem::new("documentation", "Documentation").with_icon("help-contents-symbolic"),
      MenuItem::new("shortcuts", "Keyboard Shortcuts").with_icon("input-keyboard-symbolic"),
      MenuItem::new("tutorial", "Tutorial").with_icon("applications-education-symbolic"),
      MenuItem::separator(),
      MenuItem::new("report_bug", "Report Bug").with_icon("tools-report-bug-symbolic"),
      MenuItem::new("about", "About").with_icon("help-about-symbolic"),
    ];
    help_menu.set_menu_items(help_items);
    self.add_widget_to_right(&help_menu);
    self.dropdowns.borrow_mut().push(help_menu);
  }

  fn add_additional_monitor_menus(&mut self, monitor_index: usize) {
    println!("Adding additional monitor {} menus (System, Monitor Info)", monitor_index);

    // System menu
    let system_menu = DropdownMenuButton::builder()
      .with_icon_and_text("computer-symbolic", &format!("System {}", monitor_index))
      .build();

    system_menu.connect_item_selected(move |_, item_id| {
      println!("Monitor {} System item selected: {}", monitor_index, item_id);
    });

    system_menu.connect_item_toggled(move |_, item_id, toggled_state| {
      println!("Monitor {} System item toggled: {} ({})", monitor_index, item_id, if toggled_state { "ON" } else { "OFF" });
    });

    let system_menu = self.setup_dropdown(system_menu);
    let system_items = vec![
      MenuItem::new("system_info", "System Information").with_icon("dialog-information-symbolic"),
      MenuItem::new("processes", "Processes").with_icon("utilities-system-monitor-symbolic"),
      MenuItem::new("services", "Services").with_icon("preferences-system-services-symbolic"),
      MenuItem::separator(),
      MenuItem::new("power", "Power Options")
        .with_icon("system-shutdown-symbolic")
        .with_submenu(vec![
          MenuItem::new("suspend", "Suspend"),
          MenuItem::new("hibernate", "Hibernate"),
          MenuItem::new("restart", "Restart"),
          MenuItem::new("shutdown", "Shutdown"),
        ]),
      MenuItem::separator(),
      MenuItem::new("auto_update", "Auto Updates")
        .with_icon("software-update-available-symbolic")
        .toggled(),
    ];
    system_menu.set_menu_items(system_items);
    self.add_widget_to_left(&system_menu);
    self.dropdowns.borrow_mut().push(system_menu);

    // Monitor info on the right
    let monitor_menu = DropdownMenuButton::builder()
      .with_icon_and_text("video-display-symbolic", &format!("Monitor {}", monitor_index))
      .build();

    monitor_menu.connect_item_selected(move |_, item_id| {
      println!("Monitor {} info item selected: {}", monitor_index, item_id);
    });

    let monitor_menu = self.setup_dropdown(monitor_menu);
    let monitor_items = vec![
      MenuItem::new("display_settings", "Display Settings").with_icon("preferences-desktop-display-symbolic"),
      MenuItem::new("resolution", "Resolution").with_icon("preferences-desktop-screensaver-symbolic"),
      MenuItem::new("brightness", "Brightness").with_icon("display-brightness-symbolic"),
      MenuItem::separator(),
      MenuItem::new("wallpaper", "Wallpaper").with_icon("preferences-desktop-wallpaper-symbolic"),
      MenuItem::new("screensaver", "Screensaver").with_icon("preferences-desktop-screensaver-symbolic"),
    ];
    monitor_menu.set_menu_items(monitor_items);
    self.add_widget_to_right(&monitor_menu);
    self.dropdowns.borrow_mut().push(monitor_menu);
  }
}
