//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Xilem view wrapper for the round progress widget.

use std::marker::PhantomData;

use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewMarker};
use xilem::{Color, Pod, ViewCtx};

use super::round_widget::{RoundProgressSize, RoundProgressWidget};
use super::widget::ProgressStyle;

/// Build a determinate round progress widget at the normal size.
///
/// `value` is normalised against `min..max`; clamps out-of-range
/// values rather than panicking. Switch to a busy / indeterminate
/// indicator with `.busy()`; switch to the small (text-height)
/// variant with `.small()`.
///
/// # Example
///
/// ```ignore
/// use xilem_extras::progress::{round_progress, ProgressStyle};
/// use xilem::Color;
///
/// // Plain determinate ring at the normal size.
/// round_progress(0.42, 0.0, 1.0)
///
/// // Inline indicator next to text.
/// round_progress(0.66, 0.0, 1.0).small()
///
/// // Busy spinner — ignores the value.
/// round_progress(0.0, 0.0, 1.0).busy()
/// ```
pub fn round_progress<State, Action>(
    value: f64,
    min: f64,
    max: f64,
) -> RoundProgressView<State, Action> {
    RoundProgressView {
        value,
        min,
        max,
        size: RoundProgressSize::Normal,
        style: ProgressStyle::Tint,
        tint: Color::from_rgb8(0x4A, 0x9E, 0xFF),
        busy: false,
        reversed: false,
        _phantom: PhantomData,
    }
}

/// Round progress view.
pub struct RoundProgressView<State, Action> {
    value: f64,
    min: f64,
    max: f64,
    size: RoundProgressSize,
    style: ProgressStyle,
    tint: Color,
    busy: bool,
    reversed: bool,
    _phantom: PhantomData<fn(&mut State) -> Action>,
}

impl<State, Action> RoundProgressView<State, Action> {
    /// Switch to the small (text-height) variant — sized to sit
    /// inline with body text.
    pub fn small(mut self) -> Self {
        self.size = RoundProgressSize::Small;
        self
    }

    /// Set the visual style explicitly.
    pub fn style(mut self, style: ProgressStyle) -> Self {
        self.style = style;
        self
    }

    /// Convenience: solid colour interpolated through the
    /// green / orange / red threshold bands.
    pub fn gradient(mut self) -> Self {
        self.style = ProgressStyle::Tint;
        self
    }

    /// Convenience: single fixed colour, no level-based
    /// interpolation.
    pub fn monochrome(mut self, color: Color) -> Self {
        self.style = ProgressStyle::Monochrome;
        self.tint = color;
        self
    }

    /// Set the tint colour. Used by `Monochrome` style and by
    /// the rotating segment in busy mode regardless of style.
    pub fn tint(mut self, color: Color) -> Self {
        self.tint = color;
        self
    }

    /// Switch to busy / indeterminate mode. The arc becomes a
    /// rotating segment; the value is ignored.
    pub fn busy(mut self) -> Self {
        self.busy = true;
        self
    }

    /// Reverse the colour gradient — high values become green,
    /// low values become red. See `progress_bar`'s `.reversed()`
    /// for full semantics. No-op on `Monochrome` style and on
    /// busy mode.
    pub fn reversed(mut self) -> Self {
        self.reversed = true;
        self
    }
}

impl<State, Action> ViewMarker for RoundProgressView<State, Action> {}

impl<State, Action> View<State, Action, ViewCtx> for RoundProgressView<State, Action>
where
    State: 'static,
    Action: 'static,
{
    type Element = Pod<RoundProgressWidget>;
    type ViewState = ();

    fn build(&self, ctx: &mut ViewCtx, _: &mut State) -> (Self::Element, Self::ViewState) {
        let w = RoundProgressWidget::new(self.value, self.min, self.max, self.size)
            .with_style(self.style)
            .with_tint(self.tint)
            .with_busy(self.busy)
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
            RoundProgressWidget::set_value(&mut element, self.value);
        }
        if prev.min != self.min || prev.max != self.max {
            RoundProgressWidget::set_range(&mut element, self.min, self.max);
        }
        if prev.style != self.style {
            RoundProgressWidget::set_style(&mut element, self.style);
        }
        if prev.tint != self.tint {
            RoundProgressWidget::set_tint(&mut element, self.tint);
        }
        if prev.size != self.size {
            RoundProgressWidget::set_size(&mut element, self.size);
        }
        if prev.busy != self.busy {
            RoundProgressWidget::set_busy(&mut element, self.busy);
        }
        if prev.reversed != self.reversed {
            RoundProgressWidget::set_reversed(&mut element, self.reversed);
        }
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
