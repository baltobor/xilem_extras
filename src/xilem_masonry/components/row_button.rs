//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Xilem view for the row button widget.

use xilem::core::MessageResult;
use xilem::core::{MessageCtx, Mut, View, ViewId, ViewMarker, ViewPathTracker};
use xilem::masonry::core::PointerButton;
use xilem::masonry::peniko::Color;
use xilem::{Pod, ViewCtx, WidgetView};

use crate::masonry::components::row_button::{RowButton, RowButtonPress};

pub use crate::masonry::components::row_button::RowButtonPress as RowButtonPressReexport;

const CHILD_VIEW_ID: ViewId = ViewId::new(0);

/// Xilem view for a row button - left-aligned, full-width, with hover highlight.
pub struct RowButtonView<F, V> {
    child: V,
    callback: F,
    hover_bg: Color,
    disabled: bool,
}

/// Create a row button with left-aligned child content.
///
/// The button stretches to fill available width and highlights on hover.
pub fn row_button<State: 'static, Action: 'static, V: WidgetView<State, Action>>(
    child: V,
    callback: impl Fn(&mut State) -> Action + Send + Sync + 'static,
) -> RowButtonView<
    impl for<'a> Fn(&'a mut State, &RowButtonPress) -> MessageResult<Action> + Send + 'static,
    V,
> {
    RowButtonView {
        child,
        callback: move |state: &mut State, press: &RowButtonPress| match press.button {
            None | Some(PointerButton::Primary) => MessageResult::Action(callback(state)),
            _ => MessageResult::Nop,
        },
        hover_bg: Color::TRANSPARENT,
        disabled: false,
    }
}

/// Create a row button that receives click count (for double-click handling).
pub fn row_button_with_clicks<State: 'static, Action: 'static, V: WidgetView<State, Action>>(
    child: V,
    callback: impl Fn(&mut State, u8) -> Action + Send + Sync + 'static,
) -> RowButtonView<
    impl for<'a> Fn(&'a mut State, &RowButtonPress) -> MessageResult<Action> + Send + 'static,
    V,
> {
    RowButtonView {
        child,
        callback: move |state: &mut State, press: &RowButtonPress| match press.button {
            None | Some(PointerButton::Primary) => {
                MessageResult::Action(callback(state, press.click_count))
            }
            _ => MessageResult::Nop,
        },
        hover_bg: Color::TRANSPARENT,
        disabled: false,
    }
}

/// Create a row button that receives keyboard modifiers (for Cmd+click, Shift+click, etc.).
pub fn row_button_with_modifiers<
    State: 'static,
    Action: 'static,
    V: WidgetView<State, Action>,
>(
    child: V,
    callback: impl Fn(&mut State, xilem::masonry::core::Modifiers) -> Action + Send + Sync + 'static,
) -> RowButtonView<
    impl for<'a> Fn(&'a mut State, &RowButtonPress) -> MessageResult<Action> + Send + 'static,
    V,
> {
    RowButtonView {
        child,
        callback: move |state: &mut State, press: &RowButtonPress| match press.button {
            None | Some(PointerButton::Primary) => {
                MessageResult::Action(callback(state, press.modifiers))
            }
            _ => MessageResult::Nop,
        },
        hover_bg: Color::TRANSPARENT,
        disabled: false,
    }
}

/// Create a row button that receives full press information.
pub fn row_button_with_press<State: 'static, Action: 'static, V: WidgetView<State, Action>>(
    child: V,
    callback: impl Fn(&mut State, &RowButtonPress) -> Action + Send + Sync + 'static,
) -> RowButtonView<
    impl for<'a> Fn(&'a mut State, &RowButtonPress) -> MessageResult<Action> + Send + 'static,
    V,
> {
    RowButtonView {
        child,
        callback: move |state: &mut State, press: &RowButtonPress| {
            MessageResult::Action(callback(state, press))
        },
        hover_bg: Color::TRANSPARENT,
        disabled: false,
    }
}

impl<F, V> RowButtonView<F, V> {
    pub fn hover_bg(mut self, color: Color) -> Self {
        self.hover_bg = color;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<F, V> ViewMarker for RowButtonView<F, V> {}

impl<F, V, State, Action> View<State, Action, ViewCtx> for RowButtonView<F, V>
where
    V: WidgetView<State, Action>,
    F: Fn(&mut State, &RowButtonPress) -> MessageResult<Action> + Send + Sync + 'static,
    State: 'static,
    Action: 'static,
{
    type Element = Pod<RowButton>;
    type ViewState = V::ViewState;

    fn build(&self, ctx: &mut ViewCtx, app_state: &mut State) -> (Self::Element, Self::ViewState) {
        let (child_pod, child_state) =
            ctx.with_id(CHILD_VIEW_ID, |ctx| self.child.build(ctx, app_state));
        let pod = ctx.with_action_widget(|ctx| {
            let widget = RowButton::new(child_pod.new_widget).with_hover_bg(self.hover_bg);
            let mut pod = ctx.create_pod(widget);
            pod.new_widget.options.disabled = self.disabled;
            pod
        });
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
        if prev.disabled != self.disabled {
            element.ctx.set_disabled(self.disabled);
        }
        if prev.hover_bg != self.hover_bg {
            RowButton::set_hover_bg(&mut element, self.hover_bg);
        }
        ctx.with_id(CHILD_VIEW_ID, |ctx| {
            self.child.rebuild(
                &prev.child,
                view_state,
                ctx,
                RowButton::child_mut(&mut element).downcast(),
                app_state,
            );
        });
    }

    fn teardown(
        &self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
    ) {
        ctx.with_id(CHILD_VIEW_ID, |ctx| {
            self.child.teardown(
                view_state,
                ctx,
                RowButton::child_mut(&mut element).downcast(),
            );
        });
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        message: &mut MessageCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) -> MessageResult<Action> {
        match message.take_first() {
            Some(CHILD_VIEW_ID) => self.child.message(
                view_state,
                message,
                RowButton::child_mut(&mut element).downcast(),
                app_state,
            ),
            None => match message.take_message::<RowButtonPress>() {
                Some(press) => (self.callback)(app_state, &press),
                None => MessageResult::Stale,
            },
            _ => MessageResult::Stale,
        }
    }
}
