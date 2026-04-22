//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! High-performance virtualized table view for efficient rendering of large datasets.
//!
//! # Architecture Overview
//!
//! This module implements a virtualized table using Xilem's View/Widget pattern:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │ TableView (Xilem View layer)                             │
//! │   - Declarative API for building tables                        │
//! │   - Manages row view lifecycle (build/rebuild/teardown)        │
//! │   - Routes messages to child row views                         │
//! │   - Handles TableAction for user interactions           │
//! └────────────────────────┬────────────────────────────────────────┘
//!                          │ Creates & manages
//!                          ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │ TableWidget (Masonry Widget layer)                              │
//! │   - Internal scroll state management (anchor-based)             │
//! │   - Computes visible range with buffer zones                   │
//! │   - Submits TableRangeAction when range changes                │
//! │   - Handles pointer events, scrollbar interaction              │
//! │   - Paints rows then header (header overlays scrolled content) │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Key Design Patterns
//!
//! ## Action-Driven Lifecycle
//!
//! The widget-view communication follows an action-driven pattern:
//!
//! 1. **Widget detects change**: During layout, TableWidget computes the new
//!    visible range and submits a `TableRangeAction` if it differs from the
//!    current active range.
//!
//! 2. **View receives action**: The `message()` method captures the action
//!    and stores it in `pending_action`, returning `MessageResult::RequestRebuild`.
//!
//! 3. **View handles in rebuild**: During `rebuild()`, the view:
//!    - Calls `will_handle_action()` to prevent duplicate action submissions
//!    - Teardowns row views no longer in the target range
//!    - Builds new row views for indices entering the range
//!    - Rebuilds existing row views to update their state
//!
//! ## Sparse Storage
//!
//! Row widgets are stored in a `HashMap<usize, WidgetPod>` rather than a Vec,
//! allowing O(1) lookup by index and memory-efficient storage of only loaded rows.
//!
//! ## Anchor-Based Scrolling
//!
//! Instead of tracking absolute scroll position, we track:
//! - `anchor_index`: The row at/above the viewport top
//! - `scroll_offset_from_anchor`: Pixel offset within that row
//!
//! This approach handles variable row heights gracefully and avoids precision
//! issues with large scroll positions.
//!
//! ## Buffer Zones
//!
//! The visible range includes buffer zones (1.5x viewport above, 2.5x below)
//! to pre-render rows before they become visible. This prevents blank areas
//! during fast scrolling.
//!
//! # Performance Characteristics
//!
//! - **Memory**: O(viewport_rows + buffer) instead of O(total_rows)
//! - **Render**: Only visible rows are laid out and painted
//! - **Scroll**: Smooth 60fps with thousands of rows
//! - **Rebuild**: Incremental - only changed rows are rebuilt

use std::collections::HashMap;
use std::marker::PhantomData;

use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewId, ViewMarker, ViewPathTracker};
use xilem::masonry::core::Widget;
use xilem::style::Style;
use xilem::view::label;
use xilem::{Pod, ViewCtx, WidgetView};

use super::widget::{TableRangeAction, TableWidget, TableWidgetAction};
use super::{ColumnDef, ColumnWidth, ColumnWidths, SortDirection, SortOrder, TableStyle};
use super::resizable_header::{ResizableHeader, ColumnResizeAction};
use crate::traits::{Identifiable, SelectionModifiers, SelectionState, TableRow};

/// Actions that can occur on virtual table rows or columns.
#[derive(Debug, Clone, PartialEq)]
pub enum TableAction<Id> {
    /// Column header clicked for sorting.
    Sort(String, SortDirection),
    /// Row selected with optional modifiers.
    Select(Id, SelectionModifiers),
    /// Row activated (double-click or Enter).
    Activate(Id),
    /// Column resized (column_key, new_width).
    ColumnResized(String, f64),
}

/// Create the view id used for child row views.
const fn view_id_for_row(idx: usize) -> ViewId {
    ViewId::new(idx as u64)
}

