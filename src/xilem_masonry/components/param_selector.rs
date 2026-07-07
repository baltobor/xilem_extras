//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Xilem view for the parameter selector widget.

use std::marker::PhantomData;

use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewMarker};
use xilem::masonry::peniko::Color;
use xilem::{Pod, ViewCtx};

use crate::masonry::components::param_selector::{LabelAlign, ParamSelectorWidget};

/// Vertical multi-choice selector view.
pub fn param_selector<State, Action, F>(
    labels: Vec<String>,
    selected: usize,
    on_change: F,
) -> ParamSelectorView<State, Action, F>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, usize) -> Action + Send + Sync + 'static,
{
    ParamSelectorView {
        labels,
        selected,
        on_change,
        label_align: LabelAlign::Left,
        tint: None,
        label_colors: None,
        _phantom: PhantomData,
    }
}

#[must_use = "View values do nothing unless provided to Xilem."]
pub struct ParamSelectorView<State, Action, F> {
    labels: Vec<String>,
    selected: usize,
    on_change: F,
    label_align: LabelAlign,
    tint: Option<Color>,
    label_colors: Option<(Color, Color)>,
    _phantom: PhantomData<fn(&mut State) -> Action>,
}

impl<State, Action, F> ParamSelectorView<State, Action, F> {
    pub fn label_align(mut self, align: LabelAlign) -> Self {
        self.label_align = align;
        self
    }

    pub fn tint(mut self, color: Color) -> Self {
        self.tint = Some(color);
        self
    }

    pub fn label_colors(mut self, selected: Color, unselected: Color) -> Self {
        self.label_colors = Some((selected, unselected));
        self
    }
}

impl<State, Action, F> ViewMarker for ParamSelectorView<State, Action, F> {}

impl<F, State, Action> View<State, Action, ViewCtx> for ParamSelectorView<State, Action, F>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, usize) -> Action + Send + Sync + 'static,
{
    type Element = Pod<ParamSelectorWidget>;
    type ViewState = ();

    fn build(&self, ctx: &mut ViewCtx, _: &mut State) -> (Self::Element, Self::ViewState) {
        let mut w = ParamSelectorWidget::new(self.labels.clone(), self.selected, self.label_align);
        if let Some(c) = self.tint {
            w = w.with_tint(c);
        }
        if let Some((sel, unsel)) = self.label_colors {
            w = w.with_label_colors(sel, unsel);
        }
        let pod = ctx.with_action_widget(|ctx| ctx.create_pod(w));
        (pod, ())
    }

    fn rebuild(
        &self,
        prev: &Self,
        _: &mut Self::ViewState,
        _: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        _: &mut State,
    ) {
        if prev.selected != self.selected {
            ParamSelectorWidget::set_selected(&mut element, self.selected);
        }
        if prev.labels != self.labels {
            ParamSelectorWidget::set_labels(&mut element, self.labels.clone());
        }
        if prev.label_align != self.label_align {
            ParamSelectorWidget::set_label_align(&mut element, self.label_align);
        }
        if prev.tint != self.tint {
            if let Some(c) = self.tint {
                ParamSelectorWidget::set_tint(&mut element, c);
            }
        }
        if prev.label_colors != self.label_colors {
            if let Some((sel, unsel)) = self.label_colors {
                ParamSelectorWidget::set_label_colors(&mut element, sel, unsel);
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
        if message.take_first().is_some() {
            return MessageResult::Stale;
        }
        match message.take_message::<usize>() {
            Some(idx) => MessageResult::Action((self.on_change)(app_state, *idx)),
            None => MessageResult::Stale,
        }
    }
}
