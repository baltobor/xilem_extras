//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! # xilem_extras
//!
//! High-level widget library for Xilem providing Tree, List, Table, and Popup widgets.
//!
//! ## Overview
//!
//! This library extends Xilem with common UI patterns:
//!
//! - **Tree View** - Hierarchical data with expand/collapse
//! - **List View** - Selectable lists with sections
//! - **Table View** - Sortable data grids
//! - **Popup Menu** - Context menus and dropdowns
//! - **Row Button** - Clickable list rows with hover states
//!
//! ## Structure
//!
//! - [`masonry`] - Pure Masonry widget implementations (backend).
//! - [`xilem_masonry`] - Bridge views wrapping the Masonry widgets for use in Xilem.
//! - [`xilem`] - Pure Xilem view compositions and the public API surface. Every
//!   widget's Xilem-facing API lives here, e.g. `xilem_extras::xilem::table::row_button`.
//!
//! ## Example
//!
//! ```ignore
//! use xilem_extras::xilem::table::row_button;
//! use xilem_extras::xilem::traits::{SelectionState, SelectionModifiers};
//!
//! fn item_row(item: &Item, selected: bool) -> impl WidgetView<AppModel> {
//!     let row = flex_row((label(item.name.clone()),));
//!
//!     row_button(row, move |model: &mut AppModel| {
//!         model.selection.select(item.id, SelectionModifiers::NONE);
//!     })
//!     .hover_bg(Color::from_rgb8(60, 60, 60))
//! }
//! ```

pub mod masonry;
pub mod xilem;
pub mod xilem_masonry;

pub mod locale;

pub use crate::masonry::flow_direction::FlowDirection;
pub use ::xilem as xilem_crate;
pub use masonry_winit;
