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
//! use xilem_extras::menu_button;
//!
//! menu_button(
//!     label("File"),
//!     vec!["New", "Open...", "---", "Save", "Save As...", "---", "Exit"],
//!     |state: &mut AppState, index: usize| {
//!         match index {
//!             0 => state.new_file(),
//!             1 => state.open_file(),
//!             // index 2 is separator, skipped
//!             3 => state.save_file(),
//!             4 => state.save_as(),
//!             // index 5 is separator, skipped
//!             6 => state.exit(),
//!             _ => {}
//!         }
//!     },
//! )
//! ```
//!
//! Use `"---"` in the items list to insert a visual separator between groups.

mod widget;
mod dropdown;
mod menu_item;
mod separator;
mod view;

pub use widget::{MenuButton, MenuButtonPress};
pub use dropdown::MenuDropdown;
pub use menu_item::{PulldownMenuItem, DEFAULT_ITEM_HEIGHT};
pub use separator::MenuSeparator;
pub use view::{menu_button, MenuButtonView};
