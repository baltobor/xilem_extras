//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

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
//! ```text
//! ┌────────────────────────────────────────────────────────────────┐
//! │  1. Widget computes target_range in layout()                   │
//! │  2. If target_range != active_range, submit TableRangeAction   │
//! │  3. Set action_pending = true to prevent duplicate submissions │
//! │  4. View calls will_handle_action() with the action            │
//! │  5. View adds/removes row widgets                              │
//! │  6. View calls did_handle_action() when done                   │
//! │  7. Widget sets action_pending = false                         │
//! └────────────────────────────────────────────────────────────────┘
//! ```
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

use tracing::{trace_span, Span};
use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, CursorIcon, EventCtx, LayoutCtx, MeasureCtx, NewWidget,
    PaintCtx, PointerButtonEvent, PointerEvent, PointerScrollEvent, PointerUpdate,
    PropertiesMut, PropertiesRef, QueryCtx, RegisterCtx, ScrollDelta, TextEvent, Update,
    UpdateCtx, Widget, WidgetId, WidgetMut, WidgetPod,
    keyboard::{Key, NamedKey},
};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Axis, Point, Rect, RoundedRect, Size};
use xilem::masonry::layout::LenReq;
use xilem::masonry::peniko::Color;
use xilem::masonry::properties::Background;

use super::state::TableScrollState;
use super::TableStyle;

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
    /// Column key.
    pub column_key: String,
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

/// Column layout info for hit testing.
#[derive(Debug, Clone)]
struct ColumnLayout {
    key: String,
    x_start: f64,
    width: f64,
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
    column_keys: Vec<String>,
    /// Computed column layouts for hit testing.
    column_layouts: Vec<ColumnLayout>,
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
    pub fn new(header: NewWidget<dyn Widget>, style: TableStyle, column_keys: Vec<String>) -> Self {
        Self::new_with_item_count(header, style, column_keys, 0)
    }

    /// Creates a new table widget with a header and initial item count.
    pub fn new_with_item_count(
        header: NewWidget<dyn Widget>,
        style: TableStyle,
        column_keys: Vec<String>,
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

    /// Updates column layout info for hit testing (called after header layout).
    fn update_column_layouts(&mut self) {
        self.column_layouts.clear();
        let available_width = self.size.width - SCROLLBAR_WIDTH;
        let column_count = self.column_keys.len();
        if column_count == 0 {
            return;
        }

        // Simple equal-width columns for now
        // TODO: Use actual column widths from style/definitions
        let col_width = available_width / column_count as f64;
        let mut x = 0.0;
        for key in &self.column_keys {
            self.column_layouts.push(ColumnLayout {
                key: key.clone(),
                x_start: x,
                width: col_width,
            });
            x += col_width;
        }
    }

    /// Hit test for header column.
    fn hit_test_header_column(&self, x: f64) -> Option<(usize, &str)> {
        for (i, col) in self.column_layouts.iter().enumerate() {
            if x >= col.x_start && x < col.x_start + col.width {
                return Some((i, &col.key));
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
                // Handle mouse wheel scrolling
                // Negate delta to match OS scroll direction (natural scrolling)
                let scroll_delta = -Self::scroll_delta_to_pixels(delta);
                self.state.scroll_by(scroll_delta);
                ctx.request_layout();
                ctx.request_compose();
                ctx.set_handled();
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
                                column_key: col_key.to_string(),
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
            TextEvent::Keyboard(key_event) if !key_event.state.is_up() => {
                match &key_event.key {
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
                }
            }
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
        _cross_length: Option<f64>,
    ) -> f64 {
        match axis {
            Axis::Horizontal => {
                // Table fills available width (columns scale to fit)
                match len_req {
                    LenReq::FitContent(available) => available,
                    LenReq::MinContent => 200.0, // Minimum reasonable table width
                    LenReq::MaxContent => f64::INFINITY,
                }
            }
            Axis::Vertical => {
                // Table takes available height (internal scrolling)
                match len_req {
                    LenReq::FitContent(available) => available,
                    LenReq::MinContent | LenReq::MaxContent => 400.0, // Default fallback
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
        let width_changed = self.last_layout_width.map_or(true, |w| (w - content_width).abs() > 0.5);
        if width_changed && !self.size_change_pending {
            self.last_layout_width = Some(content_width);
            ctx.submit_action::<Self::Action>(TableWidgetAction::SizeChanged { width: content_width });
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
        let header_rect =
            Rect::new(0.0, 0.0, self.size.width - SCROLLBAR_WIDTH, self.header_height);
        painter.fill(header_rect, self.style.header_bg).draw();

        // Note: Children (header and rows) paint themselves based on their layout positions
        // The header is placed at the top (y=0) so it naturally overlays row content

        // 4. Paint scrollbar
        self.paint_scrollbar(painter);

        // Suppress unused variable warning
        let _ = content_rect;
    }

    fn get_cursor(&self, ctx: &QueryCtx<'_>, pos: Point) -> CursorIcon {
        let local_pos = ctx.to_local(pos);
        if self.scrollbar_hit_test(local_pos) {
            CursorIcon::Default
        } else {
            CursorIcon::Default
        }
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
