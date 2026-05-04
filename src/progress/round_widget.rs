//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Round progress widget — a thin arc that sweeps from 0 % to
//! 100 %.
//!
//! Geometry borrowed from `xilem_synth_widgets::widgets::knob`
//! (track arc + lit arc + a configurable tint), with the
//! interactive parts (drag, body fill, indicator line) stripped
//! out. The arc starts at 12 o'clock and sweeps clockwise — the
//! conventional round-progress gesture, distinct from the knob's
//! 270° dial.
//!
//! Two sizes:
//! - **Normal** — 36×36 px outer box, 18 px radius.
//! - **Small** — 14×14 px outer box, ~6.5 px radius. Sized to
//!   sit inline with body text so it can stand in for a tiny
//!   inline progress glyph.
//!
//! Doubles as a **busy / indeterminate indicator** when
//! `set_busy(true)` is set. The lit arc becomes a fixed-length
//! ~45° segment that rotates around the track at a steady
//! ~1 rev / sec rate.

use std::f64::consts::{PI, TAU};

use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::core::{
    AccessCtx, EventCtx, LayoutCtx, MeasureCtx, PaintCtx, PointerEvent, PropertiesMut,
    PropertiesRef, RegisterCtx, Update, UpdateCtx, Widget, WidgetId, WidgetMut,
};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Arc, Axis, Cap, Point, Size, Stroke, Vec2};
use xilem::masonry::layout::LenReq;
use xilem::Color;

use smallvec::SmallVec;
use tracing::trace_span;

use super::widget::ProgressStyle;

/// Outer box radius for the normal size — leaves a small gap
/// to the bounding box so antialiasing doesn't clip the stroke.
const RADIUS_NORMAL: f64 = 18.0;
/// Outer box radius for the small (text-height) size.
const RADIUS_SMALL: f64 = 6.5;
/// Stroke thickness for the normal size.
const RING_NORMAL: f64 = 3.0;
/// Stroke thickness for the small size.
const RING_SMALL: f64 = 1.5;
/// Padding added around the radius to size the bounding box.
const BOX_PADDING: f64 = 2.0;

/// Track colour (the unfilled portion of the ring).
const TRACK_COLOR: Color = Color::from_rgb8(0x40, 0x40, 0x40);
/// Default tint for `Monochrome` style when the caller doesn't
/// override.
const DEFAULT_MONO_TINT: Color = Color::from_rgb8(0x4A, 0x9E, 0xFF);
/// Three-zone palette for `Gradient` / `Tint` styles, mirroring
/// the linear progress bar so the same colour choices read
/// consistently across both widgets.
const GREEN: Color = Color::from_rgb8(0x30, 0xC0, 0x30);
const ORANGE: Color = Color::from_rgb8(0xFF, 0x8C, 0x00);
const RED: Color = Color::from_rgb8(0xE0, 0x20, 0x20);

/// Threshold (as a normalised fraction of `min..max`) at which
/// gradient/tint mode transitions from green into orange.
const ZONE_LOW: f64 = 0.75;
/// Threshold at which gradient/tint mode transitions from
/// orange into red.
const ZONE_HIGH: f64 = 0.90;

/// 12 o'clock — the start angle for the lit arc. Kurbo's angle
/// convention is 0 rad pointing along +X (3 o'clock), increasing
/// clockwise; -π/2 rad points up.
const ARC_START: f64 = -PI / 2.0;
/// Full 360° sweep — round progress wraps the entire ring,
/// unlike a knob's 270° dial.
const ARC_SWEEP_FULL: f64 = TAU;
/// Length of the rotating arc segment in busy / indeterminate
/// mode (≈45°). Visible enough to read as motion without
/// looking like a half-finished progress.
const BUSY_SEGMENT: f64 = PI / 4.0;
/// One full rotation per second when busy.
const BUSY_ROTATION_HZ: f64 = 1.0;

/// Two predefined sizes for the round progress.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum RoundProgressSize {
    #[default]
    Normal,
    /// Sized to match body text height — useful inline.
    Small,
}

impl RoundProgressSize {
    fn radius(self) -> f64 {
        match self {
            RoundProgressSize::Normal => RADIUS_NORMAL,
            RoundProgressSize::Small => RADIUS_SMALL,
        }
    }
    fn ring_w(self) -> f64 {
        match self {
            RoundProgressSize::Normal => RING_NORMAL,
            RoundProgressSize::Small => RING_SMALL,
        }
    }
    fn box_side(self) -> f64 {
        self.radius() * 2.0 + BOX_PADDING * 2.0
    }
}

