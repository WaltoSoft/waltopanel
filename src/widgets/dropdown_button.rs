use gtk::prelude::*;
use gtk::gdk;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub is_toggled: bool,
    pub is_separator: bool,
    pub submenu: Option<Vec<MenuItem>>,
}

impl MenuItem {
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            icon: None,
            is_toggled: false,
            is_separator: false,
            submenu: None,
        }
    }

    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }


    pub fn toggled(mut self) -> Self {
        self.is_toggled = true;
        self
    }

    pub fn with_submenu(mut self, submenu: Vec<MenuItem>) -> Self {
        self.submenu = Some(submenu);
        self
    }

    pub fn separator() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            icon: None,
            is_toggled: false,
            is_separator: true,
            submenu: None,
        }
    }
}

#[derive(Clone)]
pub struct DropdownButton {
    button: gtk::Button,
    popover: gtk::Popover,
    pub menu_items: Rc<RefCell<Vec<MenuItem>>>,
    pub on_item_selected: Rc<RefCell<Option<Box<dyn Fn(&str, bool) + 'static>>>>,
    focused_item_index: Rc<RefCell<Option<usize>>>,
    menu_containers: Rc<RefCell<Vec<gtk::Box>>>,
}

impl DropdownButton {
    pub fn new() -> Self {
        let button = gtk::Button::new();
        let popover = gtk::Popover::builder()
            .autohide(true)
            .has_arrow(false)
            .position(gtk::PositionType::Bottom)
            .can_focus(true)
            .focusable(true)
            .build();
        
        popover.set_parent(&button);
        
        let menu_items = Rc::new(RefCell::new(Vec::new()));
        
        let popover_clone = popover.clone();
        button.connect_clicked(move |_| {
            popover_clone.popup();
            popover_clone.grab_focus();
        });

        let dropdown_button = Self {
            button: button.clone(),
            popover: popover.clone(),
            menu_items: menu_items.clone(),
            on_item_selected: Rc::new(RefCell::new(None)),
            focused_item_index: Rc::new(RefCell::new(None)),
            menu_containers: Rc::new(RefCell::new(Vec::new())),
        };

        dropdown_button.setup_keyboard_navigation();
        dropdown_button
    }
    

    pub fn with_text(self, text: &str) -> Self {
        self.button.set_label(text);
        self
    }

    pub fn with_icon(self, icon_name: &str) -> Self {
        let icon = gtk::Image::from_icon_name(icon_name);
        self.button.set_child(Some(&icon));
        self
    }

    pub fn with_icon_and_text(self, icon_name: &str, text: &str) -> Self {
        let container = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(6)
            .build();
        
        let icon = gtk::Image::from_icon_name(icon_name);
        let label = gtk::Label::new(Some(text));
        
        container.append(&icon);
        container.append(&label);
        
        self.button.set_child(Some(&container));
        self
    }

    pub fn set_menu_items(&self, items: Vec<MenuItem>) {
        *self.menu_items.borrow_mut() = items;
        self.rebuild_menu();
    }

    pub fn on_item_toggled<F>(self, callback: F) -> Self 
    where
        F: Fn(&str, bool) + 'static,
    {
        *self.on_item_selected.borrow_mut() = Some(Box::new(callback));
        self
    }
    
