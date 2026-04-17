//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Unified application menu bar for xilem.
//!
//! This module provides a single declarative API for defining application menus that:
//! - Uses **muda** for native menus on macOS/Windows
//! - Falls back to a custom menu bar widget on Linux
//! - Integrates with xilem's View lifecycle for reactive updates
//!
//! **Important:** This is for the **application menu bar** (File, Edit, View...).
//! For in-app dropdown menus (toolbars, context menus), use `menu_button` instead.
//!
//! # Example
//!
//! ```ignore
//! use xilem_extras::app_menu::{app_menu_bar, Key, CMD, SHIFT};
//!
//! fn build_menu(model: &AppModel) -> impl WidgetView<AppModel> {
//!     app_menu_bar(|m| m
//!         .menu("File", |m| m
//!             .item("New", |s: &mut AppModel| s.new_file())
//!                 .shortcut(CMD + Key::N)
//!             .item("Open...", |s| s.open_file())
//!                 .shortcut(CMD + Key::O)
//!             .separator()
//!             .item("Quit", |s| s.quit())
//!                 .shortcut(CMD + Key::Q)
//!         )
//!         .menu("Edit", |m| m
//!             .item("Undo", |s| s.undo())
//!                 .shortcut(CMD + Key::Z)
//!                 .enabled(|s| s.can_undo())
//!             .item("Redo", |s| s.redo())
//!                 .shortcut(CMD + SHIFT + Key::Z)
//!                 .enabled(|s| s.can_redo())
//!         )
//!     )
//! }
//! ```

mod shortcut;
mod builder;
mod view;
mod native;
mod action_trait;
mod render;

pub use shortcut::{Key, Modifiers, Shortcut, CMD, SHIFT, ALT, CTRL};
pub use builder::{MenuBarBuilder, MenuBuilder, MenuItemBuilder, MenuItemChain};
pub use view::{app_menu_bar, AppMenuBarView, with_app_menu, menu_bar_label};
pub use action_trait::{MenuActionTrait, MenuActionHandler};
pub use render::render_menu_bar;

// Re-export muda and build_muda_menu for native menus (macOS/Windows only)
#[cfg(all(feature = "app-menu", not(target_os = "linux")))]
pub use muda;
#[cfg(all(feature = "app-menu", not(target_os = "linux")))]
pub use native::build_muda_menu;
