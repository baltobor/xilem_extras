// Copyright 2026 the Xilem Authors
// SPDX-License-Identifier: Apache-2.0

//! High-performance virtualized table view for efficient rendering of large datasets.
//!
//! # Architecture Overview
//!
//! This module implements a virtualized table using Xilem's View/Widget pattern:
//!
//! TableView (Xilem View layer)
//!    - Declarative API for building tables
//!    - Manages row view lifecycle (build/rebuild/teardown)
//!    - Routes messages to child row views
//!    - Handles TableAction for user interactions
//!                   Creates & manages
//!
//!  TableWidget (Masonry Widget layer)
//!    - Internal scroll state management (anchor-based)
//!    - Computes visible range with buffer zones
//!    - Submits TableRangeAction when range changes
//!    - Handles pointer events, scrollbar interaction
//!    - Paints rows then header (header overlays scrolled content)
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
use std::sync::Arc;

use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewId, ViewMarker, ViewPathTracker};
use xilem::masonry::core::Widget;
use xilem::masonry::kurbo::Point;
use xilem::masonry::layout::Length;
use xilem::masonry::peniko::Color;
use xilem::masonry::widgets::Portal;
use xilem::style::Style;
use xilem::view::label;
use xilem::{Pod, ViewCtx, WidgetView};

use super::direction_detect::detect_direction;
use super::{ColumnDef, ColumnWidth, ColumnWidths, SortDirection, SortOrder};
use crate::masonry::flow_direction::FlowDirection;
use crate::masonry::table::column_layout::{self, ColumnBox, ColumnResizeMode};
use crate::masonry::table::resizable_header::{
    ColumnLayoutAction, ColumnResizeAction, ResizableHeader,
};
use crate::masonry::table::widget::{TableRangeAction, TableWidget, TableWidgetAction};
use crate::xilem::components::clipped;
use crate::xilem::traits::{Keyed, SelectionModifiers, SelectionState, TableRow};

/// Style configuration for the table.
#[derive(Debug, Clone)]
pub struct TableStyle {
    /// Background color on hover.
    pub hover_bg: Color,
    /// Background color for selected rows.
    pub selected_bg: Color,
    /// Background color for alternating rows (if striped).
    pub stripe_bg: Color,
    /// Header background color.
    pub header_bg: Color,
    /// Header text color.
    pub header_text_color: Color,
    /// Cell text color.
    pub text_color: Color,
    /// Column divider color.
    pub divider_color: Color,
    /// Row height in pixels.
    pub row_height: f64,
    /// Header height in pixels.
    pub header_height: f64,
    /// Whether to show alternating row backgrounds.
    pub striped: bool,
    /// Gap between columns.
    pub column_gap: f64,
    /// Layout direction override. `None` (the default) auto-detects from
    /// column header titles at build time (see `direction_detect`); `Some(_)`
    /// forces a specific direction instead.
    pub direction: Option<FlowDirection>,
    /// How columns behave once their configured widths exceed the viewport.
    /// Defaults to [`ColumnResizeMode::Overflow`] (matches Apple Numbers'
    /// actual resize behavior — the table scrolls horizontally via its own
    /// internal `Portal`, no external wrapping needed).
    /// [`ColumnResizeMode::FixedViewport`] instead compresses columns
    /// after the dragged one down to their minimum and never lets the
    /// table exceed its container.
    pub resize_mode: ColumnResizeMode,
    /// Whether to always show a full-height divider line at every column
    /// boundary (the same guideline normally only shown while actively
    /// dragging a divider). Defaults to `false`.
    pub column_dividers: bool,
}

impl Default for TableStyle {
    fn default() -> Self {
        Self {
            hover_bg: Color::from_rgba8(55, 53, 50, 255),
            selected_bg: Color::from_rgba8(65, 62, 58, 255),
            stripe_bg: Color::from_rgba8(45, 43, 40, 255),
            header_bg: Color::from_rgba8(50, 48, 45, 255),
            header_text_color: Color::from_rgba8(180, 178, 175, 255),
            text_color: Color::from_rgba8(220, 218, 214, 255),
            divider_color: Color::from_rgba8(80, 78, 75, 255),
            row_height: 28.0,
            header_height: 32.0,
            striped: false,
            column_gap: 8.0,
            direction: None,
            resize_mode: ColumnResizeMode::default(),
            column_dividers: false,
        }
    }
}