    fn setup_keyboard_navigation(&self) {
        let key_controller = gtk::EventControllerKey::new();
        let focused_index = self.focused_item_index.clone();
        let menu_items = self.menu_items.clone();
        let menu_containers = self.menu_containers.clone();
        let callback = self.on_item_selected.clone();
        let popover = self.popover.clone();
        
        key_controller.connect_key_pressed(move |_, key, _, _| {
            println!("Key pressed: {:?}", key);
            let (non_separator_count, item_id_to_callback) = {
                let items = menu_items.borrow();
                let non_separator_items: Vec<_> = items.iter().enumerate()
                    .filter(|(_, item)| !item.is_separator)
                    .collect();
                
                if non_separator_items.is_empty() {
                    return false.into();
                }
                
                // For Enter key, get the item ID we need to callback
                let item_id = if matches!(key, gdk::Key::Return | gdk::Key::KP_Enter) {
                    if let Some(focused_idx) = *focused_index.borrow() {
                        non_separator_items.get(focused_idx).map(|(_, item)| item.id.clone())
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                (non_separator_items.len(), item_id)
            }; // Drop the borrow here
            
            match key {
                gdk::Key::Down => {
                    let mut current_focused = focused_index.borrow_mut();
                    let new_index = match *current_focused {
                        None => 0,
                        Some(idx) => (idx + 1) % non_separator_count,
                    };
                    *current_focused = Some(new_index);
                    drop(current_focused);
                    Self::update_visual_focus(&menu_containers, &[], Some(new_index));
                    true.into()
                },
                gdk::Key::Up => {
                    let mut current_focused = focused_index.borrow_mut();
                    let new_index = match *current_focused {
                        None => non_separator_count - 1,
                        Some(0) => non_separator_count - 1,
                        Some(idx) => idx - 1,
                    };
                    *current_focused = Some(new_index);
                    drop(current_focused);
                    Self::update_visual_focus(&menu_containers, &[], Some(new_index));
                    true.into()
                },
                gdk::Key::Return | gdk::Key::KP_Enter => {
                    if let Some(item_id) = item_id_to_callback {
                        popover.popdown();
                        if let Some(cb) = callback.borrow().as_ref() {
                            cb(&item_id, false);
                        }
                    }
                    true.into()
                },
                gdk::Key::Escape => {
                    popover.popdown();
                    true.into()
                },
                _ => false.into(),
            }
        });
        
        self.popover.add_controller(key_controller);
        
        // Reset focus when popover opens
        let focused_index_clone = self.focused_item_index.clone();
        let menu_containers_clone = self.menu_containers.clone();
        let menu_items_clone = self.menu_items.clone();
        
        self.popover.connect_show(move |_| {
            *focused_index_clone.borrow_mut() = None;
            let items = menu_items_clone.borrow();
            let non_separator_items: Vec<_> = items.iter().enumerate()
                .filter(|(_, item)| !item.is_separator)
                .collect();
            Self::update_visual_focus(&menu_containers_clone, &non_separator_items, None);
        });
    }
    
    fn update_visual_focus(
        menu_containers: &Rc<RefCell<Vec<gtk::Box>>>,
        _non_separator_items: &[(usize, &MenuItem)],
        focused_index: Option<usize>
    ) {
        let containers = menu_containers.borrow();
        println!("Updating focus, containers: {}, focused: {:?}", containers.len(), focused_index);
        
        for (container_idx, container) in containers.iter().enumerate() {
            if let Some(focused_idx) = focused_index {
                if container_idx == focused_idx {
                    println!("Setting focus styling for container {}", container_idx);
                    // Use GTK selection colors
                    let css_provider = gtk::CssProvider::new();
                    css_provider.load_from_data("* { background-color: @theme_selected_bg_color; color: @theme_selected_fg_color; }");
                    container.style_context().add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1);
                } else {
                    // Remove any custom styling
                    let css_provider = gtk::CssProvider::new();
                    css_provider.load_from_data("* { background-color: transparent; }");
                    container.style_context().add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1);
                }
            } else {
                // Remove any custom styling
                let css_provider = gtk::CssProvider::new();
                css_provider.load_from_data("* { background-color: transparent; }");
                container.style_context().add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1);
            }
        }
    }
    
    pub fn toggle_item(&self, item_id: &str) -> bool {
        let new_state = {
            let mut items = self.menu_items.borrow_mut();
            if let Some(item) = items.iter_mut().find(|i| i.id == item_id) {
                item.is_toggled = !item.is_toggled;
                item.is_toggled
            } else {
                false
            }
        };
        self.rebuild_menu();
        new_state
    }
    
    pub fn set_item_toggled(&self, item_id: &str, toggled: bool) {
        {
            let mut items = self.menu_items.borrow_mut();
            if let Some(item) = items.iter_mut().find(|i| i.id == item_id) {
                item.is_toggled = toggled;
            }
        }
        self.rebuild_menu();
    }

    fn rebuild_menu(&self) {
        if let Some(_child) = self.popover.child() {
            self.popover.set_child(gtk::Widget::NONE);
        }

        let items = self.menu_items.borrow();
        if items.is_empty() {
            return;
        }

        let menu_box = self.create_menu_container(&items);
        self.popover.set_child(Some(&menu_box));
    }

    fn create_menu_container(&self, items: &[MenuItem]) -> gtk::Widget {
        let menu_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(0)
            .css_classes(vec!["dropdown-menu".to_string()])
            .build();

        let mut containers = Vec::new();

        for item in items {
            if item.is_separator {
                let separator = gtk::Separator::builder()
                    .orientation(gtk::Orientation::Horizontal)
                    .css_classes(vec!["dropdown-separator".to_string()])
                    .build();
                menu_box.append(&separator);
            } else {
                let menu_item = self.create_menu_item(item);
                // Store the container for keyboard navigation
                if let Some(container) = menu_item.downcast_ref::<gtk::Box>() {
                    containers.push(container.clone());
                }
                menu_box.append(&menu_item);
            }
        }

        *self.menu_containers.borrow_mut() = containers;
        menu_box.upcast()
    }

    fn create_menu_item(&self, item: &MenuItem) -> gtk::Widget {
        let item_container = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .css_classes(vec!["dropdown-item".to_string()])
            .build();

        if item.is_toggled {
            item_container.add_css_class("toggled");
        }

        let event_controller = gtk::GestureClick::new();
        item_container.add_controller(event_controller.clone());

        let content_grid = gtk::Grid::builder()
            .column_spacing(12)
            .build();

        let mut col = 0;

        let icon_placeholder = gtk::Box::builder()
            .width_request(16)
            .height_request(16)
            .build();

        if item.is_toggled {
            let checkmark = gtk::Image::from_icon_name("object-select-symbolic");
            checkmark.set_pixel_size(16);
            content_grid.attach(&checkmark, col, 0, 1, 1);
        } else if let Some(icon_name) = &item.icon {
            let icon = gtk::Image::from_icon_name(icon_name);
            icon.set_pixel_size(16);
            content_grid.attach(&icon, col, 0, 1, 1);
        } else {
            content_grid.attach(&icon_placeholder, col, 0, 1, 1);
        }
        col += 1;

        let label = gtk::Label::builder()
            .label(&item.label)
            .halign(gtk::Align::Start)
            .hexpand(true)
            .build();
        content_grid.attach(&label, col, 0, 1, 1);
        col += 1;

        // Right-aligned column for submenu arrows
        if item.submenu.is_some() {
            let submenu_arrow = gtk::Image::from_icon_name("go-next-symbolic");
            submenu_arrow.set_pixel_size(12);
            submenu_arrow.set_halign(gtk::Align::End);
            content_grid.attach(&submenu_arrow, col, 0, 1, 1);
        } else {
            let arrow_placeholder = gtk::Box::builder()
                .width_request(16)
                .build();
            content_grid.attach(&arrow_placeholder, col, 0, 1, 1);
        }

        item_container.append(&content_grid);

        let item_id = item.id.clone();
        let popover = self.popover.clone();
        let callback_ref = self.on_item_selected.clone();
        
        event_controller.connect_pressed(move |_, _, _, _| {
            popover.popdown();
            
            if let Some(callback) = callback_ref.borrow().as_ref() {
                callback(&item_id, false); // For now, just report the click
            }
        });

        item_container.upcast()
    }

    pub fn widget(&self) -> &gtk::Button {
        &self.button
    }
}

impl Default for DropdownButton {
    fn default() -> Self {
        Self::new()
    }
}