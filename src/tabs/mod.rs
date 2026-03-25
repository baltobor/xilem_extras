//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Tab bar component for document-style interfaces.
//!
//! Provides a scrollable tab strip with navigation buttons, close buttons,
//! and dirty state indicators.
//!
//! # Features
//!
//! - Horizontal scrolling via portal (touchpad support)
//! - Previous/Next navigation buttons
//! - Per-tab close button
//! - Dirty state indicator (asterisk)
//! - Active/inactive visual states
//!
//! # Example
//!
//! ```ignore
//! use xilem_extras::tabs::{TabBar, TabItem};
//!
//! struct MyTab {
//!     title: String,
//!     dirty: bool,
//! }
//!
//! impl TabItem for MyTab {
//!     fn title(&self) -> &str { &self.title }
//!     fn is_dirty(&self) -> bool { self.dirty }
//! }
//!
//! fn tab_strip(model: &mut AppModel) -> impl WidgetView<AppModel> {
//!     TabBar::new(&model.tabs, model.active_tab)
//!         .on_select(|model, idx| model.active_tab = idx)
//!         .on_close(|model, idx| model.close_tab(idx))
//! }
//! ```

mod tab_item;
mod tab_bar;

pub use tab_item::{TabItem, SimpleTab};
pub use tab_bar::{TabBar, TabBarColors};
