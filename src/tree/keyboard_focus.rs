//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! `keyboard_focus` — wrap any xilem view in a transparent keyboard handler.
//!
//! ## What this is
//!
//! A xilem `View` plus a tiny masonry widget (`KeyHandler`) that sits
//! transparently around a child view. It accepts focus, captures keyboard
//! events (arrow keys, Home, End, PageUp/PageDown, Enter, Space, F2,
//! Escape), and emits semantic [`KeyAction`]s — nothing else. It does not
//! paint, does not manage scroll, does not draw focus rings, does not own
//! selection state. Hover/selection/focus visualization is the row
//! builder's responsibility.
//!
//! ## Why this exists (xilem upstream gap)
//!
//! There is no stock xilem view that captures global keyboard events on
//! behalf of an arbitrary child without simultaneously owning paint,
//! layout, and focus visualization (for example
//! [`xilem::view::sized_box`] is transparent but ignores keys; the
//! built-in interactive views like `text_input` or `button` consume keys
//! they think are theirs). Tree-style navigation (Up/Down/Left/Right
//! across a flex_col of rows) needs *outer* keyboard capture: the rows
//! stay clickable and stylable independently. We therefore implement the
//! masonry `Widget` trait directly so we can override `on_text_event` and
//! emit our own action type, while still forwarding layout/paint/measure
//! to the child unchanged.
//!
//! ## Smallest upstream change that would let us delete this file
//!
//! A xilem helper view of the shape
//! `keys(child).on_key(|state, key_event| Action)` — i.e. a transparent
//! wrapper that exposes a callback for `TextEvent::Keyboard` events
//! addressed to either the wrapper or any descendant — would replace this
//! file with one expression. Equivalently, masonry could expose a
//! "key listener" widget that does the same. As long as the wrapper is
//! type-preserving (so child rebuilds work), the precise API shape does
//! not matter. See the proposal section in
//! `~/.claude/plans/inherited-fluttering-wolf.md`.
//!
//! ## Implementation note
//!
//! The masonry widget stores `WidgetPod<dyn Widget>` (same pattern as
//! masonry's `SizedBox`). The xilem `View` wrapper preserves the inner
//! view's concrete widget type via `WidgetMut::downcast`, so child
//! rebuilds flow through xilem's normal lifecycle — no `replace_child`
//! workaround.

use std::any::TypeId;
use std::marker::PhantomData;
use tracing::{trace_span, Span};

use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, EventCtx, LayoutCtx, MeasureCtx, NewWidget, PaintCtx,
    PropertiesMut, PropertiesRef, RegisterCtx, TextEvent, Update, UpdateCtx, Widget, WidgetId,
    WidgetMut, WidgetPod,
    keyboard::{Key, NamedKey},
};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Axis, Point, Size};
use xilem::masonry::layout::LenReq;

use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewMarker};
use xilem::{Pod, ViewCtx, WidgetView};

/// Semantic key actions emitted by [`KeyHandler`].
///
/// Maps platform key events to tree-navigation intent. The view consumer
/// decides what each action means (move focus, toggle, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    /// Enter pressed.
    Activate,
    /// Space pressed.
    Toggle,
    /// F2 pressed.
    Edit,
    /// Escape pressed. Used to cancel inline edits.
    Cancel,
}

// =============================================================================
// MARK: Masonry widget
// =============================================================================

/// A transparent wrapper widget that captures keyboard events and emits
/// [`KeyAction`]s. Forwards layout, paint, and measure to its single child
/// unchanged.
///
/// Stores its child as `WidgetPod<dyn Widget>`. The xilem `View` layer uses
/// `WidgetMut::downcast` to recover the concrete child type, so rebuild
/// flows through xilem normally.
pub struct KeyHandler {
    child: WidgetPod<dyn Widget>,
}

impl KeyHandler {
    pub fn new(child: NewWidget<impl Widget + ?Sized>) -> Self {
        Self {
            child: child.erased().to_pod(),
        }
    }

    /// Mutable access to the child for the xilem `View` layer to drive
    /// rebuild via `downcast`.
    pub fn child_mut<'t>(this: &'t mut WidgetMut<'_, Self>) -> WidgetMut<'t, dyn Widget> {
        this.ctx.get_mut(&mut this.widget.child)
    }
}

impl Widget for KeyHandler {
    type Action = KeyAction;

