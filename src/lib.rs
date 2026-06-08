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

pub mod app_menu;
pub mod calendar_picker;
pub mod chart;
pub mod components;
pub mod context_menu;
pub mod dropdown_select;
pub mod form;
mod list;
pub mod locale;
pub mod menu_button;
pub mod menu_items;
pub mod progress;
mod selection;
pub mod sheet;
pub mod stock_chart;
mod table;
pub mod tabs;
pub mod theme;
pub mod time_picker;
mod traits;
mod tree;

pub use traits::{
    CellValue, Identifiable, ListItem, SelectionModifiers, SelectionState, TableRow, TreeNode,
};

pub use selection::{MultiSelection, SingleSelection};

pub use components::{
    CheckboxColors, CheckboxStyle, ClickInterceptorView, ClickInterceptorWidget, ClippedView,
    ClippedWidget, Disclosure, GroupBox, GroupBoxView, LabelAlign, ParamSelectorView,
    ParamSelectorWidget, RadioToggled, RadioWidget, RowButtonPress, RowButtonView, ScaleMode,
    StyledTextInput, SvgIcon, SvgIconView, SvgIconWidget, SwitchToggled, SwitchWidget, SynthRadio,
    SynthSwitch, TextInputColors, click_interceptor, clipped, disclosure, group_box,
    inverse_contrast_color, param_selector, row_button, row_button_with_clicks,
    row_button_with_modifiers, row_button_with_press, styled_check, styled_check_colored,
    styled_checkbox, styled_checkbox_colored, styled_radio, styled_radio_colored,
    styled_secure_text_input, styled_switch, styled_switch_colored, styled_text_input,
    styled_text_input_colored, styled_text_input_with_placeholder, svg_icon, synth_radio,
    synth_switch,
};

pub use form::{
    form, form_checkbox, form_checkbox_themed, form_radio, form_radio_themed, form_row,
    form_row_themed, form_section, form_section_themed, form_themed, form_toggle,
    form_toggle_themed,
};

pub use theme::Theme;

#[cfg(feature = "rust-logos")]
pub use components::{ferris, rust_gear, rust_logo, rust_logo_complete};

pub use tree::{
    DEFAULT_CHEVRON_COLOR,
    DEFAULT_SELECTED_BG,
    DEFAULT_TEXT_COLOR,
    ExpansionState,
    // Tree primitives (used by tree_view internally; exposed for power users)
    FlattenedNode,
    HighlightFill,
    KeyAction,
    KeyHandler,
    KeyboardFocus,
    TreeAction,
    TreeStyle,
    TreeView,
    disclosure_row,
    flatten_forest,
    flatten_forest_with_parents,
    flatten_tree,
    flatten_tree_with_parents,
    keyboard_focus,
    tree,
    tree_forest,
    tree_forest_styled,
    tree_forest_view,
    tree_forest_with_context_menu,
    tree_group,
    tree_group_styled,
    tree_group_with_context_menu,
    tree_group_with_context_menu_editable,
    tree_view,
};

pub use list::{
    ListAction,
    ListNavigableView,
    ListRangeAction,
    ListRowAction,
    ListScrollState,
    ListSection,
    ListStyle,
    ListView,
    ListViewAction,
    ListViewState,
    ListViewStyle,
    // Widget-level (keyboard navigation, sections)
    ListWidget,
    ListWidgetAction,
    ListWidgetStyle,
    SectionDef,
    SectionedListView,
    SectionedListViewState,
    SectionedRowInfo,
    list,
    // Navigable list view (simple API)
    list_navigable,
    list_styled,
    // Virtualized list view (full-featured)
    list_view,
    // Sectioned list view
    list_view_sectioned,
    list_view_styled,
};

pub use table::{
    Alignment,
    ColumnBuilder,
    ColumnDef,
    ColumnResizeAction,
    ColumnWidth,
    ColumnWidths,
    LegacyTableAction,
    ResizableHeader,
    ResizableHeaderView,
    SortDescriptor,
    SortDirection,
    // Sorting
    SortOrder,
    TableAction,
    TableHeaderClickAction,
    TableRangeAction,
    TableRowClickAction,
    // Widget-level
    TableScrollState,
    TableStyle,
    TableView,
    TableViewState,
    TableWidget,
    TableWidgetAction,
    // Column definitions
    column,
    // Legacy table API (non-virtualized)
    legacy_table,
    legacy_table_styled,
    // Resizable header
    resizable_header,
    // Main table API (virtualized, high-performance)
    table,
    table_cell,
    table_styled,
};

pub use tabs::{NavButtonMode, NavTabBar, SimpleTab, TabBar, TabBarColors, TabItem};

pub use menu_button::{
    MenuButton, MenuButtonPress, MenuButtonView, MenuDropdown, PulldownMenuItem,
    PulldownSubmenuItem, menu_button,
};

pub use dropdown_select::{
    DropdownSelect, DropdownSelectAction, DropdownSelectView, SelectDropdown, SelectOptionItem,
    dropdown_select,
};

pub use context_menu::{
    ContextMenuAction, ContextMenuDropdown, ContextMenuView, ContextMenuWidget, context_menu,
};

pub use menu_items::{
    BoxedMenuEntry, Group, IntoMenuEntries, MenuEntry, MenuItem, MenuItems, Submenu, group,
    menu_item, separator, submenu,
};

pub use app_menu::{
    ALT, AppMenuBarView, CMD, CTRL, Key, MenuBarBuilder, MenuBuilder, MenuItemBuilder,
    MenuItemChain, Modifiers, PulldownMenuBarStyle, SHIFT, Shortcut, app_menu_bar, menu_bar_label,
    pulldown_menu_bar, with_app_menu,
};

pub use sheet::{
    //    SheetLayer,
    SheetAction,
    SheetView,
    SheetWidget,
    sheet,
};

pub use calendar_picker::{
    CalendarAction, CalendarPickerView, CalendarPickerWidget, calendar_picker,
};

pub use time_picker::{TimeAction, TimePickerView, TimePickerWidget, time_picker};

pub use locale::CalendarLocale;

pub use chart::{ChartAction, ChartMode, ChartView, ChartWidget, chart};

pub use progress::{
    BusyHexSize, BusyHexView, BusyHexWidget, ProgressBarView, ProgressBarWidget,
    ProgressOrientation, ProgressStyle, RoundProgressSize, RoundProgressView, RoundProgressWidget,
    busy_hex, progress_bar, round_progress,
};

pub use xilem;