/// Get the row index stored in the view id.
const fn row_index_for_view_id(id: ViewId) -> usize {
    id.routing_id() as usize
}

/// View state for each child row.
struct ChildState<View, ViewState> {
    view: View,
    state: ViewState,
}

/// Divider width between columns (must match resizable_header.rs DIVIDER_WIDTH)
const DIVIDER_WIDTH: f64 = 2.0;

/// Internal view state for VirtualTable.
pub struct TableViewState<RowView, RowViewState> {
    /// Pending action from widget.
    pending_action: Option<TableRangeAction>,
    /// Pending size change (new content width).
    pending_size_change: Option<f64>,
    /// Current content width for scaling.
    current_width: f64,
    /// Per-row view states.
    children: HashMap<usize, ChildState<RowView, RowViewState>>,
}

/// The view type for [`table`].
pub struct TableView<State, R, RowView, F, H, Sel>
where
    R: Identifiable,
{
    phantom: PhantomData<fn() -> (State, RowView)>,
    /// Data slice (indices into this are used for row building).
    item_count: usize,
    /// Column definitions.
    columns: Vec<ColumnDef>,
    /// Column widths (for resizable columns).
    column_widths: Vec<f64>,
    /// Style configuration.
    style: TableStyle,
    /// Sort order state.
    sort_order: SortOrder,
    /// Sorted indices: maps visual_idx -> data_idx.
    sorted_indices: Vec<usize>,
    /// Function to build row view: (data_index, is_selected, is_striped, column_widths) -> RowView.
    row_builder: F,
    /// Action handler.
    handler: H,
    /// Selection state for determining which rows are selected.
    selection_fn: Box<dyn Fn(usize) -> bool + Send + Sync>,
    /// ID getter for rows (uses data_idx, not visual_idx).
    id_getter: Box<dyn Fn(usize) -> R::Id + Send + Sync>,
    _sel: PhantomData<Sel>,
}

impl<State, R, RowView, F, H, Sel> ViewMarker for TableView<State, R, RowView, F, H, Sel> where
    R: Identifiable
{
}

impl<State, R, RowView, F, H, Sel> View<State, (), ViewCtx>
    for TableView<State, R, RowView, F, H, Sel>