    fn on_text_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &TextEvent,
    ) {
        let TextEvent::Keyboard(key_event) = event else {
            return;
        };
        if key_event.state.is_up() {
            return;
        }

        let action = match &key_event.key {
            Key::Named(NamedKey::ArrowUp) => Some(KeyAction::Up),
            Key::Named(NamedKey::ArrowDown) => Some(KeyAction::Down),
            Key::Named(NamedKey::ArrowLeft) => Some(KeyAction::Left),
            Key::Named(NamedKey::ArrowRight) => Some(KeyAction::Right),
            Key::Named(NamedKey::Home) => Some(KeyAction::Home),
            Key::Named(NamedKey::End) => Some(KeyAction::End),
            Key::Named(NamedKey::PageUp) => Some(KeyAction::PageUp),
            Key::Named(NamedKey::PageDown) => Some(KeyAction::PageDown),
            Key::Named(NamedKey::Enter) => Some(KeyAction::Activate),
            Key::Named(NamedKey::F2) => Some(KeyAction::Edit),
            Key::Named(NamedKey::Escape) => Some(KeyAction::Cancel),
            Key::Character(c) if c.as_str() == " " => Some(KeyAction::Toggle),
            _ => None,
        };

        if let Some(action) = action {
            ctx.submit_action::<KeyAction>(action);
            ctx.set_handled();
        }
    }

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &xilem::masonry::core::PointerEvent,
    ) {
        // Take focus on pointer down inside us, so subsequent keys land here.
        if let xilem::masonry::core::PointerEvent::Down(_) = event {
            // Take focus on click *unless* a focus-aware descendant
            // already grabbed it (e.g., a `text_input` rendered for inline
            // rename). `has_focus_target()` returns true if this widget OR
            // any descendant currently holds focus; in that case we leave
            // it alone so descendants don't lose focus to us via the
            // "last request_focus wins" rule. When nothing in our subtree
            // is focused yet — which is the common case for clicks on the
            // chevron or row body — we take focus so subsequent arrow
            // keys route to us.
            if !ctx.has_focus_target() {
                ctx.request_focus();
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

    fn update(&mut self, _ctx: &mut UpdateCtx<'_>, _props: &mut PropertiesMut<'_>, _event: &Update) {}

    fn property_changed(&mut self, _ctx: &mut UpdateCtx<'_>, _property_type: TypeId) {}

    fn register_children(&mut self, ctx: &mut RegisterCtx<'_>) {
        ctx.register_child(&mut self.child);
    }

    fn measure(
        &mut self,
        ctx: &mut MeasureCtx<'_>,
        _props: &PropertiesRef<'_>,
        axis: Axis,
        len_req: LenReq,
        cross_length: Option<f64>,
    ) -> f64 {
        let auto_length = len_req.into();
        let context_size = xilem::masonry::layout::LayoutSize::maybe(axis.cross(), cross_length);
        ctx.compute_length(&mut self.child, auto_length, context_size, axis, cross_length)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, size: Size) {
        ctx.run_layout(&mut self.child, size);
        ctx.place_child(&mut self.child, Point::ORIGIN);
        ctx.derive_baselines(&mut self.child);
    }

    fn paint(
        &mut self,
        _ctx: &mut PaintCtx<'_>,
        _props: &PropertiesRef<'_>,
        _painter: &mut Painter<'_>,
    ) {
        // Transparent — child paints itself via masonry's own traversal.
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
        ChildrenIds::from_vec(vec![self.child.id()])
    }

    fn accepts_focus(&self) -> bool {
        true
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("KeyHandler", id = id.trace())
    }
}

// =============================================================================
// MARK: Xilem view
// =============================================================================

/// View created by [`keyboard_focus`].
#[must_use = "View values do nothing unless provided to Xilem."]
pub struct KeyboardFocus<V, F, State, Action = ()> {
    inner: V,
    on_action: F,
    phantom: PhantomData<fn() -> (State, Action)>,
}

impl<V, F, State, Action> ViewMarker for KeyboardFocus<V, F, State, Action> {}

impl<V, F, State, Action> View<State, Action, ViewCtx> for KeyboardFocus<V, F, State, Action>
where
    State: 'static,
    Action: 'static,
    V: WidgetView<State, Action>,
    F: Fn(&mut State, KeyAction) -> MessageResult<Action> + Send + Sync + 'static,
{
    type Element = Pod<KeyHandler>;
    type ViewState = V::ViewState;

    fn build(&self, ctx: &mut ViewCtx, app_state: &mut State) -> (Self::Element, Self::ViewState) {
        let (child, child_state) = self.inner.build(ctx, app_state);
        let widget = KeyHandler::new(child.new_widget);
        let pod = ctx.create_pod(widget);
        ctx.record_action_source(pod.new_widget.id());
        (pod, child_state)
    }

    fn rebuild(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) {
        // Recover the concrete child widget type and forward rebuild to the
        // inner view — this is the entire point of the `keyboard_focus`
        // wrapper existing as a typed xilem view.
        let mut child = KeyHandler::child_mut(&mut element);
        self.inner
            .rebuild(&prev.inner, view_state, ctx, child.downcast(), app_state);
    }

    fn teardown(
        &self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
    ) {
        let mut child = KeyHandler::child_mut(&mut element);
        self.inner.teardown(view_state, ctx, child.downcast());
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        message: &mut MessageCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) -> MessageResult<Action> {
        // Messages with remaining path are for descendants — forward unchanged.
        // Only an empty path means the message is addressed to *this* view,
        // which for us means a `KeyAction` from `KeyHandler::on_text_event`.
        if !message.remaining_path().is_empty() {
            let mut child = KeyHandler::child_mut(&mut element);
            return self.inner.message(view_state, message, child.downcast(), app_state);
        }

        match message.take_message::<KeyAction>() {
            Some(action) => (self.on_action)(app_state, *action),
            None => {
                tracing::error!(?message, "Wrong message type in KeyboardFocus::message");
                MessageResult::Stale
            }
        }
    }
}

/// Wrap `inner` in a [`KeyHandler`] that emits [`KeyAction`]s on arrow keys,
/// Home/End, PageUp/PageDown, Enter, Space, and F2. Forwards layout and
/// paint to `inner` unchanged.
pub fn keyboard_focus<V, F, State, Action>(
    inner: V,
    on_action: F,
) -> KeyboardFocus<V, F, State, Action>
where
    State: 'static,
    Action: 'static,
    V: WidgetView<State, Action>,
    F: Fn(&mut State, KeyAction) -> MessageResult<Action> + Send + Sync + 'static,
{
    KeyboardFocus {
        inner,
        on_action,
        phantom: PhantomData,
    }
}
