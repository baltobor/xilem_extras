//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Row button widget for list/tree rows.

use std::any::TypeId;

use tracing::{Span, trace_span};
use xilem::masonry::accesskit::{self, Node, Role};
use xilem::masonry::core::PointerButton;
use xilem::masonry::core::keyboard::{Key, NamedKey};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Point, Rect, Size};
use xilem::masonry::peniko::Color;

use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, EventCtx, LayoutCtx, MeasureCtx, Modifiers, NewWidget,
    PaintCtx, PointerButtonEvent, PointerEvent, PropertiesMut, PropertiesRef, RegisterCtx,
    TextEvent, Update, UpdateCtx, Widget, WidgetId, WidgetMut, WidgetPod,
};
use xilem::masonry::kurbo::Axis;
use xilem::masonry::layout::{LayoutSize, LenReq, Length, SizeDef};
use xilem::masonry::properties::Background;

/// Action emitted when a row button is pressed.
#[derive(PartialEq, Debug, Clone)]
pub struct RowButtonPress {
    pub button: Option<PointerButton>,
    pub click_count: u8,
    pub modifiers: Modifiers,
    pub position: Point,
}

/// A button widget designed for list/tree rows.
///
/// Key features:
/// - Content is left-aligned (not centered)
/// - No minimum height - rows are compact
/// - Full-width hover/active background highlight
/// - Stretches to fill available width from parent
pub struct RowButton {
    child: WidgetPod<dyn Widget>,
    hover_bg: Color,
    click_count: u8,
    modifiers: Modifiers,
    position: Point,
    size: Size,
}

impl RowButton {
    pub fn new(child: NewWidget<impl Widget + ?Sized>) -> Self {
        Self {
            child: child.erased().to_pod(),
            hover_bg: Color::TRANSPARENT,
            click_count: 0,
            modifiers: Modifiers::default(),
            position: Point::ZERO,
            size: Size::ZERO,
        }
    }

    pub fn with_hover_bg(mut self, color: Color) -> Self {
        self.hover_bg = color;
        self
    }

    pub fn child_mut<'t>(this: &'t mut WidgetMut<'_, Self>) -> WidgetMut<'t, dyn Widget> {
        this.ctx.get_mut(&mut this.widget.child)
    }

    pub fn set_hover_bg(this: &mut WidgetMut<'_, Self>, color: Color) {
        this.widget.hover_bg = color;
        this.ctx.request_paint_only();
    }
}

impl Widget for RowButton {
    type Action = RowButtonPress;

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        match event {
            PointerEvent::Down(PointerButtonEvent { state, .. }) => {
                if ctx.is_handled() {
                    return;
                }
                self.click_count = state.count as u8;
                self.modifiers = state.modifiers;
                self.position = Point::new(state.position.x, state.position.y);
                ctx.capture_pointer();
                ctx.request_render();
            }
            PointerEvent::Up(PointerButtonEvent { button, .. }) => {
                if ctx.is_active() && ctx.is_hovered() {
                    ctx.submit_action::<Self::Action>(RowButtonPress {
                        button: *button,
                        click_count: self.click_count,
                        modifiers: self.modifiers,
                        position: self.position,
                    });
                }
                ctx.request_render();
            }
            _ => (),
        }
    }

    fn on_text_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &TextEvent,
    ) {
        match event {
            TextEvent::Keyboard(event) if event.state.is_up() => {
                if matches!(&event.key, Key::Character(c) if c == " ")
                    || event.key == Key::Named(NamedKey::Enter)
                {
                    ctx.submit_action::<Self::Action>(RowButtonPress {
                        button: None,
                        click_count: 1,
                        modifiers: event.modifiers,
                        position: Point::ZERO,
                    });
                }
            }
            _ => (),
        }
    }

    fn on_access_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &AccessEvent,
    ) {
        if event.action == accesskit::Action::Click {
            ctx.submit_action::<Self::Action>(RowButtonPress {
                button: None,
                click_count: 1,
                modifiers: Modifiers::default(),
                position: Point::ZERO,
            });
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx<'_>, _props: &mut PropertiesMut<'_>, event: &Update) {
        match event {
            Update::HoveredChanged(_)
            | Update::ActiveChanged(_)
            | Update::FocusChanged(_)
            | Update::DisabledChanged(_) => {
                ctx.request_render();
            }
            _ => {}
        }
    }

    fn register_children(&mut self, ctx: &mut RegisterCtx<'_>) {
        ctx.register_child(&mut self.child);
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
        cross_length: Option<Length>,
    ) -> Length {
        let auto_length = len_req.into();
        let context_size = LayoutSize::maybe(axis.cross(), cross_length);

        let child_length = ctx.compute_length(
            &mut self.child,
            auto_length,
            context_size,
            axis,
            cross_length,
        );

        match axis {
            Axis::Horizontal => {
                if let LenReq::FitContent(available) = len_req {
                    available
                } else {
                    child_length
                }
            }
            Axis::Vertical => child_length,
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, size: Size) {
        self.size = size;
        let child_size = ctx.compute_size(&mut self.child, SizeDef::fit(size), size.into());
        ctx.run_layout(&mut self.child, child_size);
        ctx.place_child(&mut self.child, Point::ORIGIN);
        ctx.derive_baselines(&self.child);
    }

    fn paint(
        &mut self,
        ctx: &mut PaintCtx<'_>,
        props: &PropertiesRef<'_>,
        painter: &mut Painter<'_>,
    ) {
        let rect = Rect::from_origin_size(Point::ZERO, self.size);
        let use_hover = ctx.is_hovered() && !ctx.is_disabled();

        if use_hover {
            if self.hover_bg != Color::TRANSPARENT {
                painter.fill(rect, self.hover_bg).draw();
            }
        } else {
            let cache = ctx.property_cache();
            let bg = props.get::<Background>(cache);
            let brush = bg.get_peniko_brush_for_rect(rect);
            painter.fill(rect, &brush).draw();
        }
    }

    fn accessibility_role(&self) -> Role {
        Role::Button
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        node: &mut Node,
    ) {
        node.add_action(accesskit::Action::Click);
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::from_slice(&[self.child.id()])
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
        trace_span!("RowButton", id = id.trace())
    }
}
