//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Dropdown select widget for choosing from a list of options.
//!
//! This module provides a `DropdownSelect` widget that displays the currently
//! selected value and opens a dropdown list when clicked. It uses the Layer
//! trait for floating dropdown positioning.
//!
//! # Example
//!
//! ```ignore
//! use xilem_extras::dropdown_select;
//!
//! dropdown_select(
//!     &model.selected_option,
//!     vec!["Option A", "Option B", "Option C"],
//!     |state: &mut AppState, selected: &str| {
//!         state.selected_option = selected.to_string();
//!     },
//! )
//! ```

mod widget;
mod dropdown;
mod option_item;
mod view;

pub use widget::{DropdownSelect, DropdownSelectAction};
pub use dropdown::SelectDropdown;
pub use option_item::SelectOptionItem;
pub use view::{dropdown_select, DropdownSelectView};
