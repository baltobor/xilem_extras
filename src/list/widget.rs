//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! List widget with keyboard navigation and optional sections.
//!
//! This widget handles:
//! - Keyboard navigation (ArrowUp, ArrowDown, Home, End, PageUp, PageDown)
//! - Row click events with modifier support
//! - Scroll management for virtualization
//! - Optional section headers

use std::any::TypeId;
use std::collections::HashMap;
use std::ops::Range;

use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, CursorIcon, EventCtx, LayoutCtx, MeasureCtx, NewWidget,
    PaintCtx, PointerButtonEvent, PointerEvent, PointerScrollEvent, PointerUpdate,
    PropertiesMut, PropertiesRef, QueryCtx, RegisterCtx, ScrollDelta, TextEvent, Update,
    UpdateCtx, Widget, WidgetId, WidgetMut, WidgetPod,
    keyboard::{Key, NamedKey},
};
use tracing::{trace_span, Span};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Axis, Point, Rect, RoundedRect, Size};
use xilem::masonry::layout::LenReq;
use xilem::masonry::peniko::Color;
use xilem::masonry::properties::Background;

/// Scrollbar configuration.
const SCROLLBAR_WIDTH: f64 = 8.0;
const SCROLLBAR_MIN_THUMB: f64 = 20.0;
const SCROLLBAR_CORNER_RADIUS: f64 = 4.0;
const LINE_HEIGHT_PX: f64 = 28.0;
const PAGE_HEIGHT_PX: f64 = 400.0;

/// Action sent when visible range changes.
#[derive(Debug, Clone, PartialEq)]
pub struct ListRangeAction {
    /// Previous active range.
    pub old_range: Range<usize>,
    /// New target range to load.
    pub target_range: Range<usize>,
}

/// Action sent when a row is clicked or selected via keyboard.
#[derive(Debug, Clone)]
pub struct ListRowAction {
    /// Row index that was clicked or navigated to.
    pub row_index: usize,
    /// Click count (1 = single, 2 = double, 0 = keyboard nav).
    pub click_count: u32,
    /// Whether shift was held.
    pub shift: bool,
    /// Whether command/ctrl was held.
    pub command: bool,
}

/// Combined action for list events.
#[derive(Debug, Clone)]
pub enum ListWidgetAction {
    /// Range of visible rows changed.
    RangeChanged(ListRangeAction),
    /// Row was clicked or selected via keyboard.
    RowSelect(ListRowAction),
    /// Row was activated (double-click or Enter).
    RowActivate(ListRowAction),
}

/// Scroll state for virtualized list.
#[derive(Debug, Clone)]
pub struct ListScrollState {
    /// Current scroll offset in pixels.
    pub scroll_offset: f64,
    /// Viewport height in pixels.
    pub viewport_height: f64,
    /// Row height in pixels.
    pub row_height: f64,
    /// Total item count.
    pub item_count: usize,
    /// Buffer rows above/below viewport.
    pub buffer: usize,
    /// Currently active (loaded) range of rows.
    pub active_range: Range<usize>,
}

impl Default for ListScrollState {
    fn default() -> Self {
        Self {
            scroll_offset: 0.0,
            viewport_height: 600.0,
            row_height: 28.0,
            item_count: 0,
            buffer: 3,
            active_range: 0..0,
        }
    }
}

impl ListScrollState {
    /// Creates a new scroll state.
    pub fn new(row_height: f64) -> Self {
        Self {
            row_height,
            ..Self::default()
        }
    }

    /// Sets the item count.
    pub fn set_item_count(&mut self, count: usize) {
        self.item_count = count;
        // Clamp scroll offset if needed
        let max_scroll = self.max_scroll();
        if self.scroll_offset > max_scroll {
            self.scroll_offset = max_scroll;
        }
    }

    /// Sets the viewport height.
    pub fn set_viewport_height(&mut self, height: f64) {
        self.viewport_height = height;
    }

