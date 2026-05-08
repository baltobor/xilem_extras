//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Xilem view wrapper for the hexagonal busy indicator.

use std::marker::PhantomData;

use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewMarker};
use xilem::{Color, Pod, ViewCtx};

use super::busy_hex_widget::{BusyHexSize, BusyHexWidget};

/// Build a hexagonal busy / indeterminate indicator. Seven cells
/// arranged in a beehive cluster, each breathing on a phase-
/// shifted sine cycle so the wave runs around the comb.
///
/// # Example
///
/// ```ignore
/// use xilem_extras::progress::busy_hex;
///
/// busy_hex()
///
/// // Inline next to body text.
/// busy_hex().small()
///
/// // Override the default honey amber.
/// busy_hex().tint(xilem::Color::from_rgb8(0x4A, 0x9E, 0xFF))
/// ```
pub fn busy_hex<State, Action>() -> BusyHexView<State, Action> {
    BusyHexView {
        size: BusyHexSize::Normal,
        tint: None,
        busy: true,
        hide_when_not_busy: false,
        _phantom: PhantomData,
    }
}

/// Hexagonal busy indicator view.
pub struct BusyHexView<State, Action> {
    size: BusyHexSize,
    tint: Option<Color>,
    busy: bool,
    hide_when_not_busy: bool,
    _phantom: PhantomData<fn(&mut State) -> Action>,
}

impl<State, Action> BusyHexView<State, Action> {
    /// Switch to the small (text-height) variant — sized to sit
    /// inline with body text.
    pub fn small(mut self) -> Self {
        self.size = BusyHexSize::Small;
        self
    }

    /// Override the default honey-amber tint. All seven cells
    /// share the same tint; only their alpha animates.
    pub fn tint(mut self, color: Color) -> Self {
        self.tint = Some(color);
        self
    }

    /// Drive the indicator on/off. When `busy` is `false` the
    /// animation pauses; pair with `.hide_when_not_busy(true)`
    /// to make the indicator disappear from the layout entirely
    /// while idle.
    pub fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    /// When `true`, the indicator reports a zero footprint and
    /// skips painting whenever `busy` is `false`. Useful when
    /// the spinner should occupy space only while a task is
    /// actually running.
    pub fn hide_when_not_busy(mut self, hide: bool) -> Self {
        self.hide_when_not_busy = hide;
        self
    }
}

impl<State, Action> ViewMarker for BusyHexView<State, Action> {}

impl<State, Action> View<State, Action, ViewCtx> for BusyHexView<State, Action>
where
    State: 'static,
    Action: 'static,
{
    type Element = Pod<BusyHexWidget>;
    type ViewState = ();

    fn build(&self, ctx: &mut ViewCtx, _: &mut State) -> (Self::Element, Self::ViewState) {
        let mut w = BusyHexWidget::new(self.size)
            .with_busy(self.busy)
            .with_hide_when_not_busy(self.hide_when_not_busy);
        if let Some(c) = self.tint {
            w = w.with_tint(c);
        }
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
        if prev.size != self.size {
            BusyHexWidget::set_size(&mut element, self.size);
        }
        if prev.tint != self.tint {
            // Falling back to default tint on `None` matches the
            // builder's intent — no tint specified means default.
            if let Some(c) = self.tint {
                BusyHexWidget::set_tint(&mut element, c);
            }
        }
        if prev.hide_when_not_busy != self.hide_when_not_busy {
            BusyHexWidget::set_hide_when_not_busy(&mut element, self.hide_when_not_busy);
        }
        if prev.busy != self.busy {
            BusyHexWidget::set_busy(&mut element, self.busy);
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
