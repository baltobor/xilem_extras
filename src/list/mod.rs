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
//! - Keyboard navigation with arrow keys
//! - Optional sections with sticky headers
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
mod widget;
mod list_widget_view;

pub use list_view::{list, list_styled, ListAction, ListStyle};
pub use widget::{
    ListWidget,
    ListWidgetAction,
    ListWidgetStyle,
    ListScrollState,
    ListSection,
    ListRangeAction,
    ListRowAction,
};
pub use list_widget_view::{
    // Full-featured virtualized list
    list_view,
    list_view_styled,
    ListView,
    ListViewState,
    ListViewAction,
    ListViewStyle,
    // Sectioned list
    list_view_sectioned,
    SectionDef,
    SectionedRowInfo,
    SectionedListView,
    SectionedListViewState,
    // Simple navigable list (legacy)
    list_navigable,
    ListNavigableView,
};
