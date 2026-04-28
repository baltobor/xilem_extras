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
//! The sheet automatically sizes to its content plus padding and centers
//! on screen.
//!
//! # Example
//!
//! ```ignore
//! use xilem::view::{flex_col, label, button, zstack};
//! use xilem_extras::sheet;
//!
//! // Conditionally show a sheet when `show_modal` is true
//! // Place the sheet at root level using zstack
//! fn app_view(model: &mut AppState) -> impl WidgetView<AppState> {
//!     let main_content = /* your main content */;
//!
//!     if model.show_modal {
//!         zstack((
//!             main_content,
//!             sheet(
//!                 flex_col((
//!                     label("Modal Title"),
//!                     label("Modal content here"),
//!                     button("Close", |state: &mut AppState| {
//!                         state.show_modal = false;
//!                     }),
//!                 )),
//!                 |state: &mut AppState| {
//!                     state.show_modal = false;
//!                 },
//!             ),
//!         ))
//!     } else {
//!         main_content
//!     }
//! }
//! ```

mod widget;
mod view;

pub use widget::{SheetWidget, SheetAction};
pub use view::{sheet, SheetView};

// Layer is kept for backward compatibility but not actively used
mod layer;
pub use layer::SheetLayer;