impl TableStyle {
    /// Creates a new `TableStyle` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the hover background color.
    pub fn hover_bg(mut self, color: Color) -> Self {
        self.hover_bg = color;
        self
    }

    /// Sets the selected row background color.
    pub fn selected_bg(mut self, color: Color) -> Self {
        self.selected_bg = color;
        self
    }

    /// Sets the header background color.
    pub fn header_bg(mut self, color: Color) -> Self {
        self.header_bg = color;
        self
    }

    /// Sets the row height.
    pub fn row_height(mut self, height: f64) -> Self {
        self.row_height = height;
        self
    }

    /// Sets the header height.
    pub fn header_height(mut self, height: f64) -> Self {
        self.header_height = height;
        self
    }

    /// Enables alternating row backgrounds (zebra stripes).
    pub fn striped(mut self, striped: bool) -> Self {
        self.striped = striped;
        self
    }

    /// Overrides the auto-detected layout direction.
    pub fn direction(mut self, direction: FlowDirection) -> Self {
        self.direction = Some(direction);
        self
    }

    /// Sets how columns behave once their configured widths exceed the
    /// viewport.
    pub fn resize_mode(mut self, mode: ColumnResizeMode) -> Self {
        self.resize_mode = mode;
        self
    }

    /// Always shows a full-height divider line at every column boundary
    /// (the same guideline normally only shown while actively dragging a
    /// divider). Defaults to `false`.
    pub fn column_divider(mut self, enabled: bool) -> Self {
        self.column_dividers = enabled;
        self
    }
}

/// Actions that can occur on virtual table rows or columns.
#[derive(Debug, Clone, PartialEq)]
pub enum TableAction<Id> {
    /// Column header clicked for sorting. Key is `Arc<str>` — shared with the
    /// column definition, so dispatching a sort action allocates nothing.
    Sort(Arc<str>, SortDirection),
    /// Row selected with optional modifiers.
    Select(Id, SelectionModifiers),
    /// Row activated (double-click or Enter).
    Activate(Id),
    /// Column(s) resized, on drag commit. Carries every column's current
    /// width, not just the one actually dragged — `FixedViewport` mode may
    /// have compressed others to make room, and persisting only the
    /// dragged column would leave stale, oversized desired widths in app
    /// state (see `ColumnResizeAction`'s doc comment for the exact bug
    /// this prevents: dragging a different, earlier column later would
    /// otherwise cause a visible snap in whichever column was previously
    /// dragged).
    ColumnResized(Vec<(Arc<str>, f64)>),
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

/// Internal view state for VirtualTable.
pub struct TableViewState<RowView, RowViewState> {
    /// Pending action from widget.
    pending_action: Option<TableRangeAction>,
    /// Authoritative column layout (widths + x_offsets), received directly
    /// from the header's `ColumnLayoutAction` broadcast. `ResizableHeader`
    /// is the *only* place `place_columns`/`compute_rendered_widths` are
    /// ever called — this is a pure receive-and-forward, never an
    /// independent re-derivation, so row content and the header can never
    /// disagree. Seeded once in `build()` with a plain best-effort layout
    /// so the very first frame (before the header's own first `layout()`
    /// pass has had a chance to broadcast) doesn't render degenerate
    /// zero-offset cells; every value after that comes from the broadcast.
    columns: Vec<ColumnBox>,
    /// Divider currently being dragged in the header, mirrored down into
    /// `TableWidget` for the full-height highlight. Ephemeral.
    active_divider: Option<usize>,
    /// Set when a drag just ended; consumed (and cleared) by the next
    /// `rebuild()`, which — in RTL — pans the owned `Portal` back to show
    /// the protected side. See `message()`'s `ColumnLayoutAction` arm and
    /// `rebuild()`'s use of it.
    pending_scroll_reset: bool,
    /// Per-row view states.
    children: HashMap<usize, ChildState<RowView, RowViewState>>,
}

/// The view type for [`table`].
pub struct TableView<State, R, RowView, F, H, Sel>
where
    R: Keyed,
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
    id_getter: Box<dyn Fn(usize) -> R::Key + Send + Sync>,
    _sel: PhantomData<Sel>,
}

