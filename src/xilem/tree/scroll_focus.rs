//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! `scroll_focus` — a scrollable region that auto-scrolls to a target rect on demand.
//!
//! ## What this is
//!
//! A xilem view that wraps any inner [`WidgetView`] in a masonry
//! [`Portal`](masonry::widgets::Portal) (the same widget xilem's stock
//! `portal()` view uses). On every rebuild, it compares the current
//! `target_y` against the previous one; when the value changes and is
//! `Some(y)`, it asks the `Portal` to scroll so that the row at `y` is
//! visible via [`Portal::pan_viewport_to`].
//!
//! This is what makes "press Down arrow → tree scrolls so the new
//! selection stays in view" work in [`super::tree_view_builder`].
//!
//! ## Why this exists (xilem upstream gap)
//!
//! Masonry's `Portal` widget already has the right method:
//! `Portal::pan_viewport_to(this: &mut WidgetMut<Self>, target: Rect)` —
//! see `masonry/src/widgets/portal.rs:383`. But xilem's `portal(child)`
//! view (`xilem_masonry/src/view/portal.rs`) is a thin wrapper that only
//! exposes the four constructor flags (constrain_horizontal /
//! constrain_vertical / must_fill) and provides no way to call the
//! underlying `WidgetMut` methods from rebuild. There is also no
//! generic xilem "controller" / "imperative escape hatch" view that
//! would let arbitrary code grab a `WidgetMut<Portal<_>>` during
//! rebuild. So we re-implement the [`View`] trait directly to get
//! mutable access ourselves.
//!
//! ## Smallest upstream change that would let us delete this file
//!
//! Either of:
//!
//! 1. Add `Portal::scroll_to(target: Option<Rect>)` (or `.scroll_to_on_change(...)`)
//!    to the xilem `portal(...)` view, exposing
//!    `widgets::Portal::pan_viewport_to`.
//! 2. Add a generic xilem helper `controller<V, F>(view, f)` where `f`
//!    runs once per rebuild with `WidgetMut<V::Widget>` so any imperative
//!    masonry method (scroll, focus, IME state, …) becomes reachable
//!    from xilem.
//!
//! Either lets the entire content of this file collapse to one or two
//! lines at the call site. See the proposal in
//! `~/.claude/plans/inherited-fluttering-wolf.md`.

use std::marker::PhantomData;

use masonry::widgets;
use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewMarker};
use xilem::masonry::kurbo::Rect;
use xilem::{Pod, ViewCtx, WidgetView};

/// Approximate height of one tree row, used to translate `selected_index`
/// into a pixel y-offset for [`pan_viewport_to`](widgets::Portal::pan_viewport_to).
pub const DEFAULT_ROW_HEIGHT_HINT: f64 = 24.0;

/// Construct a [`ScrollFocus`] wrapping `child`. When `target_y` changes
/// between rebuilds and is `Some`, the inner portal scrolls so that the
/// rectangle at `(0, target_y, child_width, target_y + row_height)` is
/// visible.
///
/// Defaults: both axes UNCONSTRAINED (`constrain_horizontal = false`,
/// `constrain_vertical = false`) — matching xilem's stock `portal()`
/// view. In masonry's `Portal` semantics, "constrained" actually means
/// "this axis cannot scroll" (the portal forces the child to fit, no
/// scrollbar, mouse-wheel disabled on that axis). The whole point of a
/// scrolling tree view is to *allow* the child to overflow; the parent
/// flex slot bounds the portal itself, and the portal then clips the
/// content and exposes a scrollbar.
///
/// If you want the portal to behave like a non-scrolling clip on one
/// axis (e.g. force a horizontal-only tree), opt in with
/// [`ScrollFocus::constrain_horizontal`] /
/// [`ScrollFocus::constrain_vertical`].
pub fn scroll_focus<Child, State, Action>(
    child: Child,
    target_y: Option<f64>,
    row_height: f64,
) -> ScrollFocus<Child, State, Action>
where
    State: 'static,
    Child: WidgetView<State, Action>,
{
    ScrollFocus {
        child,
        target_y,
        row_height,
        constrain_horizontal: false,
        constrain_vertical: false,
        content_must_fill: false,
        phantom: PhantomData,
    }
}

