//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Resizable table header widget with draggable column dividers.

use std::any::TypeId;

use xilem::core::{MessageCtx, Mut, View, ViewMarker, ViewPathTracker, ViewId};
use xilem::core::MessageResult;
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Point, Rect, Size};
use xilem::masonry::peniko::Color;
use tracing::{Span, trace_span};

use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, CursorIcon, EventCtx, LayoutCtx, MeasureCtx, NewWidget,
    PaintCtx, PointerButtonEvent, PointerEvent, PointerUpdate, PropertiesMut, PropertiesRef,
    QueryCtx, RegisterCtx, TextEvent, Update, UpdateCtx, Widget, WidgetId, WidgetMut, WidgetPod,
};
use xilem::masonry::layout::{LenReq, LayoutSize};
use xilem::masonry::properties::Background;
use xilem::masonry::kurbo::Axis;
use xilem::{Pod, ViewCtx, WidgetView};
use xilem::masonry::accesskit::{Node, Role};

const DIVIDER_HIT_AREA: f64 = 8.0;
const MIN_COLUMN_WIDTH: f64 = 40.0;
const DIVIDER_WIDTH: f64 = 2.0;

/// Action emitted when a column is resized.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnResizeAction {
    pub column_key: String,
    pub new_width: f64,
}

/// Internal column info for layout.
#[derive(Debug, Clone)]
struct ColumnInfo {
    key: String,
    width: f64,
    x_offset: f64,
}

/// A header row widget with draggable column dividers.
///
/// This widget wraps column header children and adds resize handles
/// between them that can be dragged to resize columns.
pub struct ResizableHeader {
    children: Vec<WidgetPod<dyn Widget>>,
    column_keys: Vec<String>,
    column_widths: Vec<f64>,
    columns: Vec<ColumnInfo>,
    size: Size,
    dragging_index: Option<usize>,
    drag_start_x: f64,
    drag_start_width: f64,
    divider_color: Color,
    divider_hover_color: Color,
    hovered_divider: Option<usize>,
}

