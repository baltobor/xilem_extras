// Copyright 2026 the Xilem Authors
// SPDX-License-Identifier: Apache-2.0

//! Masonry widget implementations for table.

pub mod resizable_header;
pub mod widget;

pub use resizable_header::{ColumnResizeAction, ResizableHeader};
pub use widget::{
    TableHeaderClickAction, TableRangeAction, TableRowClickAction, TableWidget, TableWidgetAction,
};