    /// Computes the target range based on current scroll position.
    pub fn compute_target_range(&self) -> Range<usize> {
        self.visible_range()
    }

    /// Total content height.
    pub fn content_height(&self) -> f64 {
        self.item_count as f64 * self.row_height
    }

    /// Maximum scroll offset.
    pub fn max_scroll(&self) -> f64 {
        (self.content_height() - self.viewport_height).max(0.0)
    }

    /// Scroll by a delta amount.
    pub fn scroll_by(&mut self, delta: f64) {
        self.scroll_offset = (self.scroll_offset + delta).clamp(0.0, self.max_scroll());
    }

    /// Scroll to a specific offset.
    pub fn scroll_to(&mut self, offset: f64) {
        self.scroll_offset = offset.clamp(0.0, self.max_scroll());
    }

    /// Ensure a row is visible.
    pub fn ensure_visible(&mut self, row_index: usize) {
        let row_top = row_index as f64 * self.row_height;
        let row_bottom = row_top + self.row_height;

        if row_top < self.scroll_offset {
            self.scroll_offset = row_top;
        } else if row_bottom > self.scroll_offset + self.viewport_height {
            self.scroll_offset = row_bottom - self.viewport_height;
        }
    }

    /// Calculate the visible range of items.
    pub fn visible_range(&self) -> Range<usize> {
        if self.item_count == 0 || self.row_height <= 0.0 {
            return 0..0;
        }

        let first_raw = (self.scroll_offset / self.row_height).floor() as usize;
        let first = first_raw.saturating_sub(self.buffer);

        let visible_in_viewport = (self.viewport_height / self.row_height).ceil() as usize;
        let last = (first_raw + visible_in_viewport + self.buffer).min(self.item_count);

        first..last
    }

    /// Y position of a row relative to scroll.
    pub fn row_y(&self, row_index: usize) -> f64 {
        row_index as f64 * self.row_height - self.scroll_offset
    }

    /// Row index at a given y position (relative to viewport).
    pub fn row_at_y(&self, y: f64) -> Option<usize> {
        let adjusted_y = y + self.scroll_offset;
        if adjusted_y < 0.0 {
            return None;
        }
        let row = (adjusted_y / self.row_height).floor() as usize;
        if row < self.item_count {
            Some(row)
        } else {
            None
        }
    }
}

/// Section definition for grouped lists.
#[derive(Debug, Clone)]
pub struct ListSection {
    /// Section title.
    pub title: String,
    /// Range of item indices in this section.
    pub item_range: Range<usize>,
}

impl ListSection {
    /// Creates a new section.
    pub fn new(title: impl Into<String>, item_range: Range<usize>) -> Self {
        Self {
            title: title.into(),
            item_range,
        }
    }

    /// Returns the number of items in this section.
    pub fn len(&self) -> usize {
        self.item_range.len()
    }

    /// Returns true if this section is empty.
    pub fn is_empty(&self) -> bool {
        self.item_range.is_empty()
    }
}

/// List widget style configuration.
#[derive(Debug, Clone)]
pub struct ListWidgetStyle {
    /// Row height in pixels.
    pub row_height: f64,
    /// Section header height in pixels.
    pub header_height: f64,
    /// Background color.
    pub background: Color,
    /// Scrollbar track color.
    pub scrollbar_track: Color,
    /// Scrollbar thumb color.
    pub scrollbar_thumb: Color,
    /// Scrollbar thumb hover color.
    pub scrollbar_thumb_hover: Color,
}

impl Default for ListWidgetStyle {
    fn default() -> Self {
        Self {
            row_height: 28.0,
            header_height: 32.0,
            background: Color::TRANSPARENT,
            scrollbar_track: Color::from_rgba8(60, 58, 55, 100),
            scrollbar_thumb: Color::from_rgba8(90, 87, 82, 180),
            scrollbar_thumb_hover: Color::from_rgba8(110, 107, 102, 200),
        }
    }
}