/// Round progress widget.
pub struct RoundProgressWidget {
    value: f64,
    min: f64,
    max: f64,
    size: RoundProgressSize,
    style: ProgressStyle,
    /// Tint colour used by `Monochrome` style (and the busy
    /// segment regardless of style).
    tint: Color,
    /// When true, paint a rotating segment instead of a value.
    busy: bool,
    /// Accumulated busy-rotation phase in radians, updated by
    /// `on_anim_frame`.
    busy_phase: f64,
    /// Reverse the colour gradient — high values become green,
    /// low values become red. See `ProgressBarWidget` for full
    /// semantics. No-op on `Monochrome` style and on busy mode
    /// (the busy segment uses `tint` directly regardless of
    /// level).
    reversed: bool,
}

impl RoundProgressWidget {
    pub fn new(value: f64, min: f64, max: f64, size: RoundProgressSize) -> Self {
        Self {
            value,
            min,
            max,
            size,
            style: ProgressStyle::Gradient,
            tint: DEFAULT_MONO_TINT,
            busy: false,
            busy_phase: 0.0,
            reversed: false,
        }
    }

    pub fn with_style(mut self, style: ProgressStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_tint(mut self, color: Color) -> Self {
        self.tint = color;
        self
    }

    pub fn with_busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    pub fn with_reversed(mut self, reversed: bool) -> Self {
        self.reversed = reversed;
        self
    }

    pub fn set_value(this: &mut WidgetMut<'_, Self>, value: f64) {
        if (this.widget.value - value).abs() > 0.001 {
            this.widget.value = value;
            this.ctx.request_render();
        }
    }

    pub fn set_range(this: &mut WidgetMut<'_, Self>, min: f64, max: f64) {
        this.widget.min = min;
        this.widget.max = max;
        this.ctx.request_render();
    }

    pub fn set_style(this: &mut WidgetMut<'_, Self>, style: ProgressStyle) {
        if this.widget.style != style {
            this.widget.style = style;
            this.ctx.request_render();
        }
    }

    pub fn set_tint(this: &mut WidgetMut<'_, Self>, color: Color) {
        this.widget.tint = color;
        this.ctx.request_render();
    }

    pub fn set_size(this: &mut WidgetMut<'_, Self>, size: RoundProgressSize) {
        if this.widget.size != size {
            this.widget.size = size;
            this.ctx.request_layout();
            this.ctx.request_render();
        }
    }

    /// Toggle the busy / indeterminate indicator. Kicking it on
    /// also kicks off the animation tick; turning it off lets
    /// the next frame settle the static lit arc.
    pub fn set_busy(this: &mut WidgetMut<'_, Self>, busy: bool) {
        if this.widget.busy != busy {
            this.widget.busy = busy;
            this.ctx.request_anim_frame();
            this.ctx.request_render();
        }
    }

    pub fn set_reversed(this: &mut WidgetMut<'_, Self>, reversed: bool) {
        if this.widget.reversed != reversed {
            this.widget.reversed = reversed;
            this.ctx.request_render();
        }
    }

    fn normalized(&self) -> f64 {
        let range = self.max - self.min;
        if range.abs() < f64::EPSILON {
            return 0.0;
        }
        ((self.value - self.min) / range).clamp(0.0, 1.0)
    }

    /// Smoothly interpolate green → orange → red based on
    /// `norm`'s position relative to the threshold bands.
    /// Identical to the linear bar's interpolation so colour
    /// reads consistently across both widgets. When `reversed`,
    /// the input is mirrored so 1.0 maps to the green end.
    fn interpolate_color(norm: f64, reversed: bool) -> Color {
        let n = if reversed { 1.0 - norm } else { norm };
        if n <= ZONE_LOW {
            let t = if ZONE_LOW > 0.0 { n / ZONE_LOW } else { 0.0 };
            lerp_color(GREEN, ORANGE, t)
        } else if n <= ZONE_HIGH {
            let span = ZONE_HIGH - ZONE_LOW;
            let t = if span > 0.0 { (n - ZONE_LOW) / span } else { 1.0 };
            lerp_color(ORANGE, RED, t)
        } else {
            RED
        }
    }
}

fn lerp_color(a: Color, b: Color, t: f64) -> Color {
    let a = a.to_rgba8();
    let b = b.to_rgba8();
    let t = t.clamp(0.0, 1.0) as f32;
    Color::from_rgb8(
        (a.r as f32 + (b.r as f32 - a.r as f32) * t) as u8,
        (a.g as f32 + (b.g as f32 - a.g as f32) * t) as u8,
        (a.b as f32 + (b.b as f32 - a.b as f32) * t) as u8,
    )
}

impl Widget for RoundProgressWidget {
    type Action = ();

