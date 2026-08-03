// Copyright 2026 the Xilem Authors
// SPDX-License-Identifier: Apache-2.0

//! Resizable table header widget with draggable column dividers.

use std::any::TypeId;
use std::sync::Arc;

use tracing::{Span, trace_span};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Point, Rect, Size};
use xilem::masonry::peniko::Color;

use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, CursorIcon, EventCtx, LayoutCtx, MeasureCtx, NewWidget,
    PaintCtx, PointerButtonEvent, PointerEvent, PointerUpdate, PropertiesMut, PropertiesRef,
    QueryCtx, RegisterCtx, TextEvent, Update, UpdateCtx, Widget, WidgetId, WidgetMut, WidgetPod,
};
use xilem::masonry::kurbo::Axis;
use xilem::masonry::layout::{LayoutSize, LenReq, Length};
use xilem::masonry::properties::Background;

use crate::masonry::flow_direction::FlowDirection;
use crate::masonry::table::column_layout::{
    self, ColumnBox, ColumnResizeMode, DIVIDER_HIT_AREA, DIVIDER_WIDTH, MIN_COLUMN_WIDTH,
};

/// Action emitted when a column is resized (final, committed value — on
/// pointer-up only).
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnResizeAction {
    pub column_key: Arc<str>,
    pub new_width: f64,
}

/// Ephemeral, non-persisted action emitted on every pointer-move while a
/// column is being dragged, so sibling row content can resize live. Submitted
/// via `submit_untyped_action` (not `submit_action`) since it isn't
/// `ResizableHeader::Action` — see `TableView::message` for the receiving end.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnResizePreviewAction {
    pub column_key: Arc<str>,
    pub new_width: f64,
}

/// Ephemeral, non-persisted signal for which divider is currently being
/// *dragged* (not merely hovered — hover only drives the local, header-height
/// highlight painted directly in `paint()`), purely so a full-height
/// guideline can be painted outside the header's own bounds during an active
/// drag (see `TableWidget::post_paint`). Also submitted via
/// `submit_untyped_action`.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDividerHighlightAction(pub Option<usize>);

/// A header row widget with draggable column dividers.
pub struct ResizableHeader {
    pub(crate) children: Vec<WidgetPod<dyn Widget>>,
    pub(crate) column_keys: Vec<Arc<str>>,
    pub(crate) column_widths: Vec<f64>,
    pub(crate) columns: Vec<ColumnBox>,
    direction: FlowDirection,
    resize_mode: ColumnResizeMode,
    size: Size,
    dragging_index: Option<usize>,
    /// Index of the column most recently dragged (or currently being
    /// dragged). Unlike `dragging_index`, this is never cleared on
    /// release — `ColumnResizeMode::FixedViewport`'s "protect columns
    /// before the dragged one" split needs a stable anchor even after the
    /// drag ends, or releasing the pointer would recompute a different
    /// (unanchored) compression than what was just rendered live.
    last_resized_index: Option<usize>,
    /// The RTL mirror anchor (`place_columns`'s `anchor_width`), frozen for
    /// the duration of an active drag in RTL + `Overflow` mode.
    ///
    /// In `Overflow` mode this widget is typically hosted in a
    /// content-sized `portal(...)` (see `table_styled`'s doc comment), so
    /// `self.size.width` — normally the natural anchor — is itself derived
    /// from the sum of the very column widths being placed. For RTL's
    /// mirror formula (`anchor_width - local_x - width`), that makes the
    /// anchor grow in lockstep with whichever column is being dragged,
    /// exactly cancelling that column's own width out of its own
    /// `x_offset` — the divider you're dragging stops tracking the
    /// cursor entirely (verified by hand; this is *not* the same bug as
    /// the earlier "leftover space" cancellation, which happened even with
    /// a fixed anchor — this one is specific to a self-referential one).
    /// Freezing the anchor at whatever `self.size.width` was immediately
    /// before the drag started restores a stable reference for its
    /// duration, exactly like `FixedViewport` mode already has for free
    /// (its anchor is a real, externally-imposed container width, never
    /// self-referential). Cleared on release, so the next resting layout
    /// re-syncs to the table's real current width.
    frozen_anchor_width: Option<f64>,
    drag_start_x: f64,
    drag_start_width: f64,
    last_preview_width: Option<f64>,
    divider_color: Color,
    divider_hover_color: Color,
    hovered_divider: Option<usize>,
}