/// Virtualized list widget with keyboard navigation.
pub struct ListWidget {
    /// Row widgets (sparse storage by index).
    rows: HashMap<usize, WidgetPod<dyn Widget>>,
    /// Section header widgets (by section index).
    section_headers: HashMap<usize, WidgetPod<dyn Widget>>,
    /// Scroll state.
    state: ListScrollState,
    /// Section definitions (if using sections).
    sections: Vec<ListSection>,
    /// Widget size from last layout.
    size: Size,
    /// Style configuration.
    style: ListWidgetStyle,
    /// Currently focused row index.
    focused_row_index: Option<usize>,
    /// Whether we're waiting for view to handle range action.
    action_pending: bool,
    /// Scrollbar hover state.
    scrollbar_hovered: bool,
    /// Scrollbar drag state.
    scrollbar_dragging: bool,
    /// Scrollbar drag start Y.
    scrollbar_drag_start_y: f64,
    /// Scrollbar drag start scroll position.
    scrollbar_drag_start_position: f64,
}

impl ListWidget {
    /// Creates a new list widget.
    pub fn new(row_height: f64) -> Self {
        Self {
            rows: HashMap::new(),
            section_headers: HashMap::new(),
            state: ListScrollState::new(row_height),
            sections: Vec::new(),
            size: Size::ZERO,
            style: ListWidgetStyle::default(),
            focused_row_index: None,
            action_pending: false,
            scrollbar_hovered: false,
            scrollbar_dragging: false,
            scrollbar_drag_start_y: 0.0,
            scrollbar_drag_start_position: 0.0,
        }
    }

    /// Gets the scroll state.
    pub fn state(&self) -> &ListScrollState {
        &self.state
    }

    /// Gets mutable access to the scroll state.
    pub fn state_mut(&mut self) -> &mut ListScrollState {
        &mut self.state
    }

    /// Gets the focused row index.
    pub fn focused_row(&self) -> Option<usize> {
        self.focused_row_index
    }