    fn on_pointer_event(
        &mut self,
        _: &mut EventCtx<'_>,
        _: &mut PropertiesMut<'_>,
        _: &PointerEvent,
    ) {
    }
    fn accepts_pointer_interaction(&self) -> bool {
        false
    }
    fn accepts_focus(&self) -> bool {
        false
    }
    fn register_children(&mut self, _: &mut RegisterCtx<'_>) {}

    fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        _: &mut PropertiesMut<'_>,
        event: &Update,
    ) {
        // Kick off the animation cycle on first add when busy.
        // `on_anim_frame` keeps it going; turning busy off lets
        // the cycle die naturally.
        if matches!(event, Update::WidgetAdded) && self.busy {
            ctx.request_anim_frame();
        }
    }

    fn on_anim_frame(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        _: &mut PropertiesMut<'_>,
        interval: u64,
    ) {
        if !self.busy {
            return;
        }
        // `interval` is the elapsed time in nanoseconds since
        // the last frame. Advance the rotation phase
        // proportionally so the spin rate is wall-clock-stable
        // regardless of frame rate.
        let dt_secs = interval as f64 / 1_000_000_000.0;
        self.busy_phase = (self.busy_phase + dt_secs * BUSY_ROTATION_HZ * TAU) % TAU;
        ctx.request_render();
        ctx.request_anim_frame();
    }

    fn measure(
        &mut self,
        _: &mut MeasureCtx<'_>,
        _: &PropertiesRef<'_>,
        _axis: Axis,
        _: LenReq,
        _: Option<f64>,
    ) -> f64 {
        self.size.box_side()
    }

    fn layout(&mut self, _: &mut LayoutCtx<'_>, _: &PropertiesRef<'_>, _: Size) {}

    fn paint(
        &mut self,
        ctx: &mut PaintCtx<'_>,
        _: &PropertiesRef<'_>,
        painter: &mut Painter<'_>,
    ) {
        let size = ctx.content_box_size();
        let cx = size.width / 2.0;
        let cy = size.height / 2.0;
        let r = self.size.radius();
        let ring_w = self.size.ring_w();

        // Track — full ring at the chosen radius.
        let track =
            Arc::new(Point::new(cx, cy), Vec2::new(r, r), ARC_START, ARC_SWEEP_FULL, 0.0);
        painter
            .stroke(track, &Stroke::new(ring_w).with_caps(Cap::Round), TRACK_COLOR)
            .draw();

        if self.busy {
            // Rotating segment in busy mode — start angle
            // advances each anim frame; segment length is fixed.
            let start = ARC_START + self.busy_phase;
            let lit =
                Arc::new(Point::new(cx, cy), Vec2::new(r, r), start, BUSY_SEGMENT, 0.0);
            painter
                .stroke(lit, &Stroke::new(ring_w).with_caps(Cap::Round), self.tint)
                .draw();
            return;
        }

        let norm = self.normalized();
        if norm < 0.001 {
            return;
        }

        let lit_color = match self.style {
            ProgressStyle::Monochrome => self.tint,
            ProgressStyle::Tint | ProgressStyle::Gradient => {
                Self::interpolate_color(norm, self.reversed)
            }
        };
        // Gradient mode on a round shape collapses to Tint:
        // there's no clean way to paint three coloured zones on
        // a single arc segment without breaking it into three
        // sub-arcs. We could implement that, but the value-aware
        // single-colour reading is already the common case for
        // round indicators; punt on three-arc gradient until a
        // user actually asks for it.

        let sweep = norm * ARC_SWEEP_FULL;
        let lit = Arc::new(Point::new(cx, cy), Vec2::new(r, r), ARC_START, sweep, 0.0);
        painter
            .stroke(lit, &Stroke::new(ring_w).with_caps(Cap::Round), lit_color)
            .draw();
    }

    fn accessibility_role(&self) -> Role {
        Role::ProgressIndicator
    }

    fn accessibility(&mut self, _: &mut AccessCtx<'_>, _: &PropertiesRef<'_>, node: &mut Node) {
        if !self.busy {
            node.set_numeric_value(self.value);
            node.set_min_numeric_value(self.min);
            node.set_max_numeric_value(self.max);
        }
    }

    fn children_ids(&self) -> SmallVec<[WidgetId; 16]> {
        SmallVec::new()
    }

    fn make_trace_span(&self, id: WidgetId) -> tracing::Span {
        trace_span!("RoundProgress", id = id.trace())
    }
}
