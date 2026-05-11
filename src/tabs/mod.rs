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
//! The X button is on the left side **on purpose**!
//! When you close a tab, the tab next to it moves over from the right. 
//! What I really hate about almost all other tab layouts is when the tab moves 
//! over and the close button ends up in a different spot. 
//! 
//! Because the X is on the left side, the tab's close button moves of the 
//! right tab moves over, and the new close button appears directly under 
//! the mouse pointer. This means that if you want to quickly close several 
//! tabs next to each other, you can keep the mouse in one place and simply 
//! click multiple times. Tabs resize with the text length. And if tabs change 
//! size along with the text, a close icon on the right side would shift 
//! position along with the text. This results in the mouse pointer moving 
//! very slowly when you want to close several tabs in a row one ofter another. 
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