    /// Returns row indices currently in the widget.
    pub fn row_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.rows.keys().copied()
    }

    /// Returns the current content width (excluding scrollbar).
    pub fn content_width(&self) -> f64 {
        self.size.width - SCROLLBAR_WIDTH
    }

    // ========================================================================
    // WidgetMut-based methods (for View layer interaction)
    // ========================================================================

    /// Sets the item count.
    pub fn set_item_count(this: &mut WidgetMut<'_, Self>, count: usize) {
        this.widget.state.set_item_count(count);
        this.ctx.request_layout();
    }

    /// Sets row height.
    pub fn set_row_height(this: &mut WidgetMut<'_, Self>, height: f64) {
        this.widget.state.row_height = height;
        this.ctx.request_layout();
    }

    /// Indicates that `action` is about to be handled by the view.
    ///
    /// This must be called before `add_row` or `remove_row`.
    pub fn will_handle_action(this: &mut WidgetMut<'_, Self>, action: &ListRangeAction) {
        if this.widget.state.active_range != action.old_range {
            tracing::warn!(
                "Handling a ListRangeAction with the wrong range; got {:?}, expected {:?}",
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

    /// Add a row widget at an index.
    ///
    /// This should be done only in the handling of a [`ListRangeAction`].
    /// This must be called after [`ListWidget::will_handle_action`].
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
            tracing::warn!("Tried to add row {index} twice to ListWidget");
        }
    }

    /// Remove a row widget.
    ///
    /// This should be done only in the handling of a [`ListRangeAction`].
    /// This must be called after [`ListWidget::will_handle_action`].
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

    // ========================================================================
    // Internal methods
    // ========================================================================

    /// Navigate to a specific row.
    fn navigate_to_row(&mut self, ctx: &mut EventCtx, row_index: usize, shift: bool, command: bool) {
        let row_index = row_index.min(self.state.item_count.saturating_sub(1));
        self.focused_row_index = Some(row_index);
        self.state.ensure_visible(row_index);

        ctx.submit_action::<ListWidgetAction>(ListWidgetAction::RowSelect(ListRowAction {
            row_index,
            click_count: 0,
            shift,
            command,
        }));
        ctx.request_layout();
    }

    /// Convert scroll delta to pixels.
    fn scroll_delta_to_pixels(delta: &ScrollDelta) -> f64 {
        match delta {
            ScrollDelta::PixelDelta(pos) => pos.y,
            ScrollDelta::LineDelta(_x, y) => (*y as f64) * LINE_HEIGHT_PX,
            ScrollDelta::PageDelta(_x, y) => (*y as f64) * PAGE_HEIGHT_PX,
        }
    }

    /// Scrollbar rect.
    fn scrollbar_rect(&self) -> Rect {
        Rect::new(
            self.size.width - SCROLLBAR_WIDTH,
            0.0,
            self.size.width,
            self.size.height,
        )
    }

    /// Scrollbar thumb rect.
    fn scrollbar_thumb_rect(&self) -> Rect {
        let content_height = self.state.content_height();
        if content_height <= self.state.viewport_height {
            return Rect::ZERO;
        }

        let track = self.scrollbar_rect();
        let thumb_ratio = self.state.viewport_height / content_height;
        let thumb_height = (thumb_ratio * track.height()).max(SCROLLBAR_MIN_THUMB);
        let available_track = track.height() - thumb_height;

        let scroll_ratio = if self.state.max_scroll() > 0.0 {
            self.state.scroll_offset / self.state.max_scroll()
        } else {
            0.0
        };
        let thumb_top = scroll_ratio * available_track;

        Rect::new(
            track.x0,
            thumb_top,
            track.x1,
            thumb_top + thumb_height,
        )
    }

    /// Check if point is in scrollbar area.
    fn scrollbar_hit_test(&self, pos: Point) -> bool {
        self.scrollbar_rect().contains(pos)
    }

    /// Paint the scrollbar.
    fn paint_scrollbar(&self, painter: &mut Painter<'_>) {
        let content_height = self.state.content_height();
        if content_height <= self.state.viewport_height {
            return;
        }

        // Track
        let track = self.scrollbar_rect();
        let track_rounded = RoundedRect::from_rect(track, SCROLLBAR_CORNER_RADIUS);
        painter.fill(&track_rounded, self.style.scrollbar_track).draw();

        // Thumb
        let thumb = self.scrollbar_thumb_rect();
        let thumb_rounded = RoundedRect::from_rect(thumb, SCROLLBAR_CORNER_RADIUS);
        let thumb_color = if self.scrollbar_dragging || self.scrollbar_hovered {
            self.style.scrollbar_thumb_hover
        } else {
            self.style.scrollbar_thumb
        };
        painter.fill(&thumb_rounded, thumb_color).draw();
    }
}

impl Widget for ListWidget {
    type Action = ListWidgetAction;

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        match event {
            PointerEvent::Scroll(PointerScrollEvent { delta, .. }) => {
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
                        self.scrollbar_drag_start_position = self.state.scroll_offset;
                        ctx.set_handled();
                        return;
                    } else {
                        // Click on track - jump to position
                        let track = self.scrollbar_rect();
                        let click_ratio = (pos.y - track.y0) / track.height();
                        let target_scroll = click_ratio * self.state.max_scroll();
                        self.state.scroll_to(target_scroll);
                        ctx.request_layout();
                        ctx.set_handled();
                        return;
                    }
                }

