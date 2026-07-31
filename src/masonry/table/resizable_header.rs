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

const DIVIDER_HIT_AREA: f64 = 8.0;
pub(crate) const MIN_COLUMN_WIDTH: f64 = 40.0;
pub(crate) const DIVIDER_WIDTH: f64 = 2.0;

/// Action emitted when a column is resized.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnResizeAction {
    pub column_key: Arc<str>,
    pub new_width: f64,
}

#[derive(Debug, Clone)]
struct ColumnInfo {
    key: Arc<str>,
    width: f64,
    x_offset: f64,
}

/// A header row widget with draggable column dividers.
pub struct ResizableHeader {
    pub(crate) children: Vec<WidgetPod<dyn Widget>>,
    pub(crate) column_keys: Vec<Arc<str>>,
    pub(crate) column_widths: Vec<f64>,
    pub(crate) columns: Vec<ColumnInfo>,
    size: Size,
    dragging_index: Option<usize>,
    drag_start_x: f64,
    drag_start_width: f64,
    drag_start_adjacent_width: f64,
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
            size: Size::ZERO,
            dragging_index: None,
            drag_start_x: 0.0,
            drag_start_width: 0.0,
            drag_start_adjacent_width: 0.0,
            divider_color: Color::from_rgb8(120, 118, 115),
            divider_hover_color: Color::from_rgb8(100, 150, 255),
            hovered_divider: None,
        }
    }

    pub fn with_divider_color(mut self, color: Color) -> Self {
        self.divider_color = color;
        self
    }

    fn hit_test_divider(&self, x: f64) -> Option<usize> {
        for (i, col) in self.columns.iter().enumerate() {
            if i < self.columns.len() - 1 {
                let divider_start = col.x_offset + col.width;
                let divider_center = divider_start + DIVIDER_WIDTH / 2.0;
                if (x - divider_center).abs() <= DIVIDER_HIT_AREA {
                    return Some(i);
                }
            }
        }
        None
    }

    pub(crate) fn update_column_layout(&mut self) {
        self.columns.clear();
        let mut x = 0.0;
        for (i, key) in self.column_keys.iter().enumerate() {
            let width = self.column_widths.get(i).copied().unwrap_or(100.0);
            self.columns.push(ColumnInfo {
                key: key.clone(),
                width,
                x_offset: x,
            });
            if i < self.column_keys.len() - 1 {
                x += width + DIVIDER_WIDTH;
            } else {
                x += width;
            }
        }
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
                    self.drag_start_x = pos.x;
                    self.drag_start_width = self.columns[divider_idx].width;
                    self.drag_start_adjacent_width = self
                        .columns
                        .get(divider_idx + 1)
                        .map(|c| c.width)
                        .unwrap_or(0.0);
                    ctx.request_render();
                }
            }
            PointerEvent::Move(PointerUpdate { current, .. }) => {
                let pos = ctx.local_position(current.position);

                if ctx.is_active() {
                    if let Some(divider_idx) = self.dragging_index {
                        let delta = pos.x - self.drag_start_x;

                        let new_left_width = (self.drag_start_width + delta).max(MIN_COLUMN_WIDTH);
                        let new_right_width =
                            (self.drag_start_adjacent_width - delta).max(MIN_COLUMN_WIDTH);

                        let actual_left_delta = new_left_width - self.drag_start_width;
                        let actual_right_delta = self.drag_start_adjacent_width - new_right_width;

                        let clamped_delta = if actual_left_delta.abs() < actual_right_delta.abs() {
                            actual_left_delta
                        } else {
                            actual_right_delta
                        };

                        let final_left_width = self.drag_start_width + clamped_delta;
                        let final_right_width = self.drag_start_adjacent_width - clamped_delta;

                        if let Some(col) = self.columns.get_mut(divider_idx) {
                            col.width = final_left_width;
                        }
                        if let Some(w) = self.column_widths.get_mut(divider_idx) {
                            *w = final_left_width;
                        }

                        if let Some(col) = self.columns.get_mut(divider_idx + 1) {
                            col.width = final_right_width;
                        }
                        if let Some(w) = self.column_widths.get_mut(divider_idx + 1) {
                            *w = final_right_width;
                        }

                        ctx.request_layout();
                    }
                } else {
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
                    if let Some(col) = self.columns.get(divider_idx + 1) {
                        ctx.submit_action::<Self::Action>(ColumnResizeAction {
                            column_key: col.key.clone(),
                            new_width: col.width,
                        });
                    }
                }
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
            Update::HoveredChanged(_) | Update::ActiveChanged(_) => {
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

        let divider_count = self.column_widths.len().saturating_sub(1);
        let divider_space = divider_count as f64 * DIVIDER_WIDTH;
        let configured_total: f64 = self.column_widths.iter().sum();
        let configured_with_dividers = configured_total + divider_space;
        let available_width = size.width;
        let use_configured = available_width + 0.5 >= configured_with_dividers;
        let scale = if !use_configured && configured_total > 0.0 {
            ((available_width - divider_space) / configured_total).min(1.0)
        } else {
            1.0
        };

        self.columns.clear();
        let mut x = 0.0;
        for (i, key) in self.column_keys.iter().enumerate() {
            let base_width = self.column_widths.get(i).copied().unwrap_or(100.0);
            let column_width = if use_configured {
                base_width.max(MIN_COLUMN_WIDTH)
            } else {
                (base_width * scale).max(MIN_COLUMN_WIDTH)
            };
            self.columns.push(ColumnInfo {
                key: key.clone(),
                width: column_width,
                x_offset: x,
            });
            if i < self.column_keys.len() - 1 {
                x += column_width + DIVIDER_WIDTH;
            } else {
                x += column_width;
            }
        }

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
                    let divider_rect = Rect::new(
                        col.x_offset + col.width,
                        0.0,
                        col.x_offset + col.width + DIVIDER_WIDTH,
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
