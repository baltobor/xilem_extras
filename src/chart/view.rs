//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Xilem view wrapper for the chart widget.

use std::marker::PhantomData;

use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewMarker};
use xilem::Pod;
use xilem::ViewCtx;

use super::widget::{ChartWidget, ChartAction, ChartMode};

/// Creates a chart view.
///
/// # Arguments
///
/// * `title` - Chart title/caption
/// * `values` - Data values to display
/// * `labels` - X-axis labels
/// * `mode` - Bar or line chart mode
///
/// # Example
///
/// ```ignore
/// use xilem_extras::chart::{chart, ChartMode};
///
/// chart(
///     "Sales This Year",
///     &[10.0, 25.0, 15.0, 30.0],
///     vec!["Q1", "Q2", "Q3", "Q4"],
///     ChartMode::Bar,
/// )
/// ```
pub fn chart<State, Action>(
    title: impl Into<String>,
    values: &[f64],
    labels: Vec<String>,
    mode: ChartMode,
) -> ChartView<State, Action> {
    ChartView {
        title: title.into(),
        values: values.to_vec(),
        labels,
        mode,
        show_values: true,
        _phantom: PhantomData,
    }
}

/// Chart view for xilem.
pub struct ChartView<State, Action> {
    title: String,
    values: Vec<f64>,
    labels: Vec<String>,
    mode: ChartMode,
    show_values: bool,
    _phantom: PhantomData<fn(&mut State) -> Action>,
}

impl<State, Action> ChartView<State, Action> {
    /// Sets whether to show value labels above data points.
    pub fn show_values(mut self, show: bool) -> Self {
        self.show_values = show;
        self
    }
}

#[derive(Clone)]
pub struct ChartViewState {
    values: Vec<f64>,
    labels: Vec<String>,
}

impl<State, Action> ViewMarker for ChartView<State, Action> {}

impl<State: 'static, Action: 'static> View<State, Action, ViewCtx> for ChartView<State, Action> {
    type Element = Pod<ChartWidget>;
    type ViewState = ChartViewState;

    fn build(
        &self,
        ctx: &mut ViewCtx,
        _app_state: &mut State,
    ) -> (Self::Element, Self::ViewState) {
        let widget = ChartWidget::new(
            self.title.clone(),
            self.values.clone(),
            self.labels.clone(),
            self.mode,
        ).with_show_values(self.show_values);
        let pod = ctx.with_action_widget(|ctx| {
            ctx.create_pod(widget)
        });
        let state = ChartViewState {
            values: self.values.clone(),
            labels: self.labels.clone(),
        };
        (pod, state)
    }

    fn rebuild(
        &self,
        _prev: &Self,
        view_state: &mut Self::ViewState,
        _ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        _app_state: &mut State,
    ) {
        // Update the widget data if changed
        if self.values != view_state.values || self.labels != view_state.labels {
            ChartWidget::set_data(&mut element, self.values.clone(), self.labels.clone());
            view_state.values = self.values.clone();
            view_state.labels = self.labels.clone();
        }
    }

    fn teardown(
        &self,
        _view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        element: Mut<'_, Self::Element>,
    ) {
        ctx.teardown_action_source(element);
    }

    fn message(
        &self,
        _view_state: &mut Self::ViewState,
        message: &mut MessageCtx,
        _element: Mut<'_, Self::Element>,
        _app_state: &mut State,
    ) -> MessageResult<Action> {
        if message.take_message::<ChartAction>().is_some() {
            // Could handle chart interactions here
        }
        MessageResult::Stale
    }
}
