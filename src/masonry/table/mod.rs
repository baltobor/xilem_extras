// Copyright 2026 the Xilem Authors
// SPDX-License-Identifier: Apache-2.0

//! Masonry widget implementations for table.

pub(crate) mod column_layout;
pub mod resizable_header;
pub mod widget;

pub use column_layout::{ColumnResizeMode, visual_index};
pub use resizable_header::{
    ColumnDividerHighlightAction, ColumnResizeAction, ColumnResizePreviewAction, ResizableHeader,
};
pub use widget::{
    TableHeaderClickAction, TableRangeAction, TableRowClickAction, TableWidget, TableWidgetAction,
};
