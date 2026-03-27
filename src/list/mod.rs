//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! List view for flat collections with selection support.
//!
//! Provides a SwiftUI-style `list` view that handles:
//! - Rendering items from a collection
//! - Selection with Cmd+click and Shift+click support
//! - Double-click activation
//! - Hover highlighting
//!
//! # Example
//!
//! ```ignore
//! use xilem_extras::{list, ListAction, ListStyle};
//!
//! list_styled(
//!     &model.contacts,
//!     &model.selection,
//!     ListStyle::new().hover_bg(BG_HOVER),
//!     |contact, is_selected| {
//!         contact_row(contact, is_selected)
//!     },
//!     |state, action| {
//!         match action {
//!             ListAction::Select(id, mods) => state.selection.select(id, mods),
//!             ListAction::Activate(id) => state.open_contact(&id),
//!         }
//!     },
//! )
//! ```

mod list_view;

pub use list_view::{list, list_styled, ListAction, ListStyle};