impl<State, R, RowView, F, H, Sel> ViewMarker for TableView<State, R, RowView, F, H, Sel> where
    R: Keyed
{
}

impl<State, R, RowView, F, H, Sel> View<State, (), ViewCtx>
    for TableView<State, R, RowView, F, H, Sel>
where
    State: 'static,
    R: Keyed + 'static,
    R::Key: Clone + Send + Sync + 'static,
    RowView: WidgetView<State, ()> + 'static,
    F: Fn(&mut State, usize, bool, bool, &[f64], &[f64], FlowDirection) -> RowView
        + Send
        + Sync
        + 'static,
    H: Fn(&mut State, TableAction<R::Key>) + Clone + Send + Sync + 'static,
    Sel: SelectionState<R::Key> + 'static,
{
    // The table is always internally wrapped in a `Portal` (never left to
    // the caller, unlike the earlier `xilem::view::portal(...)`-in-the-
    // gallery approach) so `rebuild()` can get imperative `WidgetMut`
    // access to it — needed to compensate the scroll position when RTL's
    // live mirror anchor shifts every column on growth (see `rebuild()`).
    // `FixedViewport` mode sets both `Portal` axes constrained, which
    // makes it behave as a plain passthrough (no scrolling, no visible
    // scrollbars) — equivalent to not being wrapped at all.
    type Element = Pod<Portal<TableWidget>>;
    type ViewState = TableViewState<RowView, RowView::ViewState>;

    // `TableWidget` (masonry) owns scrolling, row virtualization, header
    // placement and painting entirely by itself; it only ever holds header
    // and row children as `dyn Widget`. `header.new_widget.erased()` below is
    // the seam: whatever xilem view built the header (any
    // `WidgetView<State, Action>`, not just the `label(...)` cells used in
    // `build_header`) gets type-erased into a widget masonry can store
    // without knowing it came from a xilem view at all.
    fn build(&self, ctx: &mut ViewCtx, app_state: &mut State) -> (Self::Element, Self::ViewState) {
        let direction = self.effective_direction();

        // Build header widget
        let header = self.build_header(ctx, app_state, direction);

        // Extract column keys for hit testing
        let column_keys: Vec<Arc<str>> = self.columns.iter().map(|c| c.key.clone()).collect();

        // Create table widget with style and set initial item count
        let widget = TableWidget::new_with_item_count(
            header.new_widget.erased(),
            self.style.clone(),
            column_keys.clone(),
            self.item_count,
        )
        .with_direction(direction)
        .with_show_column_dividers(self.style.column_dividers);

        let table_pod = Pod::new(widget);
        ctx.record_action_source(table_pod.new_widget.id());

        let portal = Portal::new(table_pod.new_widget)
            .constrain_vertical(true)
            .constrain_horizontal(self.style.resize_mode == ColumnResizeMode::FixedViewport);
        let pod = ctx.create_pod(portal);

        // Seed a best-effort layout for the very first frame, before the
        // header's own first `layout()` pass has had a chance to broadcast
        // the authoritative `ColumnLayoutAction` (see `TableViewState::columns`'
        // doc comment) — self-corrects immediately after, the same
        // one-frame characteristic live-resize already had.
        let divider_space =
            column_keys.len().saturating_sub(1) as f64 * column_layout::DIVIDER_WIDTH;
        let initial_anchor: f64 = self.column_widths.iter().sum::<f64>() + divider_space;
        let initial_columns = column_layout::place_columns(
            &column_keys,
            &self.column_widths,
            initial_anchor,
            direction,
        );

        (
            pod,
            TableViewState {
                pending_action: None,
                columns: initial_columns,
                active_divider: None,
                pending_scroll_reset: false,
                children: HashMap::new(),
            },
        )
    }

    // Same seam as the header, applied per row: `row_builder` returns any
    // `RowView: WidgetView<State, ()>`, which gets built/rebuilt here and
    // `.erased()`'d into `NewWidget<dyn Widget>` before reaching
    // `TableWidget::add_row`/`row_mut`. `TableWidget` never sees `RowView` —
    // it just manages a `dyn Widget` per visible row, so arbitrary xilem
    // content can back a row without any masonry-side changes.
    fn rebuild(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) {
        let direction = self.effective_direction();

        // `Portal`-level concerns first, while `element` still refers to
        // it — it's narrowed down to the inner `TableWidget` right after
        // (shadowing `element`), so every existing `TableWidget::...(&mut
        // element, ...)` call below keeps working unchanged.
        if self.style.resize_mode != prev.style.resize_mode {
            Portal::set_constrain_horizontal(
                &mut element,
                self.style.resize_mode == ColumnResizeMode::FixedViewport,
            );
        }

        // The true, stable viewport width — `Portal`'s own border-box
        // size, dictated by whatever room its parent gives it (never
        // self-referential, unlike `TableWidget`'s own `size.width` in
        // `Overflow` mode, which is always exactly the content's total
        // width since `Portal` lays a `MaxContent`-measured child out at
        // its own preferred size). This is what makes RTL's mirror anchor
        // a true constant — the same property LTR's column 0 already has
        // for free at `local_x = 0` — so nothing ever needs compensating
        // after the fact: `Portal`'s own scrollbar/wheel/drag-to-scroll
        // just work, because the coordinate space they scroll over never
        // moves under them.
        let viewport_width = element.ctx.content_box().width();

        // One-shot correction, right after a drag releases: see
        // `TableViewState::pending_scroll_reset`'s doc comment. Only RTL
        // ever needs this — LTR's `Portal` default (`viewport_pos = 0`)
        // already shows column 0, since it sits at local `0` regardless
        // of overflow.
        if view_state.pending_scroll_reset {
            view_state.pending_scroll_reset = false;
            if direction == FlowDirection::Rtl {
                let divider_space = view_state.columns.len().saturating_sub(1) as f64
                    * column_layout::DIVIDER_WIDTH;
                let total_content_width: f64 =
                    view_state.columns.iter().map(|c| c.width).sum::<f64>() + divider_space;
                let max_scroll = (total_content_width - viewport_width).max(0.0);
                Portal::set_viewport_pos(&mut element, Point::new(max_scroll, 0.0));
            }
        }

        let mut element = Portal::child_mut(&mut element);
        TableWidget::set_viewport_width(&mut element, viewport_width);

        // Update item count if changed
        if self.item_count != prev.item_count {
            TableWidget::set_item_count(&mut element, self.item_count);
        }

        TableWidget::set_direction(&mut element, direction);
        TableWidget::set_show_column_dividers(&mut element, self.style.column_dividers);

        // Rebuild header if sort order changed, the resolved layout
        // direction changed (e.g. header titles flipped language), or the
        // column definitions themselves changed (title/key/sortable) —
        // otherwise a header-language toggle with unchanged sort order would
        // silently leave the old header widget in place.
        let columns_changed = self.columns.len() != prev.columns.len()
            || self
                .columns
                .iter()
                .zip(prev.columns.iter())
                .any(|(a, b)| a.key != b.key || a.title != b.title || a.sortable != b.sortable);
        if self.sort_order != prev.sort_order
            || direction != prev.effective_direction()
            || self.style.resize_mode != prev.style.resize_mode
            || columns_changed
        {
            let new_header = self.build_header(ctx, app_state, direction);
            TableWidget::replace_header(&mut element, new_header.new_widget.erased());
        }

        // `view_state.columns` is the header's own last-broadcast layout
        // (see its doc comment) — push it straight through to `TableWidget`
        // (hit-testing, the full-height highlight) and derive the row
        // builder's `widths`/`x_offsets` from it directly. No independent
        // `place_columns`/`compute_rendered_widths` call here at all.
        TableWidget::set_columns(
            &mut element,
            view_state.columns.clone(),
            view_state.active_divider,
        );
        let widths: Vec<f64> = view_state.columns.iter().map(|c| c.width).collect();
        let x_offsets: Vec<f64> = view_state.columns.iter().map(|c| c.x_offset).collect();

        // Handle pending range action
        if let Some(pending_action) = view_state.pending_action.take() {
            TableWidget::will_handle_action(&mut element, &pending_action);

            // Teardown old rows not in target range
            for idx in pending_action.old_range.clone() {
                if !pending_action.target_range.contains(&idx) {
                    if let Some(mut child_state) = view_state.children.remove(&idx) {
                        ctx.with_id(view_id_for_row(idx), |ctx| {
                            if let Some(mut row_mut) = TableWidget::row_mut(&mut element, idx) {
                                child_state.view.teardown(
                                    &mut child_state.state,
                                    ctx,
                                    row_mut.downcast(),
                                );
                            }
                            TableWidget::remove_row(&mut element, idx);
                        });
                    }
                }
            }

            // Build/rebuild rows in target range
            for visual_idx in pending_action.target_range.clone() {
                // Map visual index to data index using sorted indices
                let data_idx = self
                    .sorted_indices
                    .get(visual_idx)
                    .copied()
                    .unwrap_or(visual_idx);
                let is_selected = (self.selection_fn)(visual_idx);
                let is_striped = self.style.striped && visual_idx % 2 == 1;

                if let Some(child) = view_state.children.get_mut(&visual_idx) {
                    // Rebuild existing row (pass data_idx and scaled_widths to row_builder)
                    let next_view = (self.row_builder)(
                        app_state,
                        data_idx,
                        is_selected,
                        is_striped,
                        &widths,
                        &x_offsets,
                        direction,
                    );
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
                    let new_view = (self.row_builder)(
                        app_state,
                        data_idx,
                        is_selected,
                        is_striped,
                        &widths,
                        &x_offsets,
                        direction,
                    );
                    ctx.with_id(view_id_for_row(visual_idx), |ctx| {
                        let (new_element, child_state) = new_view.build(ctx, app_state);
                        TableWidget::add_row(
                            &mut element,
                            visual_idx,
                            new_element.new_widget.erased(),
                        );
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
                let data_idx = self
                    .sorted_indices
                    .get(visual_idx)
                    .copied()
                    .unwrap_or(visual_idx);
                let is_selected = (self.selection_fn)(visual_idx);
                let is_striped = self.style.striped && visual_idx % 2 == 1;
                let next_view = (self.row_builder)(
                    app_state,
                    data_idx,
                    is_selected,
                    is_striped,
                    &widths,
                    &x_offsets,
                    direction,
                );
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
        {
            let mut table_element = Portal::child_mut(&mut element);
            for (&idx, child) in &mut view_state.children {
                ctx.with_id(view_id_for_row(idx), |ctx| {
                    if let Some(mut row_mut) = TableWidget::row_mut(&mut table_element, idx) {
                        child
                            .view
                            .teardown(&mut child.state, ctx, row_mut.downcast());
                    }
                });
            }
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
            let mut element = Portal::child_mut(&mut element);
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
                    if let Some(col) = self
                        .columns
                        .iter()
                        .find(|c| c.key == header_click.column_key)
                    {
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

        // Committed resize (pointer-up): the only action that reaches app
        // state. Carries every column's current width, not just the
        // dragged one — see `ColumnResizeAction`'s doc comment.
        if let Some(resize_action) = message.take_message::<ColumnResizeAction>() {
            (self.handler)(
                app_state,
                TableAction::ColumnResized(resize_action.widths.clone()),
            );
            return MessageResult::Action(());
        }

        // The header's column-layout broadcast (see `ColumnLayoutAction`'s
        // doc comment) — the *only* place row widths/x_offsets come from;
        // fires on every pointer-move during a drag (cheap self-diff
        // rebuild, never a full app-logic re-invocation, which is what
        // makes row content track the drag live) and on any other layout
        // change (window resize, `FixedViewport` recompression, etc.).
        if let Some(layout) = message.take_message::<ColumnLayoutAction>() {
            view_state.columns = layout.columns.clone();
            // A drag just ended (`Some -> None`) — in RTL + `Overflow`,
            // the mirror anchor may now be floored at the total content
            // width rather than the viewport width (see
            // `ResizableHeader::layout()`'s doc comment), which can leave
            // the protected side off-screen at `Portal`'s default
            // `viewport_pos = 0`. `rebuild()` corrects this once, right
            // after release — not every frame, which is what caused the
            // earlier jiggle.
            if view_state.active_divider.is_some() && layout.active_divider.is_none() {
                view_state.pending_scroll_reset = true;
            }
            view_state.active_divider = layout.active_divider;
            return MessageResult::RequestRebuild;
        }

        tracing::error!(?message, "Wrong message type in VirtualTable::message");
        MessageResult::Stale
    }
}

impl<State, R, RowView, F, H, Sel> TableView<State, R, RowView, F, H, Sel>
where
    State: 'static,
    R: Keyed,
{
    /// Effective layout direction: the app's override if set, otherwise
    /// auto-detected from the column header titles (never row/cell content —
    /// an Arabic user's habits are RTL even if the table's data isn't).
    fn effective_direction(&self) -> FlowDirection {
        self.style
            .direction
            .unwrap_or_else(|| detect_direction(self.columns.iter().map(|c| c.title.as_str())))
    }

    /// Build the header widget using ResizableHeader for column resize support.
    ///
    /// Uses plain `label(...)` views for cells here, but any
    /// `WidgetView<State, Action>` would work the same way — see the
    /// `erased()` call below and `ResizableHeaderView::build` for why
    /// `ResizableHeader` doesn't need to know what view built its children.
    fn build_header(
        &self,
        ctx: &mut ViewCtx,
        app_state: &mut State,
        direction: FlowDirection,
    ) -> Pod<ResizableHeader> {
        use xilem::masonry::core::NewWidget;
        use xilem::masonry::properties::Background;

        let text_color = self.style.header_text_color;
        let header_bg = self.style.header_bg;

        // Build header cell widgets
        let mut children: Vec<NewWidget<dyn Widget>> = Vec::new();
        let mut column_keys: Vec<Arc<str>> = Vec::new();

        for col in self.columns.iter() {
            // Add sort indicator to title
            let sort_indicator = self
                .sort_order
                .direction_for(&col.key)
                .map(|dir| match dir {
                    SortDirection::Ascending => " ▲",
                    SortDirection::Descending => " ▼",
                })
                .unwrap_or("");
            let title = format!("{}{}", col.title, sort_indicator);

            // Build the label widget, clipped to its cell — `ResizableHeader`
            // hard-constrains each child to its column width via
            // `ctx.run_layout` in its own `layout()`, so `clipped(...)`
            // (the same wrapper `table_cell` uses for row content) truncates
            // overlong titles to that width instead of letting them paint
            // past the column boundary into neighboring cells.
            let lbl = clipped(
                label(title)
                    .text_size(13.0)
                    .color(text_color)
                    .padding(Length::px(4.0)),
            );

            // Build the view to get a widget - use View trait bound to help inference
            let (pod, _view_state) = View::<State, (), ViewCtx>::build(&lbl, ctx, app_state);
            children.push(pod.new_widget.erased());
            column_keys.push(col.key.clone());
        }

        // Create ResizableHeader with current column widths
        let header = ResizableHeader::new(children, column_keys, self.column_widths.clone())
            .with_divider_color(self.style.divider_color)
            .with_direction(direction)
            .with_resize_mode(self.style.resize_mode);

        // Wrap in a Pod with background property
        let pod = Pod::new_with_props(header, Background::Color(header_bg));
        ctx.record_action_source(pod.new_widget.id());

        pod
    }
}

/// Chainable, SwiftUI-style style modifiers.
///
/// Every table starts from [`TableStyle::default`]; these tweak individual
/// knobs on the builder returned by [`table`]. Use [`Self::table_style`] to
/// replace the whole [`TableStyle`] in one call instead.
impl<State, R, RowView, F, H, Sel> TableView<State, R, RowView, F, H, Sel>
where
    R: Keyed,
{
    /// Replaces the entire [`TableStyle`] — colors, row/header heights,
    /// dividers, layout direction and resize mode.
    pub fn table_style(mut self, style: TableStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets how columns behave once their configured widths exceed the
    /// viewport: [`ColumnResizeMode::Overflow`] (default) scrolls
    /// horizontally, [`ColumnResizeMode::FixedViewport`] compresses columns
    /// to keep the table inside its container.
    pub fn resize_mode(mut self, mode: ColumnResizeMode) -> Self {
        self.style.resize_mode = mode;
        self
    }

    /// Enables alternating row backgrounds (zebra stripes).
    pub fn striped(mut self, striped: bool) -> Self {
        self.style.striped = striped;
        self
    }

    /// Always shows a full-height divider line at every column boundary
    /// (normally only shown while actively dragging a divider).
    pub fn column_divider(mut self, enabled: bool) -> Self {
        self.style.column_dividers = enabled;
        self
    }

    /// Overrides the layout direction that would otherwise be auto-detected
    /// from the column header titles.
    pub fn direction(mut self, direction: FlowDirection) -> Self {
        self.style.direction = Some(direction);
        self
    }

    /// Sets the hover background color.
    pub fn hover_bg(mut self, color: Color) -> Self {
        self.style.hover_bg = color;
        self
    }

    /// Sets the selected-row background color.
    pub fn selected_bg(mut self, color: Color) -> Self {
        self.style.selected_bg = color;
        self
    }

    /// Sets the header background color.
    pub fn header_bg(mut self, color: Color) -> Self {
        self.style.header_bg = color;
        self
    }

    /// Sets the row height in pixels.
    pub fn row_height(mut self, height: f64) -> Self {
        self.style.row_height = height;
        self
    }

    /// Sets the header height in pixels.
    pub fn header_height(mut self, height: f64) -> Self {
        self.style.header_height = height;
        self
    }
}

/// Creates a high-performance virtualized table view for large datasets.
///
/// Only renders visible rows plus a buffer zone, making it efficient
/// for tables with thousands of rows.
///
/// The table renders with [`TableStyle::default`]. Adjust its appearance
/// and layout with the chainable, SwiftUI-style modifiers on the returned
/// [`TableView`] — [`resize_mode`](TableView::resize_mode),
/// [`striped`](TableView::striped),
/// [`column_divider`](TableView::column_divider),
/// [`hover_bg`](TableView::hover_bg), … — or swap the whole [`TableStyle`]
/// at once with [`table_style`](TableView::table_style).
///
/// # Arguments
///
/// * `data` - The collection of rows (must implement `TableRow`)
/// * `columns` - Column definitions
/// * `column_widths` - Per-column width overrides (resizable columns)
/// * `selection` - Selection state
/// * `sort_order` - Current sort state
/// * `row_builder` - Function that builds a view for each row: `(state, index, is_selected, is_striped, column_widths, column_x_offsets, direction) -> RowView`.
///   Place cells at their exact `column_x_offsets` via
///   `xilem_extras::xilem::table::row_cells` — the same mechanism the
///   header uses for its own cells — rather than a sequential layout like
///   `flex_row`, which cannot reproduce the header's positions in RTL (see
///   `table_cell`'s doc comment for an example).
/// * `handler` - Function that handles table actions
///
/// # Example
///
/// ```ignore
/// use xilem_extras::xilem::table::{table, column, TableAction};
/// use xilem_extras::masonry::table::ColumnResizeMode;
///
/// table(
///     &model.employees,
///     &[
///         column("name", "Name").flex(2.0).build(),
///         column("department", "Department").flex(1.5).build(),
///         column("salary", "Salary").fixed(100.0).build(),
///     ],
///     &model.column_widths,
///     &model.selection,
///     &model.sort_order,
///     |state, idx, is_selected, is_striped, _widths, _x_offsets, _direction| {
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
/// .resize_mode(ColumnResizeMode::FixedViewport)
/// .striped(true)
/// ```
pub fn table<State, R, RowView, Sel, F, H>(
    data: &[R],
    columns: &[ColumnDef],
    column_widths: &ColumnWidths,
    selection: &Sel,
    sort_order: &SortOrder,
    row_builder: F,
    handler: H,
) -> TableView<State, R, RowView, F, H, Sel>
where
    State: 'static,
    R: TableRow + Clone + 'static,
    R::Key: Clone + Send + Sync + 'static,
    RowView: WidgetView<State, ()> + 'static,
    Sel: SelectionState<R::Key> + Clone + Send + Sync + 'static,
    F: Fn(&mut State, usize, bool, bool, &[f64], &[f64], FlowDirection) -> RowView
        + Send
        + Sync
        + 'static,
    H: Fn(&mut State, TableAction<R::Key>) + Clone + Send + Sync + 'static,
{
    let style = TableStyle::default();
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
    let data_for_id: Vec<R::Key> = data.iter().map(|r| r.key()).collect();
    let data_for_sel: Vec<R::Key> = data.iter().map(|r| r.key()).collect();

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
