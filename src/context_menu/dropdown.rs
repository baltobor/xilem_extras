//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Floating dropdown layer for context menus.

use tracing::{Span, trace_span};
use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Rect, RoundedRect, Stroke};
use xilem::masonry::peniko::Color;

use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, ComposeCtx, EventCtx, Layer, LayoutCtx, MeasureCtx,
    NewWidget, NoAction, PaintCtx, PointerButton, PointerButtonEvent, PointerEvent, PropertiesMut,
    PropertiesRef, RegisterCtx, TextEvent, Update, UpdateCtx, Widget, WidgetId, WidgetPod,
};
use xilem::masonry::kurbo::{Axis, Point, Size};
use xilem::masonry::layout::{LayoutSize, LenReq, SizeDef};
use xilem::masonry::properties::Gap;

use super::widget::{ContextMenuAction, ContextMenuWidget};

/// Default dropdown background color.
const BG_COLOR: Color = Color::from_rgba8(0x38, 0x36, 0x34, 0xF8);
/// Default dropdown border color.
const BORDER_COLOR: Color = Color::from_rgba8(0x50, 0x4E, 0x4A, 0xFF);

/// A floating dropdown layer for context menus.
///
/// Implements [`Layer`] to capture pointer events and dismiss on
/// click-outside. When an item is clicked, it submits a [`ContextMenuAction`]
/// on behalf of the creator widget and removes itself.
pub struct ContextMenuDropdown {
    creator: WidgetId,
    children: Vec<WidgetPod<dyn Widget>>,
    bg_color: Color,
    border_color: Color,
}

impl ContextMenuDropdown {
    /// Creates a new empty context menu dropdown.
    pub fn new(creator: WidgetId) -> Self {
        Self {
            creator,
            children: Vec::new(),
            bg_color: BG_COLOR,
            border_color: BORDER_COLOR,
        }
    }

    /// Builder-style method to add a menu item widget.
    pub fn with_item(mut self, child: NewWidget<impl Widget + ?Sized>) -> Self {
        self.children.push(child.erased().to_pod());
        self
    }

    /// Sets the background color.
    pub fn with_bg_color(mut self, color: Color) -> Self {
        self.bg_color = color;
        self
    }

    /// Sets the border color.
    pub fn with_border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }
}


