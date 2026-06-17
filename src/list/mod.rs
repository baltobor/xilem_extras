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
mod list_widget_view;
mod widget;

pub use list_view::{ListAction, ListStyle, list, list_styled};
pub use list_widget_view::{
    ListNavigableView,
    ListView,
    ListViewAction,
    ListViewState,
    ListViewStyle,
    SectionDef,
    SectionedListView,
    SectionedListViewState,
    SectionedRowInfo,
    // Simple navigable list (legacy)
    list_navigable,
    // Full-featured virtualized list
    list_view,
    // Sectioned list
    list_view_sectioned,
    list_view_styled,
};
pub use widget::{
    ListRangeAction, ListRowAction, ListScrollState, ListSection, ListWidget, ListWidgetAction,
    ListWidgetStyle,
};