                // Check row click
                if let Some(row_index) = self.state.row_at_y(pos.y) {
                    ctx.request_focus();
                    self.focused_row_index = Some(row_index);
                    let click_count = state.count as u32;
                    let shift = state.modifiers.shift();
                    let command = state.modifiers.meta() || state.modifiers.ctrl();

                    if click_count >= 2 {
                        ctx.submit_action::<ListWidgetAction>(ListWidgetAction::RowActivate(ListRowAction {
                            row_index,
                            click_count,
                            shift,
                            command,
                        }));
                    } else {
                        ctx.submit_action::<ListWidgetAction>(ListWidgetAction::RowSelect(ListRowAction {
                            row_index,
                            click_count,
                            shift,
                            command,
                        }));
                    }
                    ctx.set_handled();
                }
            }
            PointerEvent::Move(PointerUpdate { current, .. }) => {
                let pos = ctx.local_position(current.position);

                if self.scrollbar_dragging {
                    let track = self.scrollbar_rect();
                    let thumb = self.scrollbar_thumb_rect();
                    let thumb_height = thumb.height();
                    let available_track = track.height() - thumb_height;

                    let delta_y = pos.y - self.scrollbar_drag_start_y;
                    let delta_scroll = if available_track > 0.0 {
                        delta_y / available_track * self.state.max_scroll()
                    } else {
                        0.0
                    };

                    self.state.scroll_to(self.scrollbar_drag_start_position + delta_scroll);
                    ctx.request_layout();
                    ctx.set_handled();
                } else {
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
        if let TextEvent::Keyboard(key_event) = event {
            if key_event.state.is_up() {
                return;
            }

            let shift = key_event.modifiers.shift();
            let command = key_event.modifiers.meta() || key_event.modifiers.ctrl();

            match &key_event.key {
                Key::Named(NamedKey::ArrowUp) => {
                    let new_idx = self.focused_row_index
                        .map(|i| i.saturating_sub(1))
                        .unwrap_or(0);
                    self.navigate_to_row(ctx, new_idx, shift, command);
                    ctx.set_handled();
                }
                Key::Named(NamedKey::ArrowDown) => {
                    let max_idx = self.state.item_count.saturating_sub(1);
                    let new_idx = self.focused_row_index
                        .map(|i| (i + 1).min(max_idx))
                        .unwrap_or(0);
                    self.navigate_to_row(ctx, new_idx, shift, command);
                    ctx.set_handled();
                }
                Key::Named(NamedKey::Home) => {
                    self.navigate_to_row(ctx, 0, shift, command);
                    ctx.set_handled();
                }
                Key::Named(NamedKey::End) => {
                    let max_idx = self.state.item_count.saturating_sub(1);
                    self.navigate_to_row(ctx, max_idx, shift, command);
                    ctx.set_handled();
                }
                Key::Named(NamedKey::PageUp) => {
                    let page_rows = (self.state.viewport_height / self.state.row_height).floor() as usize;
                    let new_idx = self.focused_row_index
                        .map(|i| i.saturating_sub(page_rows))
                        .unwrap_or(0);
                    self.navigate_to_row(ctx, new_idx, shift, command);
                    ctx.set_handled();
                }
                Key::Named(NamedKey::PageDown) => {
                    let page_rows = (self.state.viewport_height / self.state.row_height).floor() as usize;
                    let max_idx = self.state.item_count.saturating_sub(1);
                    let new_idx = self.focused_row_index
                        .map(|i| (i + page_rows).min(max_idx))
                        .unwrap_or(0);
                    self.navigate_to_row(ctx, new_idx, shift, command);
                    ctx.set_handled();
                }
                Key::Named(NamedKey::Enter) => {
                    if let Some(row_index) = self.focused_row_index {
                        ctx.submit_action::<ListWidgetAction>(ListWidgetAction::RowActivate(ListRowAction {
                            row_index,
                            click_count: 2,
                            shift,
                            command,
                        }));
                        ctx.set_handled();
                    }
                }
                _ => {}
            }
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
        for row in self.rows.values_mut() {
            ctx.register_child(row);
        }
        for header in self.section_headers.values_mut() {
            ctx.register_child(header);
        }
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
                match len_req {
                    LenReq::FitContent(available) => available,
                    LenReq::MinContent => 200.0,
                    LenReq::MaxContent => 600.0, // Reasonable default width
                }
            }
            Axis::Vertical => {
                match len_req {
                    LenReq::FitContent(available) => available,
                    LenReq::MinContent => 100.0,
                    // For max content, return actual content height or reasonable default
                    LenReq::MaxContent => {
                        let content_height = self.state.content_height();
                        if content_height > 0.0 {
                            content_height
                        } else {
                            400.0
                        }
                    }
                }
            }
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, size: Size) {
        self.size = size;
        self.state.set_viewport_height(size.height);

        // Content width excluding scrollbar
        let content_width = size.width - SCROLLBAR_WIDTH;

        // Check if range needs update
        let target_range = self.state.compute_target_range();
        if target_range != self.state.active_range && !self.action_pending {
            ctx.submit_action::<ListWidgetAction>(ListWidgetAction::RangeChanged(ListRangeAction {
                old_range: self.state.active_range.clone(),
                target_range: target_range.clone(),
            }));
            self.action_pending = true;
        }

        // Set clip path for content area
        let clip_rect = Rect::new(0.0, 0.0, content_width, size.height);
        ctx.set_clip_path(clip_rect);

        // Layout active rows
        for (&row_index, row_widget) in &mut self.rows {
            if !self.state.active_range.contains(&row_index) {
                // Stash rows outside active range
                ctx.set_stashed(row_widget, true);
                continue;
            }

            ctx.set_stashed(row_widget, false);
            let row_size = Size::new(content_width, self.state.row_height);
            ctx.run_layout(row_widget, row_size);

            // Position row based on scroll state
            let y = self.state.row_y(row_index);
            ctx.place_child(row_widget, Point::new(0.0, y));
        }
    }

    fn paint(
        &mut self,
        ctx: &mut PaintCtx<'_>,
        props: &PropertiesRef<'_>,
        painter: &mut Painter<'_>,
    ) {
        // Background
        let rect = Rect::from_origin_size(Point::ZERO, self.size);
        {
            let cache = ctx.property_cache();
            let bg = props.get::<Background>(cache);
            let brush = bg.get_peniko_brush_for_rect(rect);
            painter.fill(rect, &brush).draw();
        }

        // Paint scrollbar
        self.paint_scrollbar(painter);
    }

    fn get_cursor(&self, _ctx: &QueryCtx<'_>, _pos: Point) -> CursorIcon {
        CursorIcon::Default
    }

    fn accessibility_role(&self) -> Role {
        Role::List
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        _node: &mut Node,
    ) {
    }

    fn children_ids(&self) -> ChildrenIds {
        let mut ids = Vec::with_capacity(self.rows.len() + self.section_headers.len());
        for row in self.rows.values() {
            ids.push(row.id());
        }
        for header in self.section_headers.values() {
            ids.push(header.id());
        }
        ChildrenIds::from_vec(ids)
    }

    fn accepts_focus(&self) -> bool {
        true
    }

    fn accepts_text_input(&self) -> bool {
        false
    }

    fn propagates_pointer_interaction(&self) -> bool {
        true
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("ListWidget", id = id.trace())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_state_new() {
        let state = ListScrollState::new(28.0);
        assert_eq!(state.row_height, 28.0);
        assert_eq!(state.scroll_offset, 0.0);
    }

    #[test]
    fn scroll_state_visible_range() {
        let mut state = ListScrollState::new(28.0);
        state.viewport_height = 280.0;
        state.item_count = 100;

        let range = state.visible_range();
        assert_eq!(range.start, 0);
        assert!(range.end >= 10);
    }

    #[test]
    fn scroll_state_ensure_visible() {
        let mut state = ListScrollState::new(28.0);
        state.viewport_height = 280.0;
        state.item_count = 100;
        state.scroll_offset = 0.0;

        state.ensure_visible(50);
        assert!(state.scroll_offset > 0.0);
    }

    #[test]
    fn section_new() {
        let section = ListSection::new("Test", 0..10);
        assert_eq!(section.title, "Test");
        assert_eq!(section.len(), 10);
    }
}
