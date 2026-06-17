//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Pulldown menu button widget for creating application menus.
//!
//! This module provides a `MenuButton` widget that displays a clickable label
//! and opens a dropdown menu when activated. It follows the Layer pattern from
//! masonry for floating dropdown positioning.
//!
//! # Example
//!
//! ```ignore
//! use xilem_extras::{menu_button, menu_item, separator};
//!
//! menu_button(
//!     label("File"),
//!     (
//!         menu_item("New", |state: &mut AppState| state.new_file()),
//!         menu_item("Open...", |state| state.open_file()),
//!         separator(),
//!         menu_item("Save", |state| state.save_file()),
//!         menu_item("Exit", |state| state.exit()),
//!     ),
//! )
//! ```

mod dropdown;
mod menu_item;
mod separator;
mod submenu_item;
mod view;
mod widget;

pub use dropdown::{MenuDropdown, SubmenuDropdown};
pub use menu_item::{DEFAULT_ITEM_HEIGHT, PulldownMenuItem};
pub use separator::MenuSeparator;
pub use submenu_item::PulldownSubmenuItem;
pub use view::{MenuButtonView, menu_button};
pub use widget::{MenuButton, MenuButtonPress, MenuItemData};
