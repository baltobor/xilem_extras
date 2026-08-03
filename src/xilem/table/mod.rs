//! Table widget module.

mod column;
mod column_widths;
mod direction_detect;
mod row_cells;
mod sort_state;
mod state;
mod table_cell;
mod table_view;

pub use column::{Alignment, ColumnBuilder, ColumnDef, ColumnWidth, column};
pub use column_widths::ColumnWidths;
pub use row_cells::row_cells;
pub use sort_state::{SortDescriptor, SortDirection, SortOrder};
pub use state::TableScrollState;

pub use table_cell::table_cell;
pub use table_view::{TableAction, TableStyle, TableView, TableViewState, table, table_styled};

pub use crate::masonry::table::{
    ColumnLayoutAction, ColumnResizeAction, ResizableHeader, TableHeaderClickAction,
    TableRangeAction, TableRowClickAction, TableWidget, TableWidgetAction,
};
pub use crate::xilem_masonry::table::{ResizableHeaderView, resizable_header};