#[must_use = "View values do nothing unless provided to Xilem."]
pub struct ScrollFocus<V, State, Action> {
    child: V,
    target_y: Option<f64>,
    row_height: f64,
    constrain_horizontal: bool,
    constrain_vertical: bool,
    content_must_fill: bool,
    phantom: PhantomData<fn(State) -> Action>,
}

impl<V, State, Action> ScrollFocus<V, State, Action> {
    /// If `true`, the portal does NOT scroll horizontally — the child
    /// is forced to fit the portal's width and there is no horizontal
    /// scrollbar. Default `false` (the child overflows as wide as it
    /// wants and the portal scrolls horizontally to follow).
    /// Mirrors [`Portal::constrain_horizontal`](widgets::Portal::constrain_horizontal).
    pub fn constrain_horizontal(mut self, constrain: bool) -> Self {
        self.constrain_horizontal = constrain;
        self
    }

    /// If `true`, the portal does NOT scroll vertically — the child is
    /// forced to fit the portal's height and there is no vertical
    /// scrollbar. Default `false` (the child overflows as tall as it
    /// wants and the portal scrolls vertically to follow).
    /// Mirrors [`Portal::constrain_vertical`](widgets::Portal::constrain_vertical).
    pub fn constrain_vertical(mut self, constrain: bool) -> Self {
        self.constrain_vertical = constrain;
        self
    }

    /// If `true`, the child must fill the portal's viewport (mirrors
    /// [`Portal::content_must_fill`](widgets::Portal::content_must_fill)).
    /// Default is `false`, matching xilem's stock `portal()` view.
    pub fn content_must_fill(mut self, must_fill: bool) -> Self {
        self.content_must_fill = must_fill;
        self
    }
}

impl<V, State, Action> ViewMarker for ScrollFocus<V, State, Action> {}

impl<Child, State, Action> View<State, Action, ViewCtx> for ScrollFocus<Child, State, Action>
where
    Child: WidgetView<State, Action>,
    State: 'static,
    Action: 'static,
{
    type Element = Pod<widgets::Portal<Child::Widget>>;
    type ViewState = Child::ViewState;

    fn build(&self, ctx: &mut ViewCtx, app_state: &mut State) -> (Self::Element, Self::ViewState) {
        let (child, child_state) = self.child.build(ctx, app_state);
        // Honor the constraint flags from the builder (defaulting to
        // both axes constrained — see `scroll_focus`'s doc) so the
        // inner `Portal` sizes itself to whatever slot its parent
        // hands it. Without this, a tree taller than the slot would
        // push siblings off the screen instead of becoming
        // scrollable, which broke layouts like
        // `flex_col((header, tree, search_bar))`.
        let portal = widgets::Portal::new(child.new_widget)
            .constrain_horizontal(self.constrain_horizontal)
            .constrain_vertical(self.constrain_vertical)
            .content_must_fill(self.content_must_fill);
        let widget_pod = ctx.create_pod(portal);
        (widget_pod, child_state)
    }

    fn rebuild(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) {
        // Forward child rebuild first so the inner content reflects the
        // new state (selection, expansion, edits) BEFORE we ask the
        // portal to scroll. Otherwise we might pan to where the row used
        // to be in the previous layout.
        {
            let child_element = widgets::Portal::child_mut(&mut element);
            self.child
                .rebuild(&prev.child, view_state, ctx, child_element, app_state);
        }

        // Issue a scroll command only when target_y *changed*. This means
        // the user can scroll freely with the mouse wheel without us
        // immediately yanking the viewport back, as long as the
        // selection (or whatever drives target_y) stays put.
        if self.target_y != prev.target_y {
            if let Some(y) = self.target_y {
                // Width is intentionally narrow — `pan_viewport_to` only
                // cares about the rect's vertical span when computing
                // vertical movement.
                let target = Rect::new(0.0, y, 1.0, y + self.row_height);
                widgets::Portal::pan_viewport_to(&mut element, target);
            }
        }
    }

    fn teardown(
        &self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
    ) {
        let child_element = widgets::Portal::child_mut(&mut element);
        self.child.teardown(view_state, ctx, child_element);
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        message: &mut MessageCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) -> MessageResult<Action> {
        let child_element = widgets::Portal::child_mut(&mut element);
        self.child
            .message(view_state, message, child_element, app_state)
    }
}
