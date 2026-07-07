//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Xilem view for the radio widget.

use std::marker::PhantomData;

use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewMarker};
use xilem::masonry::peniko::Color;
use xilem::{Pod, ViewCtx};

use crate::masonry::components::radio_widget::{RadioToggled, RadioWidget};

/// Synth-styled single radio button view.
pub fn synth_radio<F, State, Action>(selected: bool, callback: F) -> SynthRadio<State, Action, F>
where
    F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
    State: 'static,
{
    SynthRadio {
        selected,
        callback,
        tint: None,
        disabled: false,
        phantom: PhantomData,
    }
}

#[must_use = "View values do nothing unless provided to Xilem."]
pub struct SynthRadio<State, Action, F> {
    selected: bool,
    callback: F,
    tint: Option<Color>,
    disabled: bool,
    phantom: PhantomData<fn(State) -> Action>,
}

impl<State, Action, F> SynthRadio<State, Action, F> {
    pub fn tint(mut self, color: Color) -> Self {
        self.tint = Some(color);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<State, Action, F> ViewMarker for SynthRadio<State, Action, F> {}

impl<F, State, Action> View<State, Action, ViewCtx> for SynthRadio<State, Action, F>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
{
    type Element = Pod<RadioWidget>;
    type ViewState = ();

    fn build(&self, ctx: &mut ViewCtx, _: &mut State) -> (Self::Element, Self::ViewState) {
        let mut w = RadioWidget::new(self.selected);
        if let Some(c) = self.tint {
            w = w.with_tint(c);
        }
        let element = ctx.with_action_widget(|ctx| {
            let mut pod = ctx.create_pod(w);
            pod.new_widget.options.disabled = self.disabled;
            pod
        });
        (element, ())
    }

    fn rebuild(
        &self,
        prev: &Self,
        _: &mut Self::ViewState,
        _ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        _: &mut State,
    ) {
        if prev.disabled != self.disabled {
            element.ctx.set_disabled(self.disabled);
        }
        if prev.selected != self.selected {
            RadioWidget::set_selected(&mut element, self.selected);
        }
        if prev.tint != self.tint {
            if let Some(c) = self.tint {
                RadioWidget::set_tint(&mut element, c);
            }
        }
    }

    fn teardown(
        &self,
        _: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        element: Mut<'_, Self::Element>,
    ) {
        ctx.teardown_action_source(element);
    }

    fn message(
        &self,
        _: &mut Self::ViewState,
        message: &mut MessageCtx,
        _element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) -> MessageResult<Action> {
        match message.take_message::<RadioToggled>() {
            Some(toggled) => MessageResult::Action((self.callback)(app_state, toggled.0)),
            None => MessageResult::Stale,
        }
    }
}