where
    State: 'static,
    R: Identifiable + 'static,
    R::Id: Clone + Send + Sync + 'static,
    RowView: WidgetView<State, ()> + 'static,
    F: Fn(&mut State, usize, bool, bool, &[f64]) -> RowView + Send + Sync + 'static,
    H: Fn(&mut State, TableAction<R::Id>) + Clone + Send + Sync + 'static,
    Sel: SelectionState<R::Id> + 'static,
{
    type Element = Pod<TableWidget>;
    type ViewState = TableViewState<RowView, RowView::ViewState>;

    fn build(&self, ctx: &mut ViewCtx, app_state: &mut State) -> (Self::Element, Self::ViewState) {
        // Build header widget
        let header = self.build_header(ctx, app_state);

        // Extract column keys for hit testing
        let column_keys: Vec<String> = self.columns.iter().map(|c| c.key.clone()).collect();

        // Create table widget with style and set initial item count
        let widget = TableWidget::new_with_item_count(
            header.new_widget.erased(),
            self.style.clone(),
            column_keys,
            self.item_count,
        );

        let pod = Pod::new(widget);
        ctx.record_action_source(pod.new_widget.id());

        // Initial width estimate (will be updated on first SizeChanged)
        let initial_width: f64 = self.column_widths.iter().sum();

        (
            pod,
            TableViewState {
                pending_action: None,
                pending_size_change: None,
                current_width: initial_width,
                children: HashMap::new(),
            },
        )
    }

    fn rebuild(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) {
        // Update item count if changed
        if self.item_count != prev.item_count {
            TableWidget::set_item_count(&mut element, self.item_count);
        }

        // Handle pending size change
        if let Some(new_width) = view_state.pending_size_change.take() {
            view_state.current_width = new_width;
            TableWidget::will_handle_size_change(&mut element);
            TableWidget::did_handle_size_change(&mut element);
        }

        // Compute scaled column widths to match header layout
        // Header uses: available_width = size.width - divider_space
        //              scale = available_width / sum(base_widths)
        let scaled_widths = self.compute_scaled_widths(view_state.current_width);

        // Rebuild header if sort order changed
        if self.sort_order != prev.sort_order {
            let new_header = self.build_header(ctx, app_state);
            TableWidget::replace_header(&mut element, new_header.new_widget.erased());
        }

        // Handle pending range action
        if let Some(pending_action) = view_state.pending_action.take() {
            TableWidget::will_handle_action(&mut element, &pending_action);

            // Teardown old rows not in target range
            for idx in pending_action.old_range.clone() {
                if !pending_action.target_range.contains(&idx) {
                    if let Some(mut child_state) = view_state.children.remove(&idx) {
                        ctx.with_id(view_id_for_row(idx), |ctx| {
                            if let Some(mut row_mut) = TableWidget::row_mut(&mut element, idx) {
                                child_state.view.teardown(&mut child_state.state, ctx, row_mut.downcast());
                            }
                            TableWidget::remove_row(&mut element, idx);
                        });
                    }
                }
            }

            // Build/rebuild rows in target range
            for visual_idx in pending_action.target_range.clone() {
                // Map visual index to data index using sorted indices
                let data_idx = self.sorted_indices.get(visual_idx).copied().unwrap_or(visual_idx);
                let is_selected = (self.selection_fn)(visual_idx);
                let is_striped = self.style.striped && visual_idx % 2 == 1;

                if let Some(child) = view_state.children.get_mut(&visual_idx) {
                    // Rebuild existing row (pass data_idx and scaled_widths to row_builder)
                    let next_view = (self.row_builder)(app_state, data_idx, is_selected, is_striped, &scaled_widths);
                    ctx.with_id(view_id_for_row(visual_idx), |ctx| {
                        if let Some(mut row_mut) = TableWidget::row_mut(&mut element, visual_idx) {
                            next_view.rebuild(
                                &child.view,
                                &mut child.state,
                                ctx,
                                row_mut.downcast(),
                                app_state,
                            );
                        }
                        child.view = next_view;
                    });
                } else {
                    // Build new row (pass data_idx and scaled_widths to row_builder)
                    let new_view = (self.row_builder)(app_state, data_idx, is_selected, is_striped, &scaled_widths);
                    ctx.with_id(view_id_for_row(visual_idx), |ctx| {
                        let (new_element, child_state) = new_view.build(ctx, app_state);
                        TableWidget::add_row(&mut element, visual_idx, new_element.new_widget.erased());
                        view_state.children.insert(
                            visual_idx,
                            ChildState {
                                view: new_view,
                                state: child_state,
                            },
                        );
                    });
                }
            }

            TableWidget::did_handle_action(&mut element);
        } else {
            // No action, just rebuild existing rows with current scaled widths
            for (&visual_idx, child) in &mut view_state.children {
                // Map visual index to data index using sorted indices
                let data_idx = self.sorted_indices.get(visual_idx).copied().unwrap_or(visual_idx);
                let is_selected = (self.selection_fn)(visual_idx);
                let is_striped = self.style.striped && visual_idx % 2 == 1;
                let next_view = (self.row_builder)(app_state, data_idx, is_selected, is_striped, &scaled_widths);
                ctx.with_id(view_id_for_row(visual_idx), |ctx| {
                    if let Some(mut row_mut) = TableWidget::row_mut(&mut element, visual_idx) {
                        next_view.rebuild(
                            &child.view,
                            &mut child.state,
                            ctx,
                            row_mut.downcast(),
                            app_state,
                        );
                    }
                    child.view = next_view;
                });
            }
        }
    }

    fn teardown(
        &self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
    ) {
        for (&idx, child) in &mut view_state.children {
            ctx.with_id(view_id_for_row(idx), |ctx| {
                if let Some(mut row_mut) = TableWidget::row_mut(&mut element, idx) {
                    child.view.teardown(&mut child.state, ctx, row_mut.downcast());
                }
            });
        }
        ctx.teardown_action_source(element);
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        message: &mut MessageCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) -> MessageResult<()> {
        // Check for child message routing
        if let Some(first) = message.take_first() {
            let child_idx = row_index_for_view_id(first);
            if let Some(target) = view_state.children.get_mut(&child_idx) {
                if let Some(mut row_mut) = TableWidget::row_mut(&mut element, child_idx) {
                    return target.view.message(
                        &mut target.state,
                        message,
                        row_mut.downcast(),
                        app_state,
                    );
                }
            }
            tracing::error!(
                "Message sent to unloaded view in `VirtualTable::message`: {message:?}"
            );
            return MessageResult::Stale;
        }

        // Handle TableWidgetAction
        if let Some(action) = message.take_message::<TableWidgetAction>() {
            match *action {
                TableWidgetAction::RangeChanged(range_action) => {
                    view_state.pending_action = Some(range_action);
                    return MessageResult::RequestRebuild;
                }
                TableWidgetAction::SizeChanged { width } => {
                    view_state.pending_size_change = Some(width);
                    return MessageResult::RequestRebuild;
                }
                TableWidgetAction::RowClick(click) => {
                    let id = (self.id_getter)(click.row_index);
                    let action = if click.click_count >= 2 {
                        TableAction::Activate(id)
                    } else {
                        let mods = SelectionModifiers {
                            shift: click.shift,
                            command: click.command,
                            alt: false, // TableRowClickAction doesn't track alt yet
                        };
                        TableAction::Select(id, mods)
                    };
                    (self.handler)(app_state, action);
                    return MessageResult::Action(());
                }
                TableWidgetAction::HeaderClick(header_click) => {
                    // Find the column and determine new sort direction
                    if let Some(col) = self.columns.iter().find(|c| c.key == header_click.column_key) {
                        if col.sortable {
                            let current_dir = self.sort_order.direction_for(&col.key);
                            let new_dir = current_dir
                                .map(|dir| dir.toggle())
                                .unwrap_or(SortDirection::Ascending);
                            (self.handler)(app_state, TableAction::Sort(col.key.clone(), new_dir));
                            return MessageResult::Action(());
                        }
                    }
                    return MessageResult::Nop;
                }
            }
        }

        // Handle ColumnResizeAction from ResizableHeader
        if let Some(resize_action) = message.take_message::<ColumnResizeAction>() {
            (self.handler)(
                app_state,
                TableAction::ColumnResized(resize_action.column_key.clone(), resize_action.new_width),
            );
            return MessageResult::Action(());
        }

        tracing::error!(?message, "Wrong message type in VirtualTable::message");
        MessageResult::Stale
    }
}

