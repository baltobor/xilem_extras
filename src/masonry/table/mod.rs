// Copyright 2026 the Xilem Authors
// SPDX-License-Identifier: Apache-2.0

//! Masonry widget implementations for table.

pub(crate) mod column_layout;
pub mod resizable_header;
pub(crate) mod row_cells;
pub mod widget;

pub use column_layout::{ColumnBox, ColumnResizeMode};
pub use resizable_header::{ColumnLayoutAction, ColumnResizeAction, ResizableHeader};
pub use widget::{
    TableHeaderClickAction, TableRangeAction, TableRowClickAction, TableWidget, TableWidgetAction,
};
