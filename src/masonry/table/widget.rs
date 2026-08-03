// Copyright 2026 the Xilem Authors
// SPDX-License-Identifier: Apache-2.0

//! Virtualized table widget (Masonry layer) for efficient rendering of large datasets.
//!
//! # Widget Responsibilities
//!
//! The `TableWidget` is the Masonry-level implementation that handles:
//!
//! - **Scroll State**: Tracks anchor position and offset for smooth scrolling
//! - **Range Computation**: Determines which rows should be loaded based on viewport
//! - **Event Handling**: Processes pointer events (scroll wheel, scrollbar, row clicks)
//! - **Layout**: Positions header and row widgets, clips content area
//! - **Paint**: Renders background, scrollbar; children paint themselves
//!
//! # Action Protocol
//!
//! When the visible range changes, the widget submits a `TableWidgetAction::RangeChanged`:
//!
//! 1. Widget computes target_range in layout()
//! 2. If target_range != active_range, submit TableRangeAction
//! 3. Set action_pending = true to prevent duplicate submissions
//! 4. View calls will_handle_action() with the action
//! 5. View adds/removes row widgets
//! 6. View calls did_handle_action() when done
//! 7. Widget sets action_pending = false
//!
//! # Scrollbar
//!
//! The widget includes a built-in scrollbar with:
//! - Track and thumb rendering
//! - Click-to-jump on track
//! - Drag-to-scroll on thumb
//! - Hover highlighting

use std::any::TypeId;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;

use tracing::{Span, trace_span};
use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, CursorIcon, EventCtx, LayoutCtx, MeasureCtx, NewWidget,
    PaintCtx, PointerButtonEvent, PointerEvent, PointerScrollEvent, PointerUpdate, PropertiesMut,
    PropertiesRef, QueryCtx, RegisterCtx, ScrollDelta, TextEvent, Update, UpdateCtx, Widget,
    WidgetId, WidgetMut, WidgetPod,
    keyboard::{Key, NamedKey},
};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Axis, Point, Rect, RoundedRect, Size};
use xilem::masonry::layout::{LenReq, Length};
use xilem::masonry::peniko::Color;
use xilem::masonry::properties::Background;

use crate::masonry::flow_direction::FlowDirection;
use crate::masonry::table::column_layout::{
    self, ColumnBox, ColumnResizeMode, DIVIDER_HIT_AREA, DIVIDER_WIDTH, MIN_COLUMN_WIDTH,
};
use crate::xilem::table::{TableScrollState, TableStyle};

/// Scrollbar configuration.
const SCROLLBAR_WIDTH: f64 = 8.0;
const SCROLLBAR_MIN_THUMB: f64 = 20.0;
const SCROLLBAR_CORNER_RADIUS: f64 = 4.0;
const LINE_HEIGHT_PX: f64 = 28.0;
const PAGE_HEIGHT_PX: f64 = 400.0;

/// Action sent when visible range changes.
#[derive(Debug, Clone, PartialEq)]
pub struct TableRangeAction {
    /// Previous active range.
    pub old_range: Range<usize>,
    /// New target range to load.
    pub target_range: Range<usize>,
}

/// Action sent when a row is clicked.
#[derive(Debug, Clone)]
pub struct TableRowClickAction {
    /// Row index that was clicked.
    pub row_index: usize,
    /// Click count (1 = single, 2 = double).
    pub click_count: u32,
    /// Whether shift was held.
    pub shift: bool,
    /// Whether command/ctrl was held.
    pub command: bool,
}

/// Action sent when a header column is clicked.
#[derive(Debug, Clone)]
pub struct TableHeaderClickAction {
    /// Column index that was clicked.
    pub column_index: usize,
    /// Column key. `Arc<str>` — cloned from the column definition, no allocation.
    pub column_key: Arc<str>,
}

/// Combined action for table events.
#[derive(Debug, Clone)]
pub enum TableWidgetAction {
    /// Range of visible rows changed.
    RangeChanged(TableRangeAction),
    /// Row was clicked.
    RowClick(TableRowClickAction),
    /// Header column was clicked (for sorting).
    HeaderClick(TableHeaderClickAction),
    /// Table size changed (for responsive column widths).
    SizeChanged { width: f64 },
}