impl<State, R, RowView, F, H, Sel> TableView<State, R, RowView, F, H, Sel>
where
    State: 'static,
    R: Identifiable,
{
    /// Compute scaled column widths to match header layout.
    ///
    /// The header scales columns proportionally to fill available width,
    /// accounting for divider space between columns. This method uses
    /// the same formula so rows align with header columns.
    fn compute_scaled_widths(&self, available_width: f64) -> Vec<f64> {
        let column_count = self.column_widths.len();
        if column_count == 0 {
            return Vec::new();
        }

        // Account for divider space (same as header)
        let divider_count = column_count.saturating_sub(1);
        let divider_space = divider_count as f64 * DIVIDER_WIDTH;
        let content_width = available_width - divider_space;

        // Compute scale factor
        let current_total: f64 = self.column_widths.iter().sum();
        let scale = if current_total > 0.0 {
            content_width / current_total
        } else {
            1.0
        };

        // Scale each column
        self.column_widths
            .iter()
            .map(|&w| (w * scale).max(40.0)) // 40.0 is MIN_COLUMN_WIDTH
            .collect()
    }

    /// Build the header widget using ResizableHeader for column resize support.
    fn build_header(&self, ctx: &mut ViewCtx, app_state: &mut State) -> Pod<ResizableHeader> {
        use xilem::masonry::core::NewWidget;
        use xilem::masonry::properties::Background;

        let text_color = self.style.header_text_color;
        let header_bg = self.style.header_bg;

        // Build header cell widgets
        let mut children: Vec<NewWidget<dyn Widget>> = Vec::new();
        let mut column_keys: Vec<String> = Vec::new();

        for col in self.columns.iter() {
            // Add sort indicator to title
            let sort_indicator = self.sort_order.direction_for(&col.key).map(|dir| {
                match dir {
                    SortDirection::Ascending => " ▲",
                    SortDirection::Descending => " ▼",
                }
            }).unwrap_or("");
            let title = format!("{}{}", col.title, sort_indicator);

            // Build the label widget
            let lbl = label(title)
                .text_size(13.0)
                .color(text_color)
                .padding(4.0);

            // Build the view to get a widget - use View trait bound to help inference
            let (pod, _view_state) = View::<State, (), ViewCtx>::build(&lbl, ctx, app_state);
            children.push(pod.new_widget.erased());
            column_keys.push(col.key.clone());
        }

        // Create ResizableHeader with current column widths
        let header = ResizableHeader::new(children, column_keys, self.column_widths.clone())
            .with_divider_color(self.style.divider_color);

        // Wrap in a Pod with background property
        let pod = Pod::new_with_props(header, Background::Color(header_bg));
        ctx.record_action_source(pod.new_widget.id());

        pod
    }
}