impl Widget for ContextMenuDropdown {
    type Action = NoAction;

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        if let PointerEvent::Up(PointerButtonEvent {
            button: None | Some(PointerButton::Primary),
            ..
        }) = event
        {
            let self_id = ctx.widget_id();
            let clicked_id = ctx.target();

            let index = self
                .children
                .iter()
                .position(|child| child.id() == clicked_id);

            if let Some(index) = index {
                ctx.mutate_later(self.creator, move |mut ctx_menu| {
                    let mut ctx_menu = ctx_menu.downcast::<ContextMenuWidget>();
                    ctx_menu
                        .ctx
                        .submit_action::<ContextMenuAction>(ContextMenuAction::ItemSelected(index));
                    ctx_menu.ctx.remove_layer(self_id);
                    ctx_menu.widget.menu_layer_id = None;
                });
            }
        }
    }

    fn on_text_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &TextEvent,
    ) {
        if let TextEvent::WindowFocusChange(false) = event {
            ctx.remove_layer(ctx.widget_id());
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
        if let Update::WidgetAdded = event {
            let id = ctx.widget_id();
            ctx.mutate_later(self.creator, move |mut ctx_menu| {
                let ctx_menu = ctx_menu.downcast::<ContextMenuWidget>();
                ctx_menu.widget.menu_layer_id = Some(id);
            });
        }
    }

    fn register_children(&mut self, ctx: &mut RegisterCtx<'_>) {
        for child in &mut self.children {
            ctx.register_child(child);
        }
    }

    fn property_changed(&mut self, _ctx: &mut UpdateCtx<'_>, _property_type: std::any::TypeId) {}

    fn measure(
        &mut self,
        ctx: &mut MeasureCtx<'_>,
        props: &PropertiesRef<'_>,
        axis: Axis,
        len_req: LenReq,
        cross_length: Option<f64>,
    ) -> f64 {
        let scale = 1.0;
        let gap = props.get::<Gap>(ctx.property_cache());
        let gap_length = gap.gap.dp(scale);

        let (len_req, min_result) = match len_req {
            LenReq::MinContent | LenReq::MaxContent => (len_req, 0.),
            LenReq::FitContent(space) => (LenReq::MinContent, space),
        };

        let auto_length = len_req.into();
        let context_size = LayoutSize::maybe(axis.cross(), cross_length);

        let mut length: f64 = 0.;
        for child in &mut self.children {
            let child_length =
                ctx.compute_length(child, auto_length, context_size, axis, cross_length);
            match axis {
                Axis::Horizontal => length = length.max(child_length),
                Axis::Vertical => length += child_length,
            }
        }

        if axis == Axis::Vertical && !self.children.is_empty() {
            let gap_count = (self.children.len() - 1) as f64;
            length += gap_count * gap_length;
        }

        min_result.max(length)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, props: &PropertiesRef<'_>, size: Size) {
        let scale = 1.0;
        let gap = props.get::<Gap>(ctx.property_cache());
        let gap_length = gap.gap.dp(scale);

        let width_def = xilem::masonry::layout::LenDef::FitContent(size.width);
        let height_def = xilem::masonry::layout::LenDef::FitContent(size.height);
        let auto_size = SizeDef::new(width_def, height_def);
        let context_size = size.into();

        let mut y_offset = 0.0;
        for child in &mut self.children {
            let child_size = ctx.compute_size(child, auto_size, context_size);
            ctx.run_layout(child, child_size);
            ctx.place_child(child, Point::new(0.0, y_offset));
            y_offset += child_size.height + gap_length;
        }

        if !self.children.is_empty() {
            let first_child = self.children.first().unwrap();
            let (first_baseline, _) = ctx.child_aligned_baselines(first_child);
            let first_child_origin = ctx.child_origin(first_child);
            let first_baseline = first_child_origin.y + first_baseline;

            let last_child = self.children.last().unwrap();
            let (_, last_baseline) = ctx.child_aligned_baselines(last_child);
            let last_child_origin = ctx.child_origin(last_child);
            let last_baseline = last_child_origin.y + last_baseline;

            ctx.set_baselines(first_baseline, last_baseline);
        } else {
            ctx.clear_baselines();
        }
    }

    fn compose(&mut self, _ctx: &mut ComposeCtx<'_>) {}

    fn paint(&mut self, ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, painter: &mut Painter<'_>) {
        let size = ctx.border_box_size();
        let padding = 6.0;
        let rect = Rect::new(
            -padding,
            -padding / 2.0,
            size.width + padding,
            size.height + padding / 2.0,
        );
        let rounded = RoundedRect::from_rect(rect, 4.0);

        painter.fill(rounded, self.bg_color).draw();
        painter.stroke(rounded, &Stroke::new(1.0), self.border_color).draw();
    }

    fn accessibility_role(&self) -> Role {
        Role::Menu
    }

    fn accessibility(&mut self, _ctx: &mut AccessCtx<'_>, _props: &PropertiesRef<'_>, node: &mut Node) {
        node.set_label("Context menu");
    }

    fn children_ids(&self) -> ChildrenIds {
        self.children.iter().map(|child| child.id()).collect()
    }

    fn as_layer(&mut self) -> Option<&mut dyn Layer> {
        Some(self)
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("ContextMenuDropdown", id = id.trace())
    }
}

impl Layer for ContextMenuDropdown {
    fn capture_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        let dismiss = match event {
            PointerEvent::Down(PointerButtonEvent { state, .. }) => {
                let local_pos = ctx.local_position(state.position);
                !ctx.border_box_size().to_rect().contains(local_pos)
            }
            PointerEvent::Cancel(..) => true,
            _ => false,
        };

        if dismiss {
            ctx.remove_layer(ctx.widget_id());
            ctx.mutate_later(self.creator, move |mut ctx_menu| {
                let ctx_menu = ctx_menu.downcast::<ContextMenuWidget>();
                ctx_menu.widget.menu_layer_id = None;
            });
        }
    }
}
