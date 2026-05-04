//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Xilem view wrapper for the linear progress bar.

use std::marker::PhantomData;

use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewMarker};
use xilem::{Color, Pod, ViewCtx};

use super::widget::{ProgressBarWidget, ProgressOrientation, ProgressStyle};

/// Build a horizontal linear progress bar with the gradient
/// style by default. Switch to other styles via `.tint()` /
/// `.monochrome(color)` / `.style(...)`.
///
/// `value` is normalised against `min..max`. The widget clamps
/// out-of-range values and never panics.
///
/// # Example
///
/// ```ignore
/// use xilem_extras::progress::{progress_bar, ProgressStyle};
/// use xilem::Color;
///
/// // Plain download bar — single fixed colour.
/// progress_bar(0.42, 0.0, 1.0).monochrome(Color::from_rgb8(0x4A, 0x9E, 0xFF))
///
/// // Health/level meter — three coloured zones.
/// progress_bar(0.85, 0.0, 1.0)  // gradient is the default
///
/// // Solid bar that shifts colour with level.
/// progress_bar(0.6, 0.0, 1.0).tint()
/// ```
pub fn progress_bar<State, Action>(
    value: f64,
    min: f64,
    max: f64,
) -> ProgressBarView<State, Action> {
    ProgressBarView {
        value,
        min,
        max,
        orientation: ProgressOrientation::Horizontal,
        style: ProgressStyle::Gradient,
        mono_tint: Color::from_rgb8(0x4A, 0x9E, 0xFF),
        main_axis_len: 120.0,
        cross_axis_len: 6.0,
        reversed: false,
        _phantom: PhantomData,
    }
}

/// Linear progress bar view.
pub struct ProgressBarView<State, Action> {
    value: f64,
    min: f64,
    max: f64,
    orientation: ProgressOrientation,
    style: ProgressStyle,
    mono_tint: Color,
    main_axis_len: f64,
    cross_axis_len: f64,
    reversed: bool,
    _phantom: PhantomData<fn(&mut State) -> Action>,
}

impl<State, Action> ProgressBarView<State, Action> {
    /// Switch to a vertical bar (fills bottom-up).
    pub fn vertical(mut self) -> Self {
        self.orientation = ProgressOrientation::Vertical;
        self
    }

    /// Set the visual style explicitly.
    pub fn style(mut self, style: ProgressStyle) -> Self {
        self.style = style;
        self
    }

    /// Convenience: solid colour interpolated through the
    /// green/orange/red threshold bands.
    pub fn tint(mut self) -> Self {
        self.style = ProgressStyle::Tint;
        self
    }

    /// Convenience: single fixed colour, no interpolation.
    pub fn monochrome(mut self, color: Color) -> Self {
        self.style = ProgressStyle::Monochrome;
        self.mono_tint = color;
        self
    }

    /// Override the bar's main-axis length (width when
    /// horizontal, height when vertical).
    pub fn main_axis_len(mut self, len: f64) -> Self {
        self.main_axis_len = len;
        self
    }

    /// Override the bar's cross-axis thickness.
    pub fn cross_axis_len(mut self, len: f64) -> Self {
        self.cross_axis_len = len;
        self
    }

    /// Reverse the colour gradient.
    ///
    /// Default (`false`): high values are "bad" — green at the
    /// low end, red at the high end. Suits level / load /
    /// pressure indicators where "more is worse".
    ///
    /// `true`: high values are "good" — green at the high end,
    /// red at the low end. Suits quality scores, completion
    /// percentages, and similar where "more is better" (SOLID
    /// scores being the motivating case).
    ///
    /// No-op on `Monochrome` style (which has no level-based
    /// colour mapping).
    pub fn reversed(mut self) -> Self {
        self.reversed = true;
        self
    }
}

impl<State, Action> ViewMarker for ProgressBarView<State, Action> {}

impl<State, Action> View<State, Action, ViewCtx> for ProgressBarView<State, Action>
where
    State: 'static,
    Action: 'static,
{
    type Element = Pod<ProgressBarWidget>;
    type ViewState = ();

    fn build(&self, ctx: &mut ViewCtx, _: &mut State) -> (Self::Element, Self::ViewState) {
        let w = ProgressBarWidget::new(self.value, self.min, self.max, self.orientation)
            .with_style(self.style)
            .with_tint(self.mono_tint)
            .with_main_axis_len(self.main_axis_len)
            .with_cross_axis_len(self.cross_axis_len)
            .with_reversed(self.reversed);
        let pod = ctx.with_action_widget(|ctx| ctx.create_pod(w));
        (pod, ())
    }

    fn rebuild(
        &self,
        prev: &Self,
        _: &mut (),
        _: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        _: &mut State,
    ) {
        if prev.value != self.value {
            ProgressBarWidget::set_value(&mut element, self.value);
        }
        if prev.min != self.min || prev.max != self.max {
            ProgressBarWidget::set_range(&mut element, self.min, self.max);
        }
        if prev.style != self.style {
            ProgressBarWidget::set_style(&mut element, self.style);
        }
        if prev.mono_tint != self.mono_tint {
            ProgressBarWidget::set_tint(&mut element, self.mono_tint);
        }
        if prev.reversed != self.reversed {
            ProgressBarWidget::set_reversed(&mut element, self.reversed);
        }
        // main_axis_len / cross_axis_len / orientation are not
        // hot-swappable — they affect measure() and would need a
        // rebuild. Callers that change them re-instantiate.
    }

    fn teardown(&self, _: &mut (), ctx: &mut ViewCtx, element: Mut<'_, Self::Element>) {
        ctx.teardown_action_source(element);
    }

    fn message(
        &self,
        _: &mut (),
        _: &mut MessageCtx,
        _: Mut<'_, Self::Element>,
        _: &mut State,
    ) -> MessageResult<Action> {
        MessageResult::Stale
    }
}