/// Virtualized table widget.
///
/// Manages internal scrolling and only renders visible rows.
/// Header is painted last to overlay scrolled content.
pub struct TableWidget {
    /// Header widget (fixed, doesn't scroll).
    header: WidgetPod<dyn Widget>,
    /// Loaded row widgets (sparse storage by index).
    rows: HashMap<usize, WidgetPod<dyn Widget>>,
    /// Scroll/visibility state.
    state: TableScrollState,
    /// Header height in pixels.
    header_height: f64,
    /// Widget size from last layout.
    size: Size,
    /// Last layout width for detecting size changes.
    last_layout_width: Option<f64>,
    /// Style configuration.
    style: TableStyle,
    /// Column keys for header click detection.
    column_keys: Vec<Arc<str>>,
    /// Real per-column widths, pushed down from the view (same source of
    /// truth the header uses). Falls back to equal-width columns until the
    /// first rebuild reaches us.
    configured_column_widths: Vec<f64>,
    /// Layout direction, kept in sync with the header's.
    direction: FlowDirection,
    /// Divider currently hovered/dragged in the header, for the full-height
    /// highlight painted in `post_paint`. Cleared on release (drag-only
    /// visual feedback, by design).
    active_divider: Option<usize>,
    /// Index of the column most recently dragged (or currently being
    /// dragged), mirroring `ResizableHeader::last_resized_index`. Unlike
    /// `active_divider`, never cleared on release — `FixedViewport`'s
    /// protection anchor needs to stay stable after the drag ends, or
    /// `update_column_layouts` would recompute a different (unanchored)
    /// split than what the header/rows are using.
    last_resized_index: Option<usize>,
    /// Mirrors `ResizableHeader::frozen_anchor_width` — frozen for the
    /// duration of an active drag (tracked via `active_divider` going from
    /// `None` to `Some`, since this widget has no `Down` event of its own)
    /// so the RTL mirror anchor used for hit-testing/the full-height
    /// highlight doesn't self-reference the dragged column's own width in
    /// `Overflow` mode. See `ResizableHeader::frozen_anchor_width`'s doc
    /// comment for the full derivation.
    frozen_available_width: Option<f64>,
    /// Computed column layouts for hit testing.
    column_layouts: Vec<ColumnBox>,
    /// How columns behave once their configured widths exceed the
    /// viewport, kept in sync with the header's.
    resize_mode: ColumnResizeMode,
    /// Whether to always show a full-height divider line at every column
    /// boundary, not just the actively-dragged one.
    show_column_dividers: bool,
    /// Whether we're waiting for view to handle range action.
    action_pending: bool,
    /// Whether we're waiting for view to handle size change.
    size_change_pending: bool,
    /// Scrollbar colors.
    scrollbar_track_color: Color,
    scrollbar_thumb_color: Color,
    scrollbar_thumb_hover_color: Color,
    /// Scrollbar interaction state.
    scrollbar_hovered: bool,
    scrollbar_dragging: bool,
    scrollbar_drag_start_y: f64,
    scrollbar_drag_start_position: f64,
    /// Currently focused row index for keyboard navigation.
    focused_row_index: Option<usize>,
}

impl TableWidget {
    /// Creates a new table widget with a header.
    pub fn new(
        header: NewWidget<dyn Widget>,
        style: TableStyle,
        column_keys: Vec<Arc<str>>,
    ) -> Self {
        Self::new_with_item_count(header, style, column_keys, 0)
    }

    /// Creates a new table widget with a header and initial item count.
    pub fn new_with_item_count(
        header: NewWidget<dyn Widget>,
        style: TableStyle,
        column_keys: Vec<Arc<str>>,
        item_count: usize,
    ) -> Self {
        let mut state = TableScrollState::new(style.row_height);
        state.set_item_count(item_count);

        Self {
            header: header.to_pod(),
            rows: HashMap::new(),
            state,
            header_height: style.header_height,
            size: Size::ZERO,
            last_layout_width: None,
            style,
            column_keys,
            configured_column_widths: Vec::new(),
            direction: FlowDirection::Ltr,
            resize_mode: ColumnResizeMode::default(),
            show_column_dividers: false,
            active_divider: None,
            last_resized_index: None,
            frozen_available_width: None,
            column_layouts: Vec::new(),
            action_pending: false,
            size_change_pending: false,
            scrollbar_track_color: Color::from_rgba8(60, 58, 55, 128),
            scrollbar_thumb_color: Color::from_rgba8(120, 118, 115, 200),
            scrollbar_thumb_hover_color: Color::from_rgba8(150, 148, 145, 255),
            scrollbar_hovered: false,
            scrollbar_dragging: false,
            scrollbar_drag_start_y: 0.0,
            scrollbar_drag_start_position: 0.0,
            focused_row_index: None,
        }
    }

