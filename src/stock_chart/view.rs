//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Stock chart view implementation.

use std::marker::PhantomData;

use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewMarker};
use xilem::{Pod, ViewCtx};

use super::widget::{StockBar, StockChartHover, StockChartMode, StockChartStyle, StockChartWidget};

/// Stock chart view for displaying OHLCV financial data.
///
/// # Example
///
/// ```ignore
/// use xilem_extras::stock_chart::{stock_chart, StockBar, StockChartMode};
///
/// let bars = vec![
///     StockBar::new("2024-01-01", 100.0, 105.0, 98.0, 103.0, 1000),
/// ];
///
/// stock_chart(bars, StockChartMode::Candlestick, |model, hover| {
///     if let Some(h) = hover {
///         model.hovered_price = Some(h.close);
///     }
/// })
/// ```
pub struct StockChartView<State, Action, F> {
    bars: Vec<StockBar>,
    mode: StockChartMode,
    style: StockChartStyle,
    on_hover: F,
    marker: PhantomData<fn(&mut State) -> Action>,
}

/// Creates a stock chart view.
pub fn stock_chart<State, Action, F>(
    bars: Vec<StockBar>,
    mode: StockChartMode,
    on_hover: F,
) -> StockChartView<State, Action, F>
where
    F: Fn(&mut State, Option<StockChartHover>) -> Action + Send + Sync + 'static,
{
    StockChartView {
        bars,
        mode,
        style: StockChartStyle::default(),
        on_hover,
        marker: PhantomData,
    }
}

impl<State, Action, F> StockChartView<State, Action, F> {
    /// Sets a custom style for the chart.
    pub fn style(mut self, style: StockChartStyle) -> Self {
        self.style = style;
        self
    }

    /// Configures whether to show volume bars.
    pub fn show_volume(mut self, show: bool) -> Self {
        self.style.show_volume = show;
        self
    }

    /// Configures whether to show grid lines.
    pub fn show_grid(mut self, show: bool) -> Self {
        self.style.show_grid = show;
        self
    }

    /// Configures whether to show crosshair on hover.
    pub fn show_crosshair(mut self, show: bool) -> Self {
        self.style.show_crosshair = show;
        self
    }
}

impl<State, Action, F> ViewMarker for StockChartView<State, Action, F> {}

impl<State, Action, F> View<State, Action, ViewCtx> for StockChartView<State, Action, F>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, Option<StockChartHover>) -> Action + Send + Sync + 'static,
{
    type Element = Pod<StockChartWidget>;
    type ViewState = ();

    fn build(&self, ctx: &mut ViewCtx, _state: &mut State) -> (Self::Element, Self::ViewState) {
        let widget = StockChartWidget::with_style(
            self.bars.clone(),
            self.mode,
            self.style.clone(),
        );
        let pod = ctx.with_action_widget(|ctx| {
            ctx.create_pod(widget)
        });
        (pod, ())
    }

    fn rebuild(
        &self,
        prev: &Self,
        _view_state: &mut Self::ViewState,
        _ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        _state: &mut State,
    ) {
        if self.bars.len() != prev.bars.len() || self.mode != prev.mode {
            element.widget.set_bars(self.bars.clone());
            element.widget.set_mode(self.mode);
            element.ctx.request_layout();
        }

        // Check for style changes
        if self.style.show_volume != prev.style.show_volume
            || self.style.show_grid != prev.style.show_grid
            || self.style.show_crosshair != prev.style.show_crosshair
        {
            element.widget.set_style(self.style.clone());
            element.ctx.request_paint_only();
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
        state: &mut State,
    ) -> MessageResult<Action> {
        match message.take_message::<Option<StockChartHover>>() {
            Some(boxed) => {
                let hover = *boxed;
                let result = (self.on_hover)(state, hover);
                MessageResult::Action(result)
            }
            None => MessageResult::Stale,
        }
    }
}