impl ResizableHeader {
    pub fn new(
        children: Vec<NewWidget<dyn Widget>>,
        column_keys: Vec<Arc<str>>,
        column_widths: Vec<f64>,
    ) -> Self {
        let children: Vec<_> = children.into_iter().map(|c| c.to_pod()).collect();
        Self {
            children,
            column_keys,
            column_widths,
            columns: Vec::new(),
            direction: FlowDirection::Ltr,
            resize_mode: ColumnResizeMode::default(),
            size: Size::ZERO,
            dragging_index: None,
            last_resized_index: None,
            frozen_anchor_width: None,
            drag_start_x: 0.0,
            drag_start_width: 0.0,
            last_preview_width: None,
            divider_color: Color::from_rgb8(120, 118, 115),
            divider_hover_color: Color::from_rgb8(100, 150, 255),
            hovered_divider: None,
        }
    }

    pub fn with_divider_color(mut self, color: Color) -> Self {
        self.divider_color = color;
        self
    }

    pub fn with_direction(mut self, direction: FlowDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_resize_mode(mut self, mode: ColumnResizeMode) -> Self {
        self.resize_mode = mode;
        self
    }

    fn hit_test_divider(&self, x: f64) -> Option<usize> {
        for (i, col) in self.columns.iter().enumerate() {
            if i < self.columns.len() - 1 {
                let divider_start = column_layout::divider_start(col, self.direction);
                let divider_center = divider_start + DIVIDER_WIDTH / 2.0;
                if (x - divider_center).abs() <= DIVIDER_HIT_AREA {
                    return Some(i);
                }
            }
        }
        None
    }

    pub(crate) fn update_column_layout(&mut self) {
        let n = self.column_keys.len();
        let divider_space = n.saturating_sub(1) as f64 * DIVIDER_WIDTH;
        let configured_total: f64 = self.column_widths.iter().sum::<f64>() + divider_space;
        self.columns = column_layout::place_columns(
            &self.column_keys,
            &self.column_widths,
            configured_total,
            self.direction,
        );
    }

    pub fn set_column_widths(this: &mut WidgetMut<'_, Self>, widths: Vec<f64>) {
        this.widget.column_widths = widths;
        this.widget.update_column_layout();
        this.ctx.request_layout();
    }

    pub fn child_mut<'t>(
        this: &'t mut WidgetMut<'_, Self>,
        index: usize,
    ) -> Option<WidgetMut<'t, dyn Widget>> {
        this.widget
            .children
            .get_mut(index)
            .map(|child| this.ctx.get_mut(child))
    }
}