/// Creates a high-performance virtualized table view for large datasets.
///
/// Only renders visible rows plus a buffer zone, making it efficient
/// for tables with thousands of rows.
///
/// # Arguments
///
/// * `data` - The collection of rows (must implement `Identifiable`)
/// * `columns` - Column definitions
/// * `selection` - Selection state
/// * `sort_order` - Current sort state
/// * `row_builder` - Function that builds a view for each row: `(state, index, is_selected, is_striped) -> RowView`
/// * `handler` - Function that handles table actions
///
/// # Example
///
/// ```ignore
/// use xilem_extras::{table, column, TableAction};
///
/// table(
///     &model.employees,
///     &[
///         column("name", "Name").flex(2.0).build(),
///         column("department", "Department").flex(1.5).build(),
///         column("salary", "Salary").fixed(100.0).build(),
///     ],
///     &model.selection,
///     &model.sort_order,
///     |state, idx, is_selected, is_striped| {
///         let employee = &state.employees[idx];
///         // Build row view...
///     },
///     |state, action| {
///         match action {
///             TableAction::Select(id, mods) => {
///                 state.selection.select(id, mods);
///             }
///             _ => {}
///         }
///     },
/// )
/// ```
pub fn table<'a, State, R, RowView, Sel, F, H>(
    data: &'a [R],
    columns: &'a [ColumnDef],
    column_widths: &'a ColumnWidths,
    selection: &'a Sel,
    sort_order: &'a SortOrder,
    row_builder: F,
    handler: H,
) -> impl WidgetView<State, ()> + use<'a, State, R, RowView, Sel, F, H>
where
    State: 'static,
    R: TableRow + Clone + 'static,
    R::Id: Clone + Send + Sync + 'static,
    RowView: WidgetView<State, ()> + 'static,
    Sel: SelectionState<R::Id> + Clone + Send + Sync + 'static,
    F: Fn(&mut State, usize, bool, bool, &[f64]) -> RowView + Send + Sync + 'static,
    H: Fn(&mut State, TableAction<R::Id>) + Clone + Send + Sync + 'static,
{
    table_styled(data, columns, column_widths, selection, sort_order, TableStyle::default(), row_builder, handler)
}