    /// Updates column layout info for hit testing and the full-height
    /// divider highlight (called after header layout).
    ///
    /// Mirrors `ResizableHeader::layout`'s scale-to-fit + direction-aware
    /// placement exactly, using the real per-column widths pushed down from
    /// the view (same source of truth the header uses), so hit-testing and
    /// the highlight line up with what's actually drawn. Falls back to
    /// equal-width columns only if `configured_column_widths` hasn't been
    /// populated yet (i.e. before the first rebuild reaches us).
    fn update_column_layouts(&mut self) {
        self.column_layouts.clear();
        let available_width = self.size.width - SCROLLBAR_WIDTH;
        let column_count = self.column_keys.len();
        if column_count == 0 {
            return;
        }

        if self.configured_column_widths.len() != column_count {
            let col_width = available_width / column_count as f64;
            let mut x = 0.0;
            for key in &self.column_keys {
                self.column_layouts.push(ColumnBox {
                    key: key.clone(),
                    x_offset: x,
                    width: col_width,
                });
                x += col_width;
            }
            return;
        }

        // Real per-column widths, resolved the same way the header resolves
        // them (`ResizableHeader::layout`), so hit-testing and the
        // highlight agree with what's actually drawn. `last_resized_index`
        // (not `active_divider`, which intentionally clears on release for
        // the drag-only highlight) is the protection anchor, mirroring
        // `ResizableHeader::last_resized_index` — kept stable after release
        // so this doesn't recompute a different split than the header/rows.
        let scaled_widths = column_layout::compute_rendered_widths(
            &self.configured_column_widths,
            self.last_resized_index,
            available_width,
            self.resize_mode,
        );

        // RTL anchors to the actual available width, mirroring
        // `ResizableHeader::layout`'s anchoring, so hit-testing and the
        // highlight agree with the header exactly — including freezing it
        // during an active drag (`frozen_available_width`) for the same
        // self-referential-anchor reason `ResizableHeader` does.
        let anchor_width = self.frozen_available_width.unwrap_or(available_width);
        self.column_layouts = column_layout::place_columns(
            &self.column_keys,
            &scaled_widths,
            anchor_width,
            self.direction,
        );
    }

    /// Report a finite intrinsic width so a horizontal-scrolling
    /// host container (typically a `portal(...)`) can wrap the
    /// table without tripping Masonry's "measured inline length
    /// must be finite" assertion.
    ///
    /// # Why this exists at all
    ///
    /// The virtualized table widget genuinely doesn't have a fixed
    /// "natural" width — it scales to whatever budget the parent
    /// hands it via `LenReq::FitContent(available)`. That's the
    /// happy path: a parent gives a finite budget, the table fills
    /// it, internal columns share that space.
    ///
    /// But the moment a host wraps the table in a horizontal-scroll
    /// container, the parent's `LenReq::MaxContent` query starts
    /// flowing in. A horizontal-scroll container *has* infinite
    /// horizontal room conceptually, so it asks the child "how
    /// wide do you want to be at most?" — the child's answer
    /// becomes the scroll content size. Returning `f64::INFINITY`
    /// (the old behaviour) is technically the most accurate answer
    /// for an infinitely-scalable widget, but Masonry's layout
    /// pipeline treats `inf` as a programming error and emits a
    /// stream of `chosen border-box size width must be
    /// non-negative` traces while the table content disappears
    /// from the screen.
    ///
    /// # The fix
    ///
    /// When `configured_column_widths` has arrived (pushed down from
    /// `TableView::rebuild()`, same source of truth `update_column_layouts`
    /// uses), compute the exact content width directly: the sum of each
    /// column's width (clamped to `MIN_COLUMN_WIDTH`, matching
    /// `update_column_layouts`), plus divider gaps, plus the scrollbar
    /// strip (`self.size.width` always includes it). Before the first
    /// rebuild reaches the widget, fall back to the last laid-out
    /// `size.width`, or `DEFAULT_FALLBACK_WIDTH` if no layout has happened
    /// yet — a one-frame approximation until real widths arrive.
    fn intrinsic_max_width(&self) -> f64 {
        /// First-frame fallback when no layout has happened yet.
        /// 800 px is wider than `MinContent` (200) and matches the
        /// vertical-axis fallback (400 — vertical's own
        /// `MaxContent` value), so a host that wraps the table in
        /// a brand-new portal sees a reasonable initial content
        /// size before the first real layout settles.
        const DEFAULT_FALLBACK_WIDTH: f64 = 800.0;

        if self.configured_column_widths.len() == self.column_keys.len()
            && !self.configured_column_widths.is_empty()
        {
            let divider_space =
                self.configured_column_widths.len().saturating_sub(1) as f64 * DIVIDER_WIDTH;
            let columns_width: f64 = self
                .configured_column_widths
                .iter()
                .map(|&w| w.max(MIN_COLUMN_WIDTH))
                .sum();
            columns_width + divider_space + SCROLLBAR_WIDTH
        } else if self.size.width > 0.0 {
            self.size.width
        } else {
            DEFAULT_FALLBACK_WIDTH
        }
    }

    /// Hit test for header column.
    fn hit_test_header_column(&self, x: f64) -> Option<(usize, Arc<str>)> {
        for (i, col) in self.column_layouts.iter().enumerate() {
            if x >= col.x_offset && x < col.x_offset + col.width {
                return Some((i, col.key.clone()));
            }
        }
        None
    }

