use gtk::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

use crate::models::{CurtainBarConfig, Margins, MenuItemModel};
use crate::widgets::PanelButton;
use crate::types::TypedListStore;

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
  panel_buttons: Rc<RefCell<Vec<PanelButton>>>,
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
      panel_buttons: Rc::new(RefCell::new(Vec::new())),
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

  fn setup_panel_button(&self, panel_button: PanelButton) -> PanelButton {
    panel_button
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
    let file_menu = PanelButton::from_label("File");

    let file_menu = self.setup_panel_button(file_menu);

    let file_items = TypedListStore::new();
    file_items.append(&MenuItemModel::new("new", "New File").with_icon("document-new-symbolic"));
    file_items.append(&MenuItemModel::new("open", "Open...").with_icon("document-open-symbolic"));

    let recent_submenu = gtk::gio::ListStore::new::<MenuItemModel>();
    recent_submenu.append(&MenuItemModel::new("recent1", "config.toml"));
    recent_submenu.append(&MenuItemModel::new("recent2", "main.rs"));
    recent_submenu.append(&MenuItemModel::new("recent3", "style.css"));
    let recent_item = MenuItemModel::new("recent", "Recent Files").with_icon("document-open-recent-symbolic");
    recent_item.set_submenu(recent_submenu);
    file_items.append(&recent_item);

    file_items.append(&MenuItemModel::separator());
    file_items.append(&MenuItemModel::new("save", "Save").with_icon("document-save-symbolic"));
    file_items.append(&MenuItemModel::new("save_as", "Save As...").with_icon("document-save-as-symbolic"));
    file_items.append(&MenuItemModel::separator());
    file_items.append(&MenuItemModel::new("quit", "Quit").with_icon("application-exit-symbolic"));

    file_menu.set_menu(file_items);
    self.add_widget_to_left(&file_menu);
    self.panel_buttons.borrow_mut().push(file_menu);

    // Edit menu
    let edit_menu = PanelButton::from_label("Edit");
    let edit_menu = self.setup_panel_button(edit_menu);

    let edit_items = TypedListStore::new();
    edit_items.append(&MenuItemModel::new("undo", "Undo").with_icon("edit-undo-symbolic"));
    edit_items.append(&MenuItemModel::new("redo", "Redo").with_icon("edit-redo-symbolic"));
    edit_items.append(&MenuItemModel::separator());
    edit_items.append(&MenuItemModel::new("cut", "Cut").with_icon("edit-cut-symbolic"));
    edit_items.append(&MenuItemModel::new("copy", "Copy").with_icon("edit-copy-symbolic"));
    edit_items.append(&MenuItemModel::new("paste", "Paste").with_icon("edit-paste-symbolic"));
    edit_items.append(&MenuItemModel::separator());
    edit_items.append(&MenuItemModel::new("find", "Find").with_icon("edit-find-symbolic"));
    edit_items.append(&MenuItemModel::new("replace", "Replace").with_icon("edit-find-replace-symbolic"));

    edit_menu.set_menu(edit_items);
    self.add_widget_to_left(&edit_menu);
    self.panel_buttons.borrow_mut().push(edit_menu);

    // View menu
    let view_menu = PanelButton::from_label("View");
    let view_menu = self.setup_panel_button(view_menu);

    let view_items = TypedListStore::new();
    view_items.append(&MenuItemModel::new("fullscreen", "Fullscreen").with_icon("view-fullscreen-symbolic"));
    view_items.append(&MenuItemModel::new("zoom_in", "Zoom In").with_icon("zoom-in-symbolic"));
    view_items.append(&MenuItemModel::new("zoom_out", "Zoom Out").with_icon("zoom-out-symbolic"));
    view_items.append(&MenuItemModel::new("zoom_reset", "Reset Zoom").with_icon("zoom-original-symbolic"));
    view_items.append(&MenuItemModel::separator());
    view_items.append(&MenuItemModel::new("dark_mode", "Dark Mode")
        .with_icon("weather-clear-night-symbolic")
        .toggled_on());
    view_items.append(&MenuItemModel::new("show_sidebar", "Show Sidebar")
        .with_icon("view-dual-symbolic")
        .toggled_on());

    view_menu.set_menu(view_items);
    self.add_widget_to_left(&view_menu);
    self.panel_buttons.borrow_mut().push(view_menu);
  }

  fn add_secondary_monitor_menus(&mut self) {
    println!("Adding secondary monitor menus (Tools, Debug, Help)");

    // Tools menu
    let tools_menu = PanelButton::from_icon_name_and_label("applications-development-symbolic", "Tools");
    let tools_menu = self.setup_panel_button(tools_menu);

    let tools_items = TypedListStore::new();
    tools_items.append(&MenuItemModel::new("terminal", "Terminal").with_icon("utilities-terminal-symbolic"));
    tools_items.append(&MenuItemModel::new("calculator", "Calculator").with_icon("accessories-calculator-symbolic"));
    tools_items.append(&MenuItemModel::new("text_editor", "Text Editor").with_icon("accessories-text-editor-symbolic"));
    tools_items.append(&MenuItemModel::separator());
    tools_items.append(&MenuItemModel::new("system_monitor", "System Monitor").with_icon("utilities-system-monitor-symbolic"));
    tools_items.append(&MenuItemModel::new("disk_usage", "Disk Usage").with_icon("baobab-symbolic"));

    let network_submenu = gtk::gio::ListStore::new::<MenuItemModel>();
    network_submenu.append(&MenuItemModel::new("ping", "Ping"));
    network_submenu.append(&MenuItemModel::new("traceroute", "Traceroute"));
    network_submenu.append(&MenuItemModel::new("netstat", "Network Status"));
    let network_item = MenuItemModel::new("network", "Network Tools").with_icon("network-wired-symbolic");
    network_item.set_submenu(network_submenu);
    tools_items.append(&network_item);

    tools_menu.set_menu(tools_items);
    self.add_widget_to_left(&tools_menu);
    self.panel_buttons.borrow_mut().push(tools_menu);

    // Debug menu
    let debug_menu = PanelButton::from_icon_name_and_label("applications-debugging-symbolic", "Debug");
    let debug_menu = self.setup_panel_button(debug_menu);

    let debug_items = TypedListStore::new();
    debug_items.append(&MenuItemModel::new("start_debug", "Start Debugging").with_icon("media-playback-start-symbolic"));
    debug_items.append(&MenuItemModel::new("stop_debug", "Stop Debugging").with_icon("media-playback-stop-symbolic"));
    debug_items.append(&MenuItemModel::separator());
    debug_items.append(&MenuItemModel::new("breakpoint", "Toggle Breakpoint").with_icon("media-record-symbolic"));
    debug_items.append(&MenuItemModel::new("step_over", "Step Over").with_icon("go-next-symbolic"));
    debug_items.append(&MenuItemModel::new("step_into", "Step Into").with_icon("go-down-symbolic"));
    debug_items.append(&MenuItemModel::separator());
    debug_items.append(&MenuItemModel::new("verbose_logging", "Verbose Logging")
        .with_icon("text-x-generic-symbolic")
        .toggled_on());

    debug_menu.set_menu(debug_items);
    self.add_widget_to_left(&debug_menu);
    self.panel_buttons.borrow_mut().push(debug_menu);

    // Help menu on the right
    let help_menu = PanelButton::from_icon_name("help-browser-symbolic");
    let help_menu = self.setup_panel_button(help_menu);

    let help_items = TypedListStore::new();
    help_items.append(&MenuItemModel::new("documentation", "Documentation").with_icon("help-contents-symbolic"));
    help_items.append(&MenuItemModel::new("shortcuts", "Keyboard Shortcuts").with_icon("input-keyboard-symbolic"));
    help_items.append(&MenuItemModel::new("tutorial", "Tutorial").with_icon("applications-education-symbolic"));
    help_items.append(&MenuItemModel::separator());
    help_items.append(&MenuItemModel::new("report_bug", "Report Bug").with_icon("tools-report-bug-symbolic"));
    help_items.append(&MenuItemModel::new("about", "About").with_icon("help-about-symbolic"));

    help_menu.set_menu(help_items);
    self.add_widget_to_right(&help_menu);
    self.panel_buttons.borrow_mut().push(help_menu);
  }

  fn add_additional_monitor_menus(&mut self, monitor_index: usize) {
    println!("Adding additional monitor {} menus (System, Monitor Info)", monitor_index);

    // System menu
    let system_menu = PanelButton::from_icon_name_and_label("computer-symbolic", &format!("System {}", monitor_index));
    system_menu.connect_local("menu-item-clicked", false, move |values| {
      let menu_item = values[1].get::<&MenuItemModel>().expect("type checked upstream");

      if(menu_item.allow_toggle()) {
        menu_item.set_toggled(!menu_item.toggled());
      }

      println!(
        "System menu item clicked on monitor {}: {} ({})",
        monitor_index,
        menu_item.text(),
        menu_item.id()
      );

      None
    });
       
    
    let system_menu = self.setup_panel_button(system_menu);

    let system_items = TypedListStore::new();
    system_items.append(&MenuItemModel::new("system_info", "System Information").with_icon("dialog-information-symbolic"));
    system_items.append(&MenuItemModel::new("processes", "Processes").with_icon("utilities-system-monitor-symbolic"));
    system_items.append(&MenuItemModel::new("services", "Services").with_icon("preferences-system-services-symbolic"));
    system_items.append(&MenuItemModel::separator());

    let power_submenu = gtk::gio::ListStore::new::<MenuItemModel>();
    power_submenu.append(&MenuItemModel::new("suspend", "Suspend"));
    power_submenu.append(&MenuItemModel::new("hibernate", "Hibernate"));
    power_submenu.append(&MenuItemModel::new("restart", "Restart"));
    power_submenu.append(&MenuItemModel::new("shutdown", "Shutdown"));
    let power_item = MenuItemModel::new("power", "Power Options").with_icon("system-shutdown-symbolic");
    power_item.set_submenu(power_submenu);
    system_items.append(&power_item);

    system_items.append(&MenuItemModel::separator());
    system_items.append(&MenuItemModel::new("auto_update", "Auto Updates")
        .with_icon("software-update-available-symbolic")
        .toggled_on());

    system_menu.set_menu(system_items);
    self.add_widget_to_left(&system_menu);
    self.panel_buttons.borrow_mut().push(system_menu);

    // Monitor info on the right
    let monitor_menu = PanelButton::from_icon_name_and_label("video-display-symbolic", &format!("Monitor {}", monitor_index));
    let monitor_menu = self.setup_panel_button(monitor_menu);

    let monitor_items = TypedListStore::new();
    monitor_items.append(&MenuItemModel::new("display_settings", "Display Settings").with_icon("preferences-desktop-display-symbolic"));
    monitor_items.append(&MenuItemModel::new("resolution", "Resolution").with_icon("preferences-desktop-screensaver-symbolic"));
    monitor_items.append(&MenuItemModel::new("brightness", "Brightness").with_icon("display-brightness-symbolic"));
    monitor_items.append(&MenuItemModel::separator());
    monitor_items.append(&MenuItemModel::new("wallpaper", "Wallpaper").with_icon("preferences-desktop-wallpaper-symbolic"));
    monitor_items.append(&MenuItemModel::new("screensaver", "Screensaver").with_icon("preferences-desktop-screensaver-symbolic"));

    monitor_menu.set_menu(monitor_items);
    self.add_widget_to_right(&monitor_menu);
    self.panel_buttons.borrow_mut().push(monitor_menu);
  }
}
