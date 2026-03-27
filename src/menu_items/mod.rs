//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! SwiftUI-style type-safe menu item API.
//!
//! This module provides a type-safe, action-per-item API for building menus.
//! Each menu item carries its own action callback, eliminating index matching errors.
//!
//! # Example
//!
//! ```ignore
//! use xilem_extras::menu_items::{menu_item, separator};
//!
//! context_menu(child, (
//!     menu_item("Open", |s: &mut State| s.open()),
//!     menu_item("Delete", |s| s.delete()),
//!     separator(),
//!     menu_item("Rename", |s| s.rename()),
//! ))
//! ```

mod entry;
mod item;
mod separator;
mod sequence;

pub use entry::{MenuEntry, BoxedMenuEntry};
pub use item::{MenuItem, menu_item};
pub use separator::{SeparatorEntry, separator};
pub use sequence::MenuItems;
