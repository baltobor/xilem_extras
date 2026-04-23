//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Sheet widget for modal dialogs.
//!
//! This module provides a `Sheet` widget that displays content in a modal
//! overlay with a semi-transparent backdrop. The sheet can be dismissed
//! by clicking the backdrop or pressing ESC.
//!
//! # Example
//!
//! ```ignore
//! use xilem_extras::sheet;
//!
//! // Conditionally show a sheet when `show_modal` is true
//! if model.show_modal {
//!     sheet(
//!         flex_col((
//!             label("Modal Title"),
//!             label("Modal content here"),
//!             button("Close", |state: &mut AppState| {
//!                 state.show_modal = false;
//!             }),
//!         )),
//!         |state: &mut AppState| {
//!             state.show_modal = false;
//!         },
//!     )
//! }
//! ```

mod widget;
mod layer;
mod view;

pub use widget::{SheetWidget, SheetAction};
pub use layer::SheetLayer;
pub use view::{sheet, SheetView};