    /// Navigates to a row via keyboard, updating focus and submitting action.
    fn navigate_to_row(
        &mut self,
        ctx: &mut EventCtx<'_>,
        row_index: usize,
        modifiers: xilem::masonry::core::Modifiers,
    ) {
        if self.state.item_count == 0 {
            return;
        }
        self.focused_row_index = Some(row_index);
        self.state.scroll_to_row(row_index);
        ctx.submit_action::<TableWidgetAction>(TableWidgetAction::RowClick(TableRowClickAction {
            row_index,
            click_count: 1,
            shift: modifiers.shift(),
            command: modifiers.meta() || modifiers.ctrl(),
        }));
        ctx.request_layout();
        ctx.set_handled();
    }

    /// Sets the item count.
    pub fn set_item_count(this: &mut WidgetMut<'_, Self>, count: usize) {
        this.widget.state.set_item_count(count);
        this.ctx.request_layout();
    }

    /// Sets header height.
    pub fn set_header_height(this: &mut WidgetMut<'_, Self>, height: f64) {
        this.widget.header_height = height;
        this.ctx.request_layout();
    }

    /// Sets row height.
    pub fn set_row_height(this: &mut WidgetMut<'_, Self>, height: f64) {
        this.widget.state.row_height = height;
        this.ctx.request_layout();
    }

