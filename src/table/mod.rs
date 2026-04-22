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
mod table_view;
mod widget;

pub use column::{column, ColumnBuilder, ColumnDef, Alignment, ColumnWidth};
pub use column_widths::ColumnWidths;
pub use resizable_header::{resizable_header, ResizableHeaderView, ResizableHeader, ColumnResizeAction};
pub use sort_state::{SortOrder, SortDirection, SortDescriptor};
pub use state::TableScrollState;

// Main table API (virtualized, high-performance)
pub use table_view::{table, table_styled, TableAction, TableView, TableViewState};

// Legacy table API (non-virtualized, for backward compatibility)
pub use legacy_table::{legacy_table, legacy_table_styled, LegacyTableAction, TableStyle};

// Widget-level exports
pub use widget::{TableWidget, TableWidgetAction, TableRangeAction, TableRowClickAction, TableHeaderClickAction};
