use gtk::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};

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

    pub fn build(self, app: &gtk::Application) -> Result<CurtainBar, Box<dyn std::error::Error>> {
        CurtainBar::with_config(app, self.config)
    }
}

pub struct CurtainBar {
    window: gtk::ApplicationWindow,
    left_box: gtk::Box,
    center_box: gtk::Box,
    right_box: gtk::Box,
}

impl CurtainBar {
    pub fn builder() -> CurtainBarBuilder {
        CurtainBarBuilder::new()
    }

    pub fn new(app: &gtk::Application) -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_config(app, CurtainBarConfig::default())
    }

    fn with_config(app: &gtk::Application, config: CurtainBarConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("Curtain Bar")
            .build();

        Self::configure_layer_shell(&window, &config)?;
        
        let (left_box, center_box, right_box) = Self::create_layout(config.spacing);
        let panel_box = Self::create_panel_container(&left_box, &center_box, &right_box, &config.margins);
        
        window.set_child(Some(&panel_box));

        Ok(Self {
            window,
            left_box,
            center_box,
            right_box,
        })
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
}