    /// Sets the layout direction, used for the same-direction column layout
    /// this widget computes independently (for hit-testing and the
    /// full-height divider highlight).
    pub fn with_direction(mut self, direction: FlowDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Sets the layout direction.
    pub fn set_direction(this: &mut WidgetMut<'_, Self>, direction: FlowDirection) {
        if this.widget.direction != direction {
            this.widget.direction = direction;
            this.ctx.request_layout();
        }
    }

    /// Sets how columns behave once their configured widths exceed the
    /// viewport, used for the same-mode column layout this widget computes
    /// independently (for hit-testing and the full-height divider
    /// highlight).
    pub fn with_resize_mode(mut self, mode: ColumnResizeMode) -> Self {
        self.resize_mode = mode;
        self
    }

    /// Sets the resize mode.
    pub fn set_resize_mode(this: &mut WidgetMut<'_, Self>, mode: ColumnResizeMode) {
        if this.widget.resize_mode != mode {
            this.widget.resize_mode = mode;
            this.ctx.request_layout();
        }
    }

    /// Always shows a full-height divider line at every column boundary
    /// (the same guideline normally only shown while actively dragging a
    /// divider), instead of just the actively-dragged one.
    pub fn with_show_column_dividers(mut self, enabled: bool) -> Self {
        self.show_column_dividers = enabled;
        self
    }

    /// Sets whether every column boundary always shows its divider line.
    pub fn set_show_column_dividers(this: &mut WidgetMut<'_, Self>, enabled: bool) {
        if this.widget.show_column_dividers != enabled {
            this.widget.show_column_dividers = enabled;
            this.ctx.request_render();
        }
    }

    /// Paints one full-height divider guideline at `col`'s trailing edge.
    fn paint_divider_line(
        col: &ColumnBox,
        direction: FlowDirection,
        height: f64,
        color: Color,
        painter: &mut Painter<'_>,
    ) {
        let x = column_layout::divider_start(col, direction);
        let rect = Rect::new(x, 0.0, x + DIVIDER_WIDTH, height);
        painter.fill(rect, color).draw();
    }

    /// Sets the real, unscaled per-column widths (same source of truth the
    /// header uses), replacing the naive equal-width fallback.
    pub fn set_column_widths(this: &mut WidgetMut<'_, Self>, widths: Vec<f64>) {
        this.widget.configured_column_widths = widths;
        this.ctx.request_layout();
    }

    /// Sets which divider (if any) is currently hovered/dragged in the
    /// header, so the full-height highlight in `post_paint` tracks it.
    pub fn set_active_divider(this: &mut WidgetMut<'_, Self>, divider: Option<usize>) {
        if this.widget.active_divider != divider {
            // Freeze/release `frozen_available_width` on the same
            // None<->Some transition that marks a drag starting/ending —
            // this widget has no `Down`/`Up` event of its own, so
            // `active_divider` (mirrored down from the header) is the only
            // signal it has for "a drag just started" / "just ended". See
            // `frozen_available_width`'s doc comment for why this matters.
            if this.widget.active_divider.is_none() && divider.is_some() {
                this.widget.frozen_available_width =
                    Some(this.widget.size.width - SCROLLBAR_WIDTH);
            } else if divider.is_none() {
                this.widget.frozen_available_width = None;
            }
            this.widget.active_divider = divider;
            this.ctx.request_render();
        }
        // `last_resized_index` mirrors `active_divider` while a divider is
        // active, but — unlike it — is never reset to `None`, so it stays
        // the `FixedViewport` protection anchor after release too.
        if let Some(idx) = divider {
            this.widget.last_resized_index = Some(idx);
        }
    }

    /// Replaces the header widget.
    pub fn replace_header(this: &mut WidgetMut<'_, Self>, new_header: NewWidget<dyn Widget>) {
        // Remove old header
        let old_header = std::mem::replace(&mut this.widget.header, new_header.to_pod());
        this.ctx.remove_child(old_header);
        this.ctx.children_changed();
        this.ctx.request_layout();
    }

    /// Indicates that `action` is about to be handled by the view.
    ///
    /// This must be called before `add_row` or `remove_row`.
    pub fn will_handle_action(this: &mut WidgetMut<'_, Self>, action: &TableRangeAction) {
        if this.widget.state.active_range != action.old_range {
            tracing::warn!(
                "Handling a TableRangeAction with the wrong range; got {:?}, expected {:?}",
                action.old_range,
                this.widget.state.active_range,
            );
        }
        this.widget.action_pending = true;
        this.widget.state.active_range = action.target_range.clone();
        this.ctx.request_layout();
    }

    /// Called after action handling is complete.
    pub fn did_handle_action(this: &mut WidgetMut<'_, Self>) {
        this.widget.action_pending = false;
    }

    /// Called when size change is about to be handled.
    pub fn will_handle_size_change(this: &mut WidgetMut<'_, Self>) {
        this.widget.size_change_pending = true;
    }

    /// Called after size change handling is complete.
    pub fn did_handle_size_change(this: &mut WidgetMut<'_, Self>) {
        this.widget.size_change_pending = false;
    }

    /// Returns the current content width (excluding scrollbar).
    pub fn content_width(&self) -> f64 {
        self.size.width - SCROLLBAR_WIDTH
    }

    /// Add a row widget at an index.
    ///
    /// This should be done only in the handling of a [`TableRangeAction`].
    /// This must be called after [`TableWidget::will_handle_action`].
    #[track_caller]
    pub fn add_row(this: &mut WidgetMut<'_, Self>, index: usize, row: NewWidget<dyn Widget>) {
        debug_assert!(
            this.widget.action_pending,
            "You must call `will_handle_action` before `add_row`."
        );
        debug_assert!(
            this.widget.state.active_range.contains(&index),
            "`add_row` should only be called with an index requested by the controller."
        );
        this.ctx.children_changed();
        if this.widget.rows.insert(index, row.to_pod()).is_some() {
            tracing::warn!("Tried to add row {index} twice to TableWidget");
        }
    }

    /// Remove a row widget.
    ///
    /// This should be done only in the handling of a [`TableRangeAction`].
    /// This must be called after [`TableWidget::will_handle_action`].
    #[track_caller]
    pub fn remove_row(this: &mut WidgetMut<'_, Self>, index: usize) {
        debug_assert!(
            this.widget.action_pending,
            "You must call `will_handle_action` before `remove_row`."
        );
        debug_assert!(
            !this.widget.state.active_range.contains(&index),
            "`remove_row` should only be called with an index which is not active."
        );
        if let Some(child) = this.widget.rows.remove(&index) {
            this.ctx.remove_child(child);
        } else {
            tracing::error!(
                "Tried to remove row ({index}) which has already been removed or was never added."
            );
        }
    }

    /// Get mutable access to header.
    pub fn header_mut<'t>(this: &'t mut WidgetMut<'_, Self>) -> WidgetMut<'t, dyn Widget> {
        this.ctx.get_mut(&mut this.widget.header)
    }

    /// Get mutable access to a row.
    pub fn row_mut<'t>(
        this: &'t mut WidgetMut<'_, Self>,
        index: usize,
    ) -> Option<WidgetMut<'t, dyn Widget>> {
        this.widget
            .rows
            .get_mut(&index)
            .map(|pod| this.ctx.get_mut(pod))
    }

    /// Returns current scroll state.
    pub fn scroll_state(&self) -> &TableScrollState {
        &self.state
    }

    /// Returns row indices currently in the widget.
    pub fn row_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.rows.keys().copied()
    }

    /// Hit test for scrollbar.
    fn scrollbar_hit_test(&self, pos: Point) -> bool {
        if self.state.item_count == 0 {
            return false;
        }

        let scrollbar_rect = self.scrollbar_rect();
        scrollbar_rect.contains(pos)
    }

    /// Get scrollbar track rectangle.
    fn scrollbar_rect(&self) -> Rect {
        let content_height = self.size.height - self.header_height;
        Rect::new(
            self.size.width - SCROLLBAR_WIDTH,
            self.header_height,
            self.size.width,
            self.header_height + content_height,
        )
    }

    /// Get scrollbar thumb rectangle.
    fn scrollbar_thumb_rect(&self) -> Rect {
        let track = self.scrollbar_rect();
        let track_height = track.height();

        let thumb_size = (self.state.scrollbar_thumb_size() * track_height)
            .max(SCROLLBAR_MIN_THUMB)
            .min(track_height);

        let available_track = track_height - thumb_size;
        let thumb_top = track.y0 + self.state.scrollbar_thumb_position() * available_track;

        Rect::new(track.x0, thumb_top, track.x1, thumb_top + thumb_size)
    }

    /// Convert ScrollDelta to pixel delta.
    fn scroll_delta_to_pixels(delta: &ScrollDelta) -> f64 {
        match delta {
            ScrollDelta::PixelDelta(pos) => pos.y,
            ScrollDelta::LineDelta(_x, y) => (*y as f64) * LINE_HEIGHT_PX,
            ScrollDelta::PageDelta(_x, y) => (*y as f64) * PAGE_HEIGHT_PX,
        }
    }

    /// Paint the scrollbar.
    fn paint_scrollbar(&self, painter: &mut Painter<'_>) {
        // Don't draw if content fits
        if self.state.content_height() <= self.state.viewport_height {
            return;
        }

        // Track
        let track = self.scrollbar_rect();
        let track_rounded = RoundedRect::from_rect(track, SCROLLBAR_CORNER_RADIUS);
        painter
            .fill(&track_rounded, self.scrollbar_track_color)
            .draw();

        // Thumb
        let thumb = self.scrollbar_thumb_rect();
        let thumb_rounded = RoundedRect::from_rect(thumb, SCROLLBAR_CORNER_RADIUS);
        let thumb_color = if self.scrollbar_hovered || self.scrollbar_dragging {
            self.scrollbar_thumb_hover_color
        } else {
            self.scrollbar_thumb_color
        };
        painter.fill(&thumb_rounded, thumb_color).draw();
    }
}

