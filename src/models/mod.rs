pub mod config;
//remove this once dropdown_menu_button is removed
pub mod menu_item;
pub mod menu_item_model;

pub use config::{CurtainBarConfig, Margins};
//remove this once dropdown_menu_button is removed
pub use menu_item::MenuItem;
pub use menu_item_model::MenuItemModel;
