//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

mod column;
mod column_widths;
mod legacy_table;
mod resizable_header;
mod sort_state;
mod state;
mod table_cell;
mod table_view;
mod widget;

pub use column::{Alignment, ColumnBuilder, ColumnDef, ColumnWidth, column};
pub use column_widths::ColumnWidths;
pub use resizable_header::{
    ColumnResizeAction, ResizableHeader, ResizableHeaderView, resizable_header,
};
pub use sort_state::{SortDescriptor, SortDirection, SortOrder};
pub use state::TableScrollState;

// Main table API (virtualized, high-performance)
pub use table_cell::table_cell;
pub use table_view::{TableAction, TableView, TableViewState, table, table_styled};

// Legacy table API (non-virtualized, for backward compatibility)
pub use legacy_table::{LegacyTableAction, TableStyle, legacy_table, legacy_table_styled};

// Widget-level exports
pub use widget::{
    TableHeaderClickAction, TableRangeAction, TableRowClickAction, TableWidget, TableWidgetAction,
};