/// Creates a high-performance virtualized table view with custom styling.
///
/// Same as [`table`] but accepts a [`TableStyle`] for customization.
pub fn table_styled<'a, State, R, RowView, Sel, F, H>(
    data: &'a [R],
    columns: &'a [ColumnDef],
    column_widths: &'a ColumnWidths,
    selection: &'a Sel,
    sort_order: &'a SortOrder,
    style: TableStyle,
    row_builder: F,
    handler: H,
) -> impl WidgetView<State, ()> + use<'a, State, R, RowView, Sel, F, H>
where
    State: 'static,
    R: TableRow + Clone + 'static,
    R::Id: Clone + Send + Sync + 'static,
    RowView: WidgetView<State, ()> + 'static,
    Sel: SelectionState<R::Id> + Clone + Send + Sync + 'static,
    F: Fn(&mut State, usize, bool, bool, &[f64]) -> RowView + Send + Sync + 'static,
    H: Fn(&mut State, TableAction<R::Id>) + Clone + Send + Sync + 'static,
{
    // Compute sorted indices: maps visual_idx -> data_idx
    let sorted_indices = sort_order.sort_indices(data);

    // Compute column widths from ColumnWidths, falling back to ColumnDef defaults
    let widths: Vec<f64> = columns
        .iter()
        .map(|col| {
            let default_width = match col.width {
                ColumnWidth::Fixed(w) => w,
                ColumnWidth::Flex(f) => f * 100.0,
                ColumnWidth::Auto => 100.0,
            };
            column_widths.get_or(&col.key, default_width)
        })
        .collect();

    // Clone data references for closures (using sorted indices)
    let data_len = data.len();
    let data_for_id: Vec<R::Id> = data.iter().map(|r| r.id()).collect();
    let data_for_sel: Vec<R::Id> = data.iter().map(|r| r.id()).collect();

    // Clone sorted indices for closures
    let sorted_for_sel = sorted_indices.clone();
    let sorted_for_id = sorted_indices.clone();

    // Create selection check closure (uses visual index -> data index mapping)
    let selection_clone = selection.clone();
    let selection_fn = Box::new(move |visual_idx: usize| {
        if visual_idx < sorted_for_sel.len() {
            let data_idx = sorted_for_sel[visual_idx];
            selection_clone.is_selected(&data_for_sel[data_idx])
        } else {
            false
        }
    });

    // Create ID getter closure (uses visual index -> data index mapping)
    let id_getter = Box::new(move |visual_idx: usize| {
        let data_idx = sorted_for_id[visual_idx];
        data_for_id[data_idx].clone()
    });

    TableView::<State, R, RowView, F, H, Sel> {
        phantom: PhantomData,
        item_count: data_len,
        column_widths: widths,
        columns: columns.to_vec(),
        style,
        sort_order: sort_order.clone(),
        sorted_indices,
        row_builder,
        handler,
        selection_fn,
        id_getter,
        _sel: PhantomData,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_id_conversion() {
        let idx = 42usize;
        let view_id = view_id_for_row(idx);
        let back = row_index_for_view_id(view_id);
        assert_eq!(idx, back);
    }

    #[test]
    fn virtual_table_action_select() {
        let action = TableAction::Select(42u64, SelectionModifiers::COMMAND);
        if let TableAction::Select(id, mods) = action {
            assert_eq!(id, 42);
            assert!(mods.command);
        } else {
            panic!("Expected Select action");
        }
    }

    #[test]
    fn virtual_table_action_activate() {
        let action = TableAction::<u64>::Activate(42);
        if let TableAction::Activate(id) = action {
            assert_eq!(id, 42);
        } else {
            panic!("Expected Activate action");
        }
    }
}