impl ResizableHeader {
    /// Creates a new resizable header with the given column children.
    pub fn new(
        children: Vec<NewWidget<dyn Widget>>,
        column_keys: Vec<String>,
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
            divider_color: Color::from_rgb8(120, 118, 115),
            divider_hover_color: Color::from_rgb8(100, 150, 255),
            hovered_divider: None,
        }
    }

    /// Sets the divider color.
    pub fn with_divider_color(mut self, color: Color) -> Self {
        self.divider_color = color;
        self
    }

    /// Returns the index of the divider at the given x position, if any.
    /// Dividers are located in the gap after each column (except the last).
    fn hit_test_divider(&self, x: f64) -> Option<usize> {
        for (i, col) in self.columns.iter().enumerate() {
            if i < self.columns.len() - 1 {
                let divider_start = col.x_offset + col.width;
                let divider_center = divider_start + DIVIDER_WIDTH / 2.0;
                // Hit test with some padding beyond the divider
                if (x - divider_center).abs() <= DIVIDER_HIT_AREA {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Updates column layout info based on current widths.
    /// Leaves gaps between columns for dividers.
    fn update_column_layout(&mut self) {
        self.columns.clear();
        let mut x = 0.0;
        for (i, key) in self.column_keys.iter().enumerate() {
            let width = self.column_widths.get(i).copied().unwrap_or(100.0);
            self.columns.push(ColumnInfo {
                key: key.clone(),
                width,
                x_offset: x,
            });
            // Add divider gap after each column except the last
            if i < self.column_keys.len() - 1 {
                x += width + DIVIDER_WIDTH;
            } else {
                x += width;
            }
        }
    }

    /// Updates column widths from external source.
    pub fn set_column_widths(this: &mut WidgetMut<'_, Self>, widths: Vec<f64>) {
        this.widget.column_widths = widths;
        this.widget.update_column_layout();
        this.ctx.request_layout();
    }

    /// Returns a mutable reference to a child widget.
    pub fn child_mut<'t>(this: &'t mut WidgetMut<'_, Self>, index: usize) -> Option<WidgetMut<'t, dyn Widget>> {
        this.widget.children.get_mut(index).map(|child| this.ctx.get_mut(child))
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
                    ctx.request_render();
                }
            }
            PointerEvent::Move(PointerUpdate { current, .. }) => {
                let pos = ctx.local_position(current.position);

                if ctx.is_active() {
                    if let Some(divider_idx) = self.dragging_index {
                        let delta = pos.x - self.drag_start_x;
                        let new_width = (self.drag_start_width + delta).max(MIN_COLUMN_WIDTH);

                        if let Some(col) = self.columns.get_mut(divider_idx) {
                            col.width = new_width;
                        }
                        if let Some(w) = self.column_widths.get_mut(divider_idx) {
                            *w = new_width;
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
        _cross_length: Option<f64>,
    ) -> f64 {
        self.update_column_layout();

        match axis {
            Axis::Horizontal => {
                self.columns.iter().map(|c| c.width).sum()
            }
            Axis::Vertical => {
                let mut max_height = 0.0f64;
                for (i, child) in self.children.iter_mut().enumerate() {
                    let col_width = self.columns.get(i).map(|c| c.width);
                    let height = ctx.compute_length(
                        child,
                        len_req.into(),
                        LayoutSize::maybe(Axis::Horizontal, col_width),
                        axis,
                        col_width,
                    );
                    max_height = max_height.max(height);
                }
                max_height
            }
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, size: Size) {
        self.size = size;
        self.update_column_layout();

        for (i, child) in self.children.iter_mut().enumerate() {
            if let Some(col) = self.columns.get(i) {
                let child_size = Size::new(col.width, size.height);
                ctx.run_layout(child, child_size);
                ctx.place_child(child, Point::new(col.x_offset, 0.0));
            }
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx<'_>, props: &PropertiesRef<'_>, painter: &mut Painter<'_>) {
        let rect = Rect::from_origin_size(Point::ZERO, self.size);

        {
            let cache = ctx.property_cache();
            let bg = props.get::<Background>(cache);
            let brush = bg.get_peniko_brush_for_rect(rect);
            painter.fill(rect, &brush).draw();
        }

        // Paint dividers only on hover
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

/// Xilem view for a resizable header row.
pub struct ResizableHeaderView<F, State, Action, V> {
    column_keys: Vec<String>,
    column_widths: Vec<f64>,
    children: Vec<V>,
    callback: F,
    _phantom: std::marker::PhantomData<(State, Action)>,
}

/// Creates a resizable header view.
///
/// # Arguments
///
/// * `columns` - Slice of (key, width) pairs defining columns
/// * `children` - Views for each column header
/// * `callback` - Function called when a column is resized
pub fn resizable_header<State: 'static, Action: 'static, V, F>(
    columns: &[(&str, f64)],
    children: Vec<V>,
    callback: F,
) -> ResizableHeaderView<F, State, Action, V>
where
    V: WidgetView<State, Action>,
    F: Fn(&mut State, String, f64) -> Action + Send + Sync + 'static,
{
    ResizableHeaderView {
        column_keys: columns.iter().map(|(k, _)| k.to_string()).collect(),
        column_widths: columns.iter().map(|(_, w)| *w).collect(),
        children,
        callback,
        _phantom: std::marker::PhantomData,
    }
}

impl<F, State, Action, V> ViewMarker for ResizableHeaderView<F, State, Action, V> {}

impl<F, State, Action, V> View<State, Action, ViewCtx> for ResizableHeaderView<F, State, Action, V>
where
    V: WidgetView<State, Action>,
    F: Fn(&mut State, String, f64) -> Action + Send + Sync + 'static,
    State: 'static,
    Action: 'static,
{
    type Element = Pod<ResizableHeader>;
    type ViewState = Vec<V::ViewState>;

    fn build(
        &self,
        ctx: &mut ViewCtx,
        app_state: &mut State,
    ) -> (Self::Element, Self::ViewState) {
        let mut child_pods = Vec::new();
        let mut child_states = Vec::new();

        for (i, child) in self.children.iter().enumerate() {
            let (pod, state) = ctx.with_id(ViewId::new(i as u64), |ctx| {
                child.build(ctx, app_state)
            });
            child_pods.push(pod.new_widget.erased());
            child_states.push(state);
        }

        let pod = ctx.with_action_widget(|ctx| {
            let widget = ResizableHeader::new(
                child_pods,
                self.column_keys.clone(),
                self.column_widths.clone(),
            );
            ctx.create_pod(widget)
        });

        (pod, child_states)
    }

    fn rebuild(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) {
        if prev.column_widths != self.column_widths {
            ResizableHeader::set_column_widths(&mut element, self.column_widths.clone());
        }

        for (i, (child, prev_child)) in self.children.iter().zip(prev.children.iter()).enumerate() {
            if let Some(state) = view_state.get_mut(i) {
                if let Some(mut child_element) = ResizableHeader::child_mut(&mut element, i) {
                    ctx.with_id(ViewId::new(i as u64), |ctx| {
                        child.rebuild(prev_child, state, ctx, child_element.downcast(), app_state);
                    });
                }
            }
        }
    }

    fn teardown(
        &self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
    ) {
        for (i, child) in self.children.iter().enumerate() {
            if let Some(state) = view_state.get_mut(i) {
                if let Some(mut child_element) = ResizableHeader::child_mut(&mut element, i) {
                    ctx.with_id(ViewId::new(i as u64), |ctx| {
                        child.teardown(state, ctx, child_element.downcast());
                    });
                }
            }
        }
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        message: &mut MessageCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) -> MessageResult<Action> {
        match message.take_first() {
            Some(id) => {
                let idx = id.routing_id() as usize;
                if let (Some(child), Some(state)) = (self.children.get(idx), view_state.get_mut(idx)) {
                    if let Some(mut child_element) = ResizableHeader::child_mut(&mut element, idx) {
                        return child.message(state, message, child_element.downcast(), app_state);
                    }
                }
                MessageResult::Stale
            }
            None => match message.take_message::<ColumnResizeAction>() {
                Some(action) => {
                    MessageResult::Action((self.callback)(app_state, action.column_key, action.new_width))
                }
                None => MessageResult::Stale,
            },
        }
    }
}
