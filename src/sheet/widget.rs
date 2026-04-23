//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Sheet widget that triggers a modal layer.

use std::any::TypeId;

use tracing::{Span, trace_span};
use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::imaging::Painter;
use xilem::masonry::peniko::Color;

use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, EventCtx, LayerType, LayoutCtx, MeasureCtx, NewWidget,
    PaintCtx, PointerEvent, PropertiesMut, PropertiesRef, RegisterCtx, TextEvent, Update,
    UpdateCtx, Widget, WidgetId, WidgetMut, WidgetPod,
};
use xilem::masonry::kurbo::{Axis, Point, Size};
use xilem::masonry::layout::{LayoutSize, LenReq, SizeDef};

use super::SheetLayer;

/// Action emitted when the sheet is dismissed.
#[derive(PartialEq, Debug, Clone)]
pub enum SheetAction {
    /// The sheet was dismissed (via backdrop click or ESC).
    Dismissed,
}

/// A widget that displays content in a modal sheet/overlay.
///
/// The sheet appears as a floating layer with a semi-transparent backdrop.
/// Clicking the backdrop or pressing ESC dismisses the sheet.
pub struct SheetWidget {
    /// The content to display in the sheet.
    child: WidgetPod<dyn Widget>,
    /// Whether the sheet is currently visible.
    is_presented: bool,
    /// Tracked layer ID so we can toggle/remove it.
    pub(crate) sheet_layer_id: Option<WidgetId>,
    /// Background color for the sheet content area.
    bg_color: Color,
    /// Corner radius for the sheet content area.
    corner_radius: f64,
    /// Padding around the content.
    padding: f64,
}

impl SheetWidget {
    /// Creates a new sheet widget.
    ///
    /// - `child`: the content widget to display in the sheet.
    /// - `is_presented`: whether the sheet should be initially visible.
    pub fn new(child: NewWidget<impl Widget + ?Sized>, is_presented: bool) -> Self {
        Self {
            child: child.erased().to_pod(),
            is_presented,
            sheet_layer_id: None,
            bg_color: Color::WHITE,
            corner_radius: 12.0,
            padding: 24.0,
        }
    }

    /// Sets the background color of the sheet content area.
    pub fn with_bg_color(mut self, color: Color) -> Self {
        self.bg_color = color;
        self
    }

    /// Sets the corner radius of the sheet content area.
    pub fn with_corner_radius(mut self, radius: f64) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Sets the padding around the content.
    pub fn with_padding(mut self, padding: f64) -> Self {
        self.padding = padding;
        self
    }

    #[allow(dead_code)]
    fn show_sheet(&mut self, ctx: &mut EventCtx<'_>) {
        if self.sheet_layer_id.is_some() {
            return; // Already showing
        }

        let layer = SheetLayer::new(
            ctx.widget_id(),
            self.bg_color,
            self.corner_radius,
            self.padding,
        );

        // Create layer at origin (0, 0) since it covers full window
        ctx.create_layer(LayerType::Other, NewWidget::new(layer), Point::ORIGIN);
    }

    #[allow(dead_code)]
    fn hide_sheet(&mut self, ctx: &mut EventCtx<'_>) {
        if let Some(layer_id) = self.sheet_layer_id.take() {
            ctx.remove_layer(layer_id);
        }
    }
}

impl SheetWidget {
    /// Returns a mutable reference to the child widget.
    pub fn child_mut<'t>(this: &'t mut WidgetMut<'_, Self>) -> WidgetMut<'t, dyn Widget> {
        this.ctx.get_mut(&mut this.widget.child)
    }

    /// Sets whether the sheet is presented.
    pub fn set_presented(this: &mut WidgetMut<'_, Self>, presented: bool) {
        if this.widget.is_presented != presented {
            this.widget.is_presented = presented;
            this.ctx.request_layout();
        }
    }
}

impl Widget for SheetWidget {
    type Action = SheetAction;

    fn on_pointer_event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &PointerEvent,
    ) {
        // The sheet itself doesn't handle pointer events - the layer does
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
            Update::WidgetAdded => {
                // Show sheet on initial add if presented
                if self.is_presented {
                    ctx.request_layout();
                }
            }
            _ => {}
        }
    }

    fn register_children(&mut self, ctx: &mut RegisterCtx<'_>) {
        ctx.register_child(&mut self.child);
    }

    fn property_changed(&mut self, _ctx: &mut UpdateCtx<'_>, _property_type: TypeId) {}

    fn measure(
        &mut self,
        ctx: &mut MeasureCtx<'_>,
        _props: &PropertiesRef<'_>,
        axis: Axis,
        len_req: LenReq,
        cross_length: Option<f64>,
    ) -> f64 {
        // The sheet widget itself is invisible - just measure child for layout purposes
        let auto_length = len_req.into();
        let context_size = LayoutSize::maybe(axis.cross(), cross_length);
        ctx.compute_length(&mut self.child, auto_length, context_size, axis, cross_length)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, size: Size) {
        // Layout child (content reference - the actual content is in the layer)
        let child_size = ctx.compute_size(&mut self.child, SizeDef::fit(size), size.into());
        ctx.run_layout(&mut self.child, child_size);
        ctx.place_child(&mut self.child, Point::ORIGIN);
        ctx.derive_baselines(&self.child);

        // Trigger layer show/hide based on is_presented
        // NOTE: This is handled via event context, not layout context
    }

    fn paint(
        &mut self,
        _ctx: &mut PaintCtx<'_>,
        _props: &PropertiesRef<'_>,
        _painter: &mut Painter<'_>,
    ) {
        // The sheet widget itself is invisible - content is in the layer
    }

    fn accessibility_role(&self) -> Role {
        Role::Dialog
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        _node: &mut Node,
    ) {
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::from_slice(&[self.child.id()])
    }

    fn accepts_focus(&self) -> bool {
        false
    }

    fn accepts_text_input(&self) -> bool {
        false
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("SheetWidget", id = id.trace())
    }
}
