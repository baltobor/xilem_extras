//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! # xilem_extras
//!
//! High-level widget library for Xilem providing Tree, List, Table, and Popup widgets.
//!
//! ## Overview
//!
//! This library extends Xilem with common UI patterns:
//!
//! - **Tree View** - Hierarchical data with expand/collapse
//! - **List View** - Selectable lists with sections
//! - **Table View** - Sortable data grids
//! - **Popup Menu** - Context menus and dropdowns
//! - **Row Button** - Clickable list rows with hover states
//!
//! ## Core Traits
//!
//! - [`Identifiable`] - Stable identity for diffing
//! - [`TreeNode`] - Hierarchical data structure
//! - [`TableRow`] - Table data with column access
//! - [`SelectionState`] - Selection strategy pattern
//!
//! ## Selection Types
//!
//! - [`SingleSelection`] - Single item selection
//! - [`MultiSelection`] - Multi-select with Cmd/Shift modifiers
//!
//! ## Example
//!
//! ```ignore
//! use xilem_extras::{row_button, SelectionState, SelectionModifiers};
//!
//! fn item_row(item: &Item, selected: bool) -> impl WidgetView<AppModel> {
//!     let row = flex_row((label(item.name.clone()),));
//!
//!     row_button(row, move |model: &mut AppModel| {
//!         model.selection.select(item.id, SelectionModifiers::NONE);
//!     })
//!     .hover_bg(Color::from_rgb8(60, 60, 60))
//! }
//! ```

mod traits;
mod selection;
pub mod components;
mod tree;
mod list;
mod table;
pub mod menu_button;
pub mod dropdown_select;
pub mod tabs;
pub mod theme;

pub use traits::{
    Identifiable,
    TreeNode,
    ListItem,
    TableRow,
    CellValue,
    SelectionState,
    SelectionModifiers,
};

pub use selection::{
    SingleSelection,
    MultiSelection,
};

pub use components::{
    icon,
    icon_sm,
    icon_md,
    icon_lg,
    Icon,
    row_button,
    row_button_with_clicks,
    row_button_with_modifiers,
    RowButtonView,
    disclosure,
    Disclosure,
};

pub use tree::{
    tree,
    TreeView,
    TreeAction,
    ExpansionState,
};

pub use list::{
    list,
    ListView,
    ListAction,
    list_with_children,
    NestedListView,
    section,
    Section,
    disclosure_group,
    DisclosureGroup,
};

pub use table::{
    table,
    TableView,
    TableAction,
    column,
    ColumnBuilder,
    ColumnDef,
    ColumnWidth,
    ColumnWidths,
    resizable_header,
    ResizableHeaderView,
    ResizableHeader,
    ColumnResizeAction,
    SortOrder,
    SortDirection,
    SortDescriptor,
    Alignment,
};

pub use tabs::{
    TabItem,
    SimpleTab,
    TabBar,
    TabBarColors,
};

pub use menu_button::{
    menu_button,
    MenuButtonView,
    MenuButton,
    MenuButtonPress,
    MenuDropdown,
    PulldownMenuItem,
    MenuSeparator,
};

pub use dropdown_select::{
    dropdown_select,
    DropdownSelectView,
    DropdownSelect,
    DropdownSelectAction,
    SelectDropdown,
    SelectOptionItem,
};

pub use xilem;
