//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Xilem view for the switch widget.

use std::marker::PhantomData;

use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewMarker};
use xilem::{Pod, ViewCtx};

use crate::masonry::components::switch_widget::{SwitchToggled, SwitchWidget};

/// Compact synth-styled on/off switch view.
pub fn synth_switch<F, State, Action>(on: bool, callback: F) -> SynthSwitch<State, Action, F>
where
    F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
    State: 'static,
{
    SynthSwitch {
        on,
        callback,
        disabled: false,
        phantom: PhantomData,
    }
}

#[must_use = "View values do nothing unless provided to Xilem."]
pub struct SynthSwitch<State, Action, F> {
    on: bool,
    callback: F,
    disabled: bool,
    phantom: PhantomData<fn(State) -> Action>,
}

impl<State, Action, F> SynthSwitch<State, Action, F> {
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<State, Action, F> ViewMarker for SynthSwitch<State, Action, F> {}

impl<F, State, Action> View<State, Action, ViewCtx> for SynthSwitch<State, Action, F>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
{
    type Element = Pod<SwitchWidget>;
    type ViewState = ();

    fn build(&self, ctx: &mut ViewCtx, _: &mut State) -> (Self::Element, Self::ViewState) {
        let element = ctx.with_action_widget(|ctx| {
            let mut pod = ctx.create_pod(SwitchWidget::new(self.on));
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
        if prev.on != self.on {
            SwitchWidget::set_on(&mut element, self.on);
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
        match message.take_message::<SwitchToggled>() {
            Some(toggled) => MessageResult::Action((self.callback)(app_state, toggled.0)),
            None => MessageResult::Stale,
        }
    }
}
