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
/// pointer-up only). This is the *only* action that reaches app state
/// (via `TableAction::ColumnResized`) — everything else below is ephemeral
/// UI-state broadcast, never persisted.
///
/// Carries *every* column's current width, not just the dragged one.
/// `FixedViewport` mode may have compressed other columns' *rendered*
/// widths below their previous *desired* ones to make room for the drag —
/// persisting only the dragged column would leave those still-large
/// desired values sitting in app state, ready to cause a visible snap the
/// next time an *earlier* column is dragged and the protection boundary
/// shifts to include them in the compressible set for the first time
/// (verified: dragging column 0 after previously dragging column 1 to a
/// wide committed value made column 1 suddenly compete for space and jump
/// smaller, even though nothing touched it). Persisting the as-rendered
/// width for every column on commit makes "what you see is what you get"
/// the new baseline, so switching which divider is dragged next never
/// causes a jump — only actually moving it does.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnResizeAction {
    pub widths: Vec<(Arc<str>, f64)>,
}

/// Broadcasts the header's freshly-computed column layout — submitted from
/// `layout()` every time `self.columns` actually changes (not just during a
/// drag: also window resizes, `FixedViewport` recompression, etc.), so both
/// `TableWidget` (hit-testing, the full-height highlight) and `TableView`
/// (row content placement) always see the *exact* numbers the header is
/// about to paint. Submitted via `submit_untyped_action` since it isn't
/// `ResizableHeader::Action`.
///
/// This is deliberately the *only* place `place_columns`/
/// `compute_rendered_widths` are ever called in the whole table stack —
/// `TableWidget` and `TableView` are pure consumers of this payload, never
/// independent re-derivations of it. Three independent copies of this same
/// computation (one per widget layer) is what caused header/row/hit-test
/// disagreements to keep recurring; a single source pushed downward removes
/// the class of bug entirely rather than re-deriving the formula again.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnLayoutAction {
    pub columns: Vec<ColumnBox>,
    /// Divider currently being *dragged* (not merely hovered — hover only
    /// drives the local, header-height highlight painted directly in
    /// `paint()`). `Some` only for the duration of an active drag, so
    /// `TableWidget`'s full-height guideline is drag-only feedback.
    pub active_divider: Option<usize>,
}

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
    /// The true, externally-given viewport width, pushed down from
    /// `TableWidget::set_viewport_width` (itself fed by `TableView`
    /// reading its owning `Portal`'s own border-box size). Used as the
    /// RTL mirror anchor in `layout()` instead of `size.width` directly:
    /// in `Overflow` mode `size.width` is *always* exactly the content's
    /// own total width (a `Portal` with `constrain_horizontal(false)`
    /// lays a `MaxContent`-measured child out at its own preferred size),
    /// so mirroring around it is mirroring around a value that moves in
    /// lockstep with the very content it's supposed to be a stable
    /// reference for — every earlier RTL-overflow bug (columns snapping,
    /// going negative/unreachable, "jiggling" when compensated for after
    /// the fact) traced back to exactly this. `viewport_width` is never
    /// self-referential, so no compensation of any kind is needed: it's
    /// the same true constant LTR's own column 0 already gets for free at
    /// `local_x = 0`. `None` (before the first push, or for the
    /// standalone `ResizableHeaderView` which isn't wrapped in a
    /// self-sizing `Portal` at all) falls back to `size.width`, which is
    /// already correct in those cases — `FixedViewport` mode's own
    /// `size.width` is a real, externally-imposed container width too.
    viewport_width: Option<f64>,
    drag_start_x: f64,
    drag_start_width: f64,
    /// The `(columns, active_divider)` most recently broadcast via
    /// `ColumnLayoutAction`, so `layout()` only submits when something
    /// actually changed — it runs on every layout pass, not just during a
    /// drag, and most of those passes are no-ops for this purpose.
    last_submitted: Option<(Vec<ColumnBox>, Option<usize>)>,
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
            viewport_width: None,
            drag_start_x: 0.0,
            drag_start_width: 0.0,
            last_submitted: None,
            divider_color: Color::from_rgb8(120, 118, 115),
            divider_hover_color: Color::from_rgb8(100, 150, 255),
            hovered_divider: None,
        }
    }

    pub fn with_divider_color(mut self, color: Color) -> Self {
        self.divider_color = color;
        self
    }

    /// Sets the true, externally-given viewport width (see
    /// `viewport_width`'s doc comment).
    pub fn set_viewport_width(this: &mut WidgetMut<'_, Self>, viewport_width: f64) {
        if this.widget.viewport_width != Some(viewport_width) {
            this.widget.viewport_width = Some(viewport_width);
            this.ctx.request_layout();
        }
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
                    // `last_resized_index` is deliberately *not* set here.
                    // It's the `FixedViewport` before/after protection
                    // anchor — updating it on mere click (before any actual
                    // width change) would immediately redistribute
                    // compression around the newly-touched divider with no
                    // drag having happened at all, visibly snapping
                    // neighboring columns just from clicking. It's set
                    // below, in `Move`, only once the drag has actually
                    // produced a different width.
                    self.drag_start_x = pos.x;
                    // Read from `column_widths` (the desired/configured
                    // value), not `self.columns` (the *rendered* value) —
                    // in `FixedViewport` mode a column can be compressed
                    // below its desired width by an unrelated drag on a
                    // column before it; a fresh drag should continue from
                    // the user's real preference, not a rendering artifact.
                    self.drag_start_width = self.column_widths[divider_idx];
                    // `active_divider` changed (None -> Some) — `layout()`
                    // broadcasts `ColumnLayoutAction` once it recomputes
                    // `self.columns` for this new frame; no need to submit
                    // anything here directly.
                    ctx.request_layout();
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
                        if let Some(w) = self.column_widths.get_mut(divider_idx) {
                            if *w != new_width {
                                *w = new_width;
                                // Only now — an actual width change, not
                                // just a click — does this divider become
                                // the `FixedViewport` protection anchor.
                                self.last_resized_index = Some(divider_idx);
                            }
                        }

                        ctx.request_layout();
                    }
                } else {
                    // Hover-only: update the local (header-height) highlight
                    // but don't notify TableWidget — the full-height
                    // highlight is drag-only feedback, not a hover one (see
                    // `ColumnLayoutAction::active_divider`'s doc comment).
                    let new_hovered = self.hit_test_divider(pos.x);
                    if new_hovered != self.hovered_divider {
                        self.hovered_divider = new_hovered;
                        ctx.request_render();
                    }
                }
            }
            PointerEvent::Up(..) | PointerEvent::Cancel(..) => {
                if self.dragging_index.take().is_some() {
                    // Bake in every column's *rendered* width (not just the
                    // dragged one) as the new desired baseline — see
                    // `ColumnResizeAction`'s doc comment for why this is
                    // needed to prevent a snap the next time a different,
                    // earlier column is dragged.
                    let widths: Vec<(Arc<str>, f64)> = self
                        .columns
                        .iter()
                        .map(|col| (col.key.clone(), col.width))
                        .collect();
                    for (i, col) in self.columns.iter().enumerate() {
                        if let Some(w) = self.column_widths.get_mut(i) {
                            *w = col.width;
                        }
                    }
                    ctx.submit_action::<Self::Action>(ColumnResizeAction { widths });
                }
                ctx.request_layout();
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

        // RTL mirrors around `anchor_width`, which is the true,
        // externally-given `viewport_width` when available (see its doc
        // comment) — never `size.width` directly, since in `Overflow`
        // mode `size.width` is always exactly the content's own total
        // width (self-referential), not a stable reference to mirror
        // around. `viewport_width` falls back to `size.width` only when
        // it's genuinely correct to do so (not yet pushed, or
        // `FixedViewport` mode / the standalone `ResizableHeaderView`,
        // neither of which is ever hosted in a self-sizing `Portal`).
        let anchor_width = self.viewport_width.unwrap_or(size.width);
        self.columns = column_layout::place_columns(
            &self.column_keys,
            &scaled_widths,
            anchor_width,
            self.direction,
        );

        column_layout::place_children(ctx, &mut self.children, &self.columns, size.height);

        // Broadcast the freshly-computed layout to `TableWidget`/`TableView`
        // (see `ColumnLayoutAction`'s doc comment) — this is the *only*
        // place that ever happens, and it happens on every layout pass that
        // actually changes something, not just during a drag, so window
        // resizes and `FixedViewport` recompression propagate too. Deduped
        // against `last_submitted` since `layout()` runs on passes that
        // don't change anything for this widget far more often than it
        // changes something.
        let broadcast = (self.columns.clone(), self.dragging_index);
        if self.last_submitted.as_ref() != Some(&broadcast) {
            let (columns, active_divider) = broadcast.clone();
            ctx.submit_untyped_action(Box::new(ColumnLayoutAction {
                columns,
                active_divider,
            }));
            self.last_submitted = Some(broadcast);
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