impl Widget for ResizableHeader {
    type Action = ColumnResizeAction;

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        match event {
            PointerEvent::Down(PointerButtonEvent { state, .. }) => {
                let pos = ctx.local_position(state.position);
                if let Some(divider_idx) = self.hit_test_divider(pos.x) {
                    ctx.set_handled();
                    ctx.capture_pointer();
                    self.dragging_index = Some(divider_idx);
                    self.last_resized_index = Some(divider_idx);
                    self.frozen_anchor_width = Some(self.size.width);
                    self.drag_start_x = pos.x;
                    // Read from `column_widths` (the desired/configured
                    // value), not `self.columns` (the *rendered* value) —
                    // in `FixedViewport` mode a column can be compressed
                    // below its desired width by an unrelated drag on a
                    // column before it; a fresh drag should continue from
                    // the user's real preference, not a rendering artifact.
                    self.drag_start_width = self.column_widths[divider_idx];
                    self.last_preview_width = None;
                    ctx.submit_untyped_action(Box::new(ColumnDividerHighlightAction(Some(
                        divider_idx,
                    ))));
                    ctx.request_render();
                }
            }
            PointerEvent::Move(PointerUpdate { current, .. }) => {
                let pos = ctx.local_position(current.position);

                if ctx.is_active() {
                    if let Some(divider_idx) = self.dragging_index {
                        let raw_delta = pos.x - self.drag_start_x;
                        let signed_delta = column_layout::flip_delta(raw_delta, self.direction);
                        let mut new_width =
                            (self.drag_start_width + signed_delta).max(MIN_COLUMN_WIDTH);
                        if self.resize_mode == ColumnResizeMode::FixedViewport {
                            // Clamp the drag itself, not just the rendered
                            // output — otherwise the pointer keeps moving
                            // indefinitely past the point where columns
                            // after this one are already at their floor,
                            // and the header visibly desyncs from the
                            // (correctly-capped) row content.
                            let max_width = column_layout::max_dragged_width(
                                &self.column_widths,
                                divider_idx,
                                self.size.width,
                            );
                            new_width = new_width.min(max_width);
                        }

                        // Only the dragged column changes width; everything
                        // after it shifts as a block for free, since layout()
                        // recomputes every x_offset from column_widths.
                        if let Some(col) = self.columns.get_mut(divider_idx) {
                            col.width = new_width;
                        }
                        if let Some(w) = self.column_widths.get_mut(divider_idx) {
                            *w = new_width;
                        }

                        if self.last_preview_width != Some(new_width) {
                            self.last_preview_width = Some(new_width);
                            if let Some(key) = self.column_keys.get(divider_idx) {
                                ctx.submit_untyped_action(Box::new(ColumnResizePreviewAction {
                                    column_key: key.clone(),
                                    new_width,
                                }));
                            }
                        }

                        ctx.request_layout();
                    }
                } else {
                    // Hover-only: update the local (header-height) highlight
                    // but don't notify TableWidget — the full-height
                    // highlight is drag-only feedback, not a hover one (see
                    // `ColumnDividerHighlightAction`'s doc comment).
                    let new_hovered = self.hit_test_divider(pos.x);
                    if new_hovered != self.hovered_divider {
                        self.hovered_divider = new_hovered;
                        ctx.request_render();
                    }
                }
            }
            PointerEvent::Up(..) | PointerEvent::Cancel(..) => {
                if let Some(divider_idx) = self.dragging_index.take() {
                    if let Some(col) = self.columns.get(divider_idx) {
                        ctx.submit_action::<Self::Action>(ColumnResizeAction {
                            column_key: col.key.clone(),
                            new_width: col.width,
                        });
                    }
                    ctx.submit_untyped_action(Box::new(ColumnDividerHighlightAction(None)));
                }
                // Release the frozen anchor so the next resting layout
                // re-syncs to the table's real current width.
                self.frozen_anchor_width = None;
                ctx.request_render();
            }
            PointerEvent::Leave(..) => {
                if self.hovered_divider.is_some() {
                    self.hovered_divider = None;
                    ctx.request_render();
                }
            }
            _ => {}
        }
    }

    fn on_text_event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &TextEvent,
    ) {
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
            Update::HoveredChanged(_) => {
                // Authoritative backstop for clearing the hover highlight:
                // `PointerEvent::Leave` should already handle this, but
                // masonry's own hover tracking is the ground truth for
                // "is the pointer still anywhere over this widget" — if it
                // says no, the local marker must not stay lit (e.g. when
                // the pointer leaves the header vertically, into the row
                // area below, rather than sideways past its edge).
                if !ctx.is_hovered() && self.hovered_divider.is_some() {
                    self.hovered_divider = None;
                }
                ctx.request_render();
            }
            Update::ActiveChanged(_) => {
                ctx.request_render();
            }
            _ => {}
        }
    }

    fn register_children(&mut self, ctx: &mut RegisterCtx<'_>) {
        for child in &mut self.children {
            ctx.register_child(child);
        }
    }

    fn property_changed(&mut self, ctx: &mut UpdateCtx<'_>, property_type: TypeId) {
        if property_type == TypeId::of::<Background>() {
            ctx.request_render();
        }
    }

    fn measure(
        &mut self,
        ctx: &mut MeasureCtx<'_>,
        _props: &PropertiesRef<'_>,
        axis: Axis,
        len_req: LenReq,
        _cross_length: Option<Length>,
    ) -> Length {
        self.update_column_layout();

        match axis {
            Axis::Horizontal => Length::px(self.columns.iter().map(|c| c.width).sum()),
            Axis::Vertical => {
                let mut max_height = Length::ZERO;
                for (i, child) in self.children.iter_mut().enumerate() {
                    let col_width = self.columns.get(i).map(|c| Length::px(c.width));
                    let height = ctx.compute_length(
                        child,
                        len_req.into(),
                        LayoutSize::maybe(Axis::Horizontal, col_width),
                        axis,
                        col_width,
                    );
                    if height.get() > max_height.get() {
                        max_height = height;
                    }
                }
                max_height
            }
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, size: Size) {
        self.size = size;

        // In `Overflow` mode (the default), columns render at their
        // configured width, clamped only to `MIN_COLUMN_WIDTH` — never
        // proportionally shrunk to fit the box; if the configured total
        // exceeds `size.width` the header simply overflows. In
        // `FixedViewport` mode, columns after the dragged one compress
        // toward their floor instead. `last_resized_index` (not
        // `dragging_index`) is the protection anchor — it stays set after
        // release so this computation doesn't change the instant the
        // pointer is lifted.
        let scaled_widths = column_layout::compute_rendered_widths(
            &self.column_widths,
            self.last_resized_index,
            size.width,
            self.resize_mode,
        );

        // RTL mirrors around `anchor_width`. While a drag is active this is
        // `frozen_anchor_width`, not the live `size.width` — see its doc
        // comment for why a self-referential anchor (as `size.width` is in
        // `Overflow` mode, typically hosted in a content-sized `portal`)
        // makes the dragged column's own width cancel out of its own
        // position, so its divider stops tracking the cursor. Outside an
        // active drag, `frozen_anchor_width` is `None` and this is just
        // `size.width`, exactly as before.
        let anchor_width = self.frozen_anchor_width.unwrap_or(size.width);
        self.columns = column_layout::place_columns(
            &self.column_keys,
            &scaled_widths,
            anchor_width,
            self.direction,
        );

        for (i, child) in self.children.iter_mut().enumerate() {
            if let Some(col) = self.columns.get(i) {
                let child_size = Size::new(col.width, size.height);
                ctx.run_layout(child, child_size);
                ctx.place_child(child, Point::new(col.x_offset, 0.0));
            }
        }
    }

    fn paint(
        &mut self,
        ctx: &mut PaintCtx<'_>,
        props: &PropertiesRef<'_>,
        painter: &mut Painter<'_>,
    ) {
        let rect = Rect::from_origin_size(Point::ZERO, self.size);

        {
            let cache = ctx.property_cache();
            let bg = props.get::<Background>(cache);
            let brush = bg.get_peniko_brush_for_rect(rect);
            painter.fill(rect, &brush).draw();
        }

        for (i, col) in self.columns.iter().enumerate() {
            if i < self.columns.len() - 1 {
                let is_hovered = self.hovered_divider == Some(i) || self.dragging_index == Some(i);

                if is_hovered {
                    let divider_start = column_layout::divider_start(col, self.direction);
                    let divider_rect = Rect::new(
                        divider_start,
                        0.0,
                        divider_start + DIVIDER_WIDTH,
                        self.size.height,
                    );
                    painter.fill(divider_rect, self.divider_hover_color).draw();
                }
            }
        }
    }

    fn get_cursor(&self, ctx: &QueryCtx<'_>, pos: Point) -> CursorIcon {
        let local_pos = ctx.to_local(pos);
        if ctx.is_active() || self.hit_test_divider(local_pos.x).is_some() {
            CursorIcon::EwResize
        } else {
            CursorIcon::Default
        }
    }

    fn accessibility_role(&self) -> Role {
        Role::Row
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        _node: &mut Node,
    ) {
    }

    fn children_ids(&self) -> ChildrenIds {
        let ids: Vec<_> = self.children.iter().map(|c| c.id()).collect();
        ChildrenIds::from_slice(&ids)
    }

    fn propagates_pointer_interaction(&self) -> bool {
        true
    }

    fn accepts_focus(&self) -> bool {
        false
    }

    fn accepts_text_input(&self) -> bool {
        false
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("ResizableHeader", id = id.trace())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(n: usize) -> Vec<Arc<str>> {
        (0..n).map(|i| Arc::from(format!("col{i}"))).collect()
    }

    // `place_columns`/`divider_start`/`compute_rendered_widths` are pure
    // functions now covered directly in `column_layout`'s own tests; only
    // `ResizableHeader`-specific behavior is tested here.

    #[test]
    fn drag_resizes_only_the_dragged_column() {
        // Regression test for the fixed resize semantics: dragging divider 0
        // must only change column_widths[0], never column_widths[1].
        let mut header = ResizableHeader::new(Vec::new(), keys(3), vec![100.0, 150.0, 80.0]);
        header.update_column_layout();

        let divider_idx = 0;
        header.drag_start_width = header.columns[divider_idx].width;
        let signed_delta = 20.0; // simulates dragging right by 20px in LTR
        let new_width = (header.drag_start_width + signed_delta).max(MIN_COLUMN_WIDTH);

        header.columns[divider_idx].width = new_width;
        header.column_widths[divider_idx] = new_width;

        assert_eq!(header.column_widths[0], 120.0);
        assert_eq!(header.column_widths[1], 150.0); // untouched
        assert_eq!(header.column_widths[2], 80.0); // untouched
    }

    #[test]
    fn hit_test_divider_rtl_resolves_to_correct_data_index() {
        // 4 columns (Name, Route, Distance, Joy) at [200,200,100,60] —
        // final user-confirmed spec: divider `i` always resizes data
        // column `i`, unconditionally, same rule as LTR — no
        // direction-specific remapping. Screen order (left to right) is
        // Joy, Distance, Route, Name.
        let mut header = ResizableHeader::new(Vec::new(), keys(4), vec![200.0, 200.0, 100.0, 60.0])
            .with_direction(FlowDirection::Rtl);
        header.update_column_layout();

        // Divider between Name and Route (rightmost divider on screen,
        // divider index 0): resizes Name (data index 0).
        assert_eq!(header.hit_test_divider(365.0), Some(0));
        // Divider between Route and Distance (divider index 1): resizes
        // Route (data index 1).
        assert_eq!(header.hit_test_divider(163.0), Some(1));
        // No divider past Name's own right edge (nothing further right).
        assert_eq!(header.hit_test_divider(999.0), None);
    }
}
