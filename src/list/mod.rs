//! List view for flat collections with selection support.

mod list_view;

pub use list_view::{ListAction, ListStyle, list, list_styled};

pub use crate::masonry::list::{
    ListRangeAction, ListRowAction, ListScrollState, ListSection, ListWidget, ListWidgetAction,
    ListWidgetStyle,
};

pub use crate::xilem_masonry::list::{
    ListNavigableView, ListViewAction, ListViewState, ListViewStyle,
    ListView, SectionDef, SectionedListView, SectionedListViewState, SectionedRowInfo,
    list_navigable, list_view, list_view_sectioned, list_view_styled,
};
