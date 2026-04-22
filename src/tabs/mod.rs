//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Tab bar components for different interface styles.
//!
//! This module provides two tab bar variants:
//!
//! - [`TabBar`] - Document-style tabs with close buttons and dirty indicators
//! - [`NavTabBar`] - Navigation tabs for fixed view switching
//!
//! # TabBar (Document Tabs)
//!
//! For document management interfaces (like IDE editors or browsers):
//! - Closable tabs with X button
//! - Dirty state indicator (asterisk)
//! - Scrollable with navigation arrows
//!
//! ```ignore
//! use xilem_extras::tabs::{TabBar, TabItem};
//!
//! TabBar::new(&model.tabs, model.active_tab)
//!     .on_select(|model, idx| model.active_tab = idx)
//!     .on_close(|model, idx| model.close_tab(idx))
//!     .build()
//! ```
//!
//! # NavTabBar (Navigation Tabs)
//!
//! For fixed navigation between views (like settings panels or dialogs):
//! - No close buttons (tabs are fixed)
//! - No dirty indicators
//! - Optional navigation arrows
//! - Simpler API
//!
//! ```ignore
//! use xilem_extras::tabs::{NavTabBar, SimpleTab};
//!
//! let tabs = vec![
//!     SimpleTab::new("Overview"),
//!     SimpleTab::new("Details"),
//!     SimpleTab::new("Settings"),
//! ];
//!
//! NavTabBar::new(&tabs, model.active_view)
//!     .on_select(|model, idx| model.active_view = idx)
//!     .build()
//! ```

mod tab_item;
mod tab_bar;
mod nav_tab_bar;

pub use tab_item::{TabItem, SimpleTab};
pub use tab_bar::{TabBar, TabBarColors};
pub use nav_tab_bar::{NavTabBar, NavButtonMode};
