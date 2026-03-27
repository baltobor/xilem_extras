//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Context menu widget that opens a dropdown layer on right-click.

use std::any::TypeId;

use tracing::{Span, trace_span};
use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::imaging::Painter;

use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, EventCtx, LayerType, LayoutCtx, MeasureCtx, NewWidget,
    PaintCtx, PointerButton, PointerButtonEvent, PointerEvent, PropertiesMut, PropertiesRef,
    RegisterCtx, TextEvent, Update, UpdateCtx, Widget, WidgetId, WidgetMut, WidgetPod,
};
use xilem::masonry::kurbo::{Axis, Point, Size};
use xilem::masonry::layout::{LayoutSize, LenReq, SizeDef};

use super::ContextMenuDropdown;
use crate::menu_button::PulldownMenuItem;

/// Action emitted when a context menu item is selected.
#[derive(PartialEq, Debug, Clone)]
pub enum ContextMenuAction {
    /// A menu item at the given index was selected.
    ItemSelected(usize),
}

/// A widget that wraps content and shows a context menu on right-click.
///
/// Similar to [`MenuButton`](crate::menu_button::MenuButton) but triggers
/// on right-click (secondary button) instead of left-click.
pub struct ContextMenuWidget {
    /// The wrapped child content.
    child: WidgetPod<dyn Widget>,
    /// Labels for each context menu item.
    items: Vec<String>,
    /// Tracked layer id so we can toggle/remove it.
    pub(crate) menu_layer_id: Option<WidgetId>,
}

impl ContextMenuWidget {
    /// Creates a new context menu widget.
    ///
    /// - `child`: the content widget to wrap.
    /// - `items`: the list of context menu item labels.
    pub fn new(child: NewWidget<impl Widget + ?Sized>, items: Vec<String>) -> Self {
        Self {
            child: child.erased().to_pod(),
            items,
            menu_layer_id: None,
        }
    }

    fn show_context_menu(&mut self, ctx: &mut EventCtx<'_>, position: Point) {
        // Close existing menu if any
        if let Some(id) = self.menu_layer_id {
            ctx.remove_layer(id);
            self.menu_layer_id = None;
        }

        let mut menu = ContextMenuDropdown::new(ctx.widget_id());
        for label in &self.items {
            if label == "---" {
                menu = menu.with_item(NewWidget::new(crate::menu_button::MenuSeparator::new()));
            } else {
                menu = menu.with_item(NewWidget::new(PulldownMenuItem::new(label.clone())));
            }
        }

        // Position at cursor
        ctx.create_layer(LayerType::Other, NewWidget::new(menu), position);
    }
}

impl ContextMenuWidget {
    /// Returns a mutable reference to the child widget.
    pub fn child_mut<'t>(this: &'t mut WidgetMut<'_, Self>) -> WidgetMut<'t, dyn Widget> {
        this.ctx.get_mut(&mut this.widget.child)
    }

    /// Replaces the context menu item labels.
    pub fn set_items(this: &mut WidgetMut<'_, Self>, items: Vec<String>) {
        this.widget.items = items;
    }
}

impl Widget for ContextMenuWidget {
    type Action = ContextMenuAction;

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        match event {
            PointerEvent::Down(PointerButtonEvent {
                button: Some(PointerButton::Secondary),
                state,
                ..
            }) => {
                // Show context menu immediately on right-click down
                let position = Point::new(state.position.x, state.position.y);
                self.show_context_menu(ctx, position);
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
            Update::HoveredChanged(_) | Update::DisabledChanged(_) => {
                ctx.request_paint_only();
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
        let auto_length = len_req.into();
        let context_size = LayoutSize::maybe(axis.cross(), cross_length);

        ctx.compute_length(&mut self.child, auto_length, context_size, axis, cross_length)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, size: Size) {
        let child_size = ctx.compute_size(&mut self.child, SizeDef::fit(size), size.into());
        ctx.run_layout(&mut self.child, child_size);

        let child_origin = Point::ORIGIN;
        ctx.place_child(&mut self.child, child_origin);
        ctx.derive_baselines(&self.child);
    }

    fn paint(
        &mut self,
        _ctx: &mut PaintCtx<'_>,
        _props: &PropertiesRef<'_>,
        _painter: &mut Painter<'_>,
    ) {
    }

    fn accessibility_role(&self) -> Role {
        Role::GenericContainer
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
        trace_span!("ContextMenuWidget", id = id.trace())
    }
}
