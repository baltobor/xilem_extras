//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Menu button widget that opens a dropdown menu layer when clicked.

use std::any::TypeId;

use xilem::masonry::accesskit::{self, Node, Role};
use tracing::{Span, trace_span};
use xilem::masonry::imaging::Painter;

use xilem::masonry::core::keyboard::{Key, NamedKey};
use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, EventCtx, LayerType, LayoutCtx, MeasureCtx, NewWidget,
    PaintCtx, PointerButtonEvent, PointerEvent, PropertiesMut, PropertiesRef, RegisterCtx,
    TextEvent, Update, UpdateCtx, Widget, WidgetId, WidgetMut, WidgetPod,
};
use xilem::masonry::kurbo::{Axis, Size, Vec2};
use xilem::masonry::layout::{LayoutSize, LenReq, SizeDef};

use super::MenuDropdown;

/// Action emitted when a menu item inside a [`MenuButton`]'s dropdown is clicked.
#[derive(PartialEq, Debug)]
pub struct MenuButtonPress {
    /// Index of the selected menu item.
    pub index: usize,
}

/// A button widget that opens a dropdown menu layer when clicked.
///
/// This follows the same pattern as masonry's `Selector` widget: on click it
/// spawns a [`MenuDropdown`] layer positioned below itself. The dropdown
/// dismisses on click-outside (via the [`Layer`] trait) or when an item is
/// selected.
pub struct MenuButton {
    /// The label shown on the button (e.g. "File", "View").
    child: WidgetPod<dyn Widget>,
    /// Labels for each dropdown menu item.
    items: Vec<String>,
    /// Tracked layer id so we can toggle/remove it.
    pub(crate) menu_layer_id: Option<WidgetId>,
}

impl MenuButton {
    /// Creates a new menu button.
    ///
    /// - `child`: the always-visible label widget (e.g. a `Label`).
    /// - `items`: the list of dropdown menu item labels.
    pub fn new(child: NewWidget<impl Widget + ?Sized>, items: Vec<String>) -> Self {
        Self {
            child: child.erased().to_pod(),
            items,
            menu_layer_id: None,
        }
    }

    fn toggle_layer(&mut self, ctx: &mut EventCtx<'_>) {
        if let Some(id) = self.menu_layer_id {
            ctx.remove_layer(id);
            self.menu_layer_id = None;
            return;
        }

        let mut menu = MenuDropdown::new(ctx.widget_id());
        for label in &self.items {
            if label == "---" {
                menu = menu.with_item(NewWidget::new(super::MenuSeparator::new()));
            } else {
                menu = menu.with_item(NewWidget::new(super::PulldownMenuItem::new(label.clone())));
            }
        }

        ctx.create_layer(
            LayerType::Other,
            NewWidget::new(menu),
            ctx.window_origin() + Vec2::new(0., ctx.border_box_size().height),
        );
    }
}

impl MenuButton {
    /// Returns a mutable reference to the child label widget.
    pub fn child_mut<'t>(this: &'t mut WidgetMut<'_, Self>) -> WidgetMut<'t, dyn Widget> {
        this.ctx.get_mut(&mut this.widget.child)
    }

    /// Replaces the dropdown item labels.
    pub fn set_items(this: &mut WidgetMut<'_, Self>, items: Vec<String>) {
        this.widget.items = items;
    }
}

impl Widget for MenuButton {
    type Action = MenuButtonPress;

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        match event {
            PointerEvent::Down(..) => {
                if self.menu_layer_id.is_none() {
                    ctx.capture_pointer();
                }
            }
            PointerEvent::Up(PointerButtonEvent { .. }) => {
                if ctx.is_active() && ctx.is_hovered() {
                    self.toggle_layer(ctx);
                }
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
                    self.toggle_layer(ctx);
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
            self.toggle_layer(ctx);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx<'_>, _props: &mut PropertiesMut<'_>, event: &Update) {
        match event {
            Update::HoveredChanged(_)
            | Update::ActiveChanged(_)
            | Update::FocusChanged(_)
            | Update::DisabledChanged(_) => {
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

        let child_origin = ((size - child_size).to_vec2() * 0.5).to_point();
        ctx.place_child(&mut self.child, child_origin);
        ctx.derive_baselines(&self.child);
    }

    fn paint(&mut self, _ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, _painter: &mut Painter<'_>) {}

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

    fn accepts_focus(&self) -> bool {
        true
    }

    fn accepts_text_input(&self) -> bool {
        false
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("MenuButton", id = id.trace())
    }
}