impl Widget for TableWidget {
    type Action = TableWidgetAction;

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        match event {
            PointerEvent::Scroll(PointerScrollEvent { delta, .. }) => {
                // Handle mouse wheel/trackpad vertical scrolling.
                // Negate delta to match OS scroll direction (natural scrolling)
                let scroll_delta = -Self::scroll_delta_to_pixels(delta);
                // Only claim the event if it actually has a vertical
                // component. A purely-horizontal delta (e.g. a two-finger
                // trackpad swipe) has nothing for this widget's own
                // (vertical-only) scroll state to act on — leave it
                // unhandled so it bubbles up to an ancestor `portal(...)`,
                // which is the only thing that can pan horizontally.
                if scroll_delta != 0.0 {
                    self.state.scroll_by(scroll_delta);
                    ctx.request_layout();
                    ctx.request_compose();
                    ctx.set_handled();
                }
            }
            PointerEvent::Down(PointerButtonEvent { state, .. }) => {
                let pos = ctx.local_position(state.position);

                // Check scrollbar first
                if self.scrollbar_hit_test(pos) {
                    let thumb = self.scrollbar_thumb_rect();
                    if thumb.contains(pos) {
                        // Start thumb drag
                        ctx.capture_pointer();
                        self.scrollbar_dragging = true;
                        self.scrollbar_drag_start_y = pos.y;
                        self.scrollbar_drag_start_position = self.state.scroll_position();
                        ctx.set_handled();
                        return;
                    } else {
                        // Click on track - jump to position
                        let track = self.scrollbar_rect();
                        let click_ratio = (pos.y - track.y0) / track.height();
                        let target_scroll = click_ratio * self.state.max_scroll_offset();
                        self.state.scroll_to(target_scroll);
                        ctx.request_layout();
                        ctx.set_handled();
                        return;
                    }
                }

                // Check if click is in header area
                if pos.y < self.header_height {
                    if let Some((col_idx, col_key)) = self.hit_test_header_column(pos.x) {
                        ctx.submit_action::<Self::Action>(TableWidgetAction::HeaderClick(
                            TableHeaderClickAction {
                                column_index: col_idx,
                                column_key: col_key.clone(),
                            },
                        ));
                        ctx.set_handled();
                        return;
                    }
                }

                // Check if click is in row area (below header)
                let y = pos.y - self.header_height;
                if y >= 0.0 {
                    if let Some(row_index) = self.state.row_at_y(y) {
                        // Request focus for keyboard navigation
                        ctx.request_focus();
                        // Update focused row for keyboard navigation
                        self.focused_row_index = Some(row_index);
                        // Determine click count (double-click detection)
                        let click_count = state.count as u32;
                        // Submit row click action
                        ctx.submit_action::<Self::Action>(TableWidgetAction::RowClick(
                            TableRowClickAction {
                                row_index,
                                click_count,
                                shift: state.modifiers.shift(),
                                command: state.modifiers.meta() || state.modifiers.ctrl(),
                            },
                        ));
                        ctx.set_handled();
                    }
                }
            }
            PointerEvent::Move(PointerUpdate { current, .. }) => {
                let pos = ctx.local_position(current.position);

                if self.scrollbar_dragging {
                    // Handle scrollbar drag
                    let track = self.scrollbar_rect();
                    let thumb_size = (self.state.scrollbar_thumb_size() * track.height())
                        .max(SCROLLBAR_MIN_THUMB);
                    let available_track = track.height() - thumb_size;

                    let delta_y = pos.y - self.scrollbar_drag_start_y;
                    let delta_scroll = if available_track > 0.0 {
                        delta_y / available_track * self.state.max_scroll_offset()
                    } else {
                        0.0
                    };

                    self.state
                        .scroll_to(self.scrollbar_drag_start_position + delta_scroll);
                    ctx.request_layout();
                    ctx.set_handled();
                } else {
                    // Update scrollbar hover state
                    let was_hovered = self.scrollbar_hovered;
                    self.scrollbar_hovered = self.scrollbar_hit_test(pos);
                    if was_hovered != self.scrollbar_hovered {
                        ctx.request_render();
                    }
                }
            }
            PointerEvent::Up(..) | PointerEvent::Cancel(..) => {
                if self.scrollbar_dragging {
                    self.scrollbar_dragging = false;
                    ctx.request_render();
                }
            }
            PointerEvent::Leave(..) => {
                if self.scrollbar_hovered {
                    self.scrollbar_hovered = false;
                    ctx.request_render();
                }
            }
            _ => {}
        }
    }

    fn on_text_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &TextEvent,
    ) {
        match event {
            TextEvent::Keyboard(key_event) if !key_event.state.is_up() => match &key_event.key {
                Key::Named(NamedKey::ArrowUp) => {
                    let new_idx = self
                        .focused_row_index
                        .map(|i| i.saturating_sub(1))
                        .unwrap_or(0);
                    self.navigate_to_row(ctx, new_idx, key_event.modifiers);
                }
                Key::Named(NamedKey::ArrowDown) => {
                    let max_idx = self.state.item_count.saturating_sub(1);
                    let new_idx = self
                        .focused_row_index
                        .map(|i| (i + 1).min(max_idx))
                        .unwrap_or(0);
                    self.navigate_to_row(ctx, new_idx, key_event.modifiers);
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn on_access_event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &AccessEvent,
    ) {
    }

    fn update(&mut self, ctx: &mut UpdateCtx<'_>, _props: &mut PropertiesMut<'_>, event: &Update) {
        match event {
            Update::HoveredChanged(_) | Update::ActiveChanged(_) => {
                ctx.request_render();
            }
            _ => {}
        }
    }

    fn register_children(&mut self, ctx: &mut RegisterCtx<'_>) {
        // Register rows first, then header last
        // This ensures header paints on top of rows
        for row in self.rows.values_mut() {
            ctx.register_child(row);
        }
        ctx.register_child(&mut self.header);
    }

    fn property_changed(&mut self, ctx: &mut UpdateCtx<'_>, property_type: TypeId) {
        if property_type == TypeId::of::<Background>() {
            ctx.request_render();
        }
    }

    fn measure(
        &mut self,
        _ctx: &mut MeasureCtx<'_>,
        _props: &PropertiesRef<'_>,
        axis: Axis,
        len_req: LenReq,
        _cross_length: Option<Length>,
    ) -> Length {
        match axis {
            Axis::Horizontal => {
                // Table fills available width (columns scale to fit)
                match len_req {
                    LenReq::FitContent(available) => available,
                    LenReq::MinContent => Length::px(200.0), // Minimum reasonable table width
                    LenReq::MaxContent => Length::px(self.intrinsic_max_width()),
                }
            }
            Axis::Vertical => {
                // Table takes available height (internal scrolling)
                match len_req {
                    LenReq::FitContent(available) => available,
                    LenReq::MinContent | LenReq::MaxContent => Length::px(400.0), // Default fallback
                }
            }
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, size: Size) {
        self.size = size;

        // Update column layouts for hit testing
        self.update_column_layouts();

        // Update viewport height (excluding header)
        self.state
            .set_viewport_height(size.height - self.header_height);

        // Check if width changed (for responsive column scaling)
        let content_width = size.width - SCROLLBAR_WIDTH;
        let width_changed = self
            .last_layout_width
            .map_or(true, |w| (w - content_width).abs() > 0.5);
        if width_changed && !self.size_change_pending {
            self.last_layout_width = Some(content_width);
            ctx.submit_action::<Self::Action>(TableWidgetAction::SizeChanged {
                width: content_width,
            });
            self.size_change_pending = true;
        }

        // Check if range needs update
        let target_range = self.state.compute_target_range();
        if target_range != self.state.active_range && !self.action_pending {
            ctx.submit_action::<Self::Action>(TableWidgetAction::RangeChanged(TableRangeAction {
                old_range: self.state.active_range.clone(),
                target_range: target_range.clone(),
            }));
            self.action_pending = true;
        }

        // Set clip path for row area BEFORE placing rows
        // This clips rows to the content area below the header
        let clip_rect = Rect::new(
            0.0,
            self.header_height,
            size.width - SCROLLBAR_WIDTH,
            size.height,
        );
        ctx.set_clip_path(clip_rect);

        // Layout active rows (clipped to content area)
        let row_width = size.width - SCROLLBAR_WIDTH;
        for (&idx, row) in &mut self.rows {
            if !self.state.active_range.contains(&idx) {
                // Stash rows outside active range
                ctx.set_stashed(row, true);
                continue;
            }

            ctx.set_stashed(row, false);
            let row_size = Size::new(row_width, self.state.row_height);
            ctx.run_layout(row, row_size);

            // Position row based on scroll state
            let y = self.header_height + self.state.row_y(idx);
            ctx.place_child(row, Point::new(0.0, y));
        }

        // Clear clip path before placing header so it's not clipped
        ctx.set_clip_path(Rect::from_origin_size(Point::ZERO, size));

        // Layout header LAST (fixed at top, not clipped)
        // This ensures header paints on top of any clipped row content
        let header_size = Size::new(size.width - SCROLLBAR_WIDTH, self.header_height);
        ctx.run_layout(&mut self.header, header_size);
        ctx.place_child(&mut self.header, Point::ORIGIN);
    }

    fn paint(
        &mut self,
        ctx: &mut PaintCtx<'_>,
        props: &PropertiesRef<'_>,
        painter: &mut Painter<'_>,
    ) {
        // 1. Background
        let rect = Rect::from_origin_size(Point::ZERO, self.size);
        {
            let cache = ctx.property_cache();
            let bg = props.get::<Background>(cache);
            let brush = bg.get_peniko_brush_for_rect(rect);
            painter.fill(rect, &brush).draw();
        }

        // 2. Set clip for row area (below header)
        // Note: Rows paint themselves when registered - we just paint our content
        let content_rect = Rect::new(
            0.0,
            self.header_height,
            self.size.width - SCROLLBAR_WIDTH,
            self.size.height,
        );

        // Clip rect is used by child painting automatically via layout placement

        // 3. Paint header background (to cover any row content at top)
        let header_rect = Rect::new(
            0.0,
            0.0,
            self.size.width - SCROLLBAR_WIDTH,
            self.header_height,
        );
        painter.fill(header_rect, self.style.header_bg).draw();

        // Note: Children (header and rows) paint themselves based on their layout positions
        // The header is placed at the top (y=0) so it naturally overlays row content

        // 4. Paint scrollbar
        self.paint_scrollbar(painter);

        // Suppress unused variable warning
        let _ = content_rect;
    }

    fn post_paint(
        &mut self,
        _ctx: &mut PaintCtx<'_>,
        _props: &PropertiesRef<'_>,
        painter: &mut Painter<'_>,
    ) {
        // Full-height divider guideline(s): painted after children (rows
        // and header) so they aren't drawn underneath their (often opaque)
        // backgrounds. Span the whole visible table height, unlike the
        // header's own local highlight which only covers its own bounds.
        if self.show_column_dividers {
            // Permanent mode: every boundary, always — the active one (if
            // any) is already covered by this, no separate draw needed.
            let n = self.column_layouts.len();
            for (i, col) in self.column_layouts.iter().enumerate() {
                if i < n - 1 {
                    Self::paint_divider_line(
                        col,
                        self.direction,
                        self.size.height,
                        self.style.divider_color,
                        painter,
                    );
                }
            }
        } else if let Some(idx) = self.active_divider {
            if let Some(col) = self.column_layouts.get(idx) {
                Self::paint_divider_line(
                    col,
                    self.direction,
                    self.size.height,
                    self.style.divider_color,
                    painter,
                );
            }
        }
    }

    fn get_cursor(&self, ctx: &QueryCtx<'_>, pos: Point) -> CursorIcon {
        let local_pos = ctx.to_local(pos);
        if self.scrollbar_hit_test(local_pos) {
            return CursorIcon::Default;
        }
        // Defensive correctness: keep the cursor consistent with the
        // full-height highlight during an active drag. In practice this
        // rarely matters — while dragging, the pointer is captured by
        // ResizableHeader, whose own get_cursor already returns EwResize
        // unconditionally for the whole drag regardless of pointer
        // position — but this makes TableWidget's own answer correct too
        // rather than the previous always-Default stub.
        if let Some(idx) = self.active_divider {
            if let Some(col) = self.column_layouts.get(idx) {
                let divider_start = column_layout::divider_start(col, self.direction);
                let divider_center = divider_start + DIVIDER_WIDTH / 2.0;
                if (local_pos.x - divider_center).abs() <= DIVIDER_HIT_AREA {
                    return CursorIcon::EwResize;
                }
            }
        }
        CursorIcon::Default
    }

    fn accessibility_role(&self) -> Role {
        Role::Table
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        _node: &mut Node,
    ) {
    }

    fn children_ids(&self) -> ChildrenIds {
        // Rows first, header last (matching register_children order)
        let mut ids: Vec<_> = self.rows.values().map(|r| r.id()).collect();
        ids.push(self.header.id());
        ChildrenIds::from_slice(&ids)
    }

    fn propagates_pointer_interaction(&self) -> bool {
        true
    }

    fn accepts_focus(&self) -> bool {
        true
    }

    fn accepts_text_input(&self) -> bool {
        false
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("TableWidget", id = id.trace())
    }
}
