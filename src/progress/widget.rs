//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Linear progress-bar widget.
//!
//! A non-interactive bar that fills proportional to its
//! current value within `min..max`. Three render styles:
//!
//! - **Gradient** — three coloured zones (green / orange / red),
//!   useful for level / health metrics where higher values are
//!   "worse". Zone thresholds at 75 % and 90 %.
//! - **Tint** — single solid colour interpolated green → orange
//!   → red across the same threshold bands. The bar's colour
//!   reflects its level continuously rather than by zone.
//! - **Monochrome** — single fixed colour regardless of level.
//!   The "boring" variant for plain progress where the value
//!   itself isn't a quality signal (e.g. download bars).
//!
//! Linear-only — no decibel or logarithmic scale. Use the
//! caller's domain (e.g. SOLID score 0..1, file count 0..N) as
//! the value range directly.

use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::core::{
    AccessCtx, EventCtx, LayoutCtx, MeasureCtx, PaintCtx, PointerEvent, PropertiesMut,
    PropertiesRef, RegisterCtx, Update, UpdateCtx, Widget, WidgetId, WidgetMut,
};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Axis, Rect, Size};
use xilem::masonry::layout::LenReq;
use xilem::masonry::peniko::Fill;
use xilem::Color;

use smallvec::SmallVec;
use tracing::trace_span;

/// Orientation of the bar.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProgressOrientation {
    Horizontal,
    Vertical,
}

/// Visual style for the bar.
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum ProgressStyle {
    /// Three-zone coloured bar (green / orange / red). Use for
    /// signals where "high" is bad — level meters, health bars.
    #[default]
    Gradient,
    /// Single solid colour interpolated through the same
    /// threshold bands as `Gradient`. The bar is a uniform
    /// colour that changes with level.
    Tint,
    /// Single fixed colour, no interpolation. The "plain
    /// progress bar" — a download bar, a file-count indicator.
    /// The colour comes from `with_tint(Color)`; defaults to
    /// the constant defined in this module.
    Monochrome,
}

const DEFAULT_WIDTH: f64 = 120.0;
const DEFAULT_HEIGHT: f64 = 6.0;
const BG_COLOR: Color = Color::from_rgb8(0x20, 0x20, 0x20);
const GREEN: Color = Color::from_rgb8(0x30, 0xC0, 0x30);
const ORANGE: Color = Color::from_rgb8(0xFF, 0x8C, 0x00);
const RED: Color = Color::from_rgb8(0xE0, 0x20, 0x20);
/// Default tint for `Monochrome` mode when the caller doesn't
/// override.
const DEFAULT_MONO_TINT: Color = Color::from_rgb8(0x4A, 0x9E, 0xFF);

/// Threshold (as a normalised fraction of `min..max`) at which
/// gradient/tint mode transitions from green into orange.
const ZONE_LOW: f64 = 0.75;
/// Threshold at which gradient/tint mode transitions from
/// orange into red.
const ZONE_HIGH: f64 = 0.90;

/// Linear progress bar.
pub struct ProgressBarWidget {
    value: f64,
    min: f64,
    max: f64,
    orientation: ProgressOrientation,
    style: ProgressStyle,
    /// Tint colour used by `Monochrome` style. Ignored by other
    /// styles.
    mono_tint: Color,
    /// Outer bar size. Caller can override the default 120×6
    /// horizontal / 6×120 vertical via `with_size`.
    main_axis_len: f64,
    cross_axis_len: f64,
    /// Reverse the colour gradient.
    ///
    /// Default `false` (existing behaviour): high values are
    /// "bad" — the alarm zone (orange/red) sits at the high end
    /// of the bar. Suitable for level meters, load indicators,
    /// memory pressure, and similar where "more is worse".
    ///
    /// `true`: high values are "good" — green sits at the high
    /// end. Suitable for quality scores, completion percentages,
    /// confidence ratings, and similar where "more is better"
    /// (SOLID scores being the motivating case).
    ///
    /// Affects `Gradient` and `Tint` styles. `Monochrome` has no
    /// level-based colour mapping, so the flag is a no-op there
    /// (the field is stored on the struct anyway for API
    /// consistency).
    reversed: bool,
}

impl ProgressBarWidget {
    pub fn new(value: f64, min: f64, max: f64, orientation: ProgressOrientation) -> Self {
        Self {
            value,
            min,
            max,
            orientation,
            style: ProgressStyle::Gradient,
            mono_tint: DEFAULT_MONO_TINT,
            main_axis_len: DEFAULT_WIDTH,
            cross_axis_len: DEFAULT_HEIGHT,
            reversed: false,
        }
    }

    pub fn with_style(mut self, style: ProgressStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the tint colour used by `Monochrome` style. No effect
    /// on `Gradient` / `Tint`.
    pub fn with_tint(mut self, color: Color) -> Self {
        self.mono_tint = color;
        self
    }

    /// Override the bar's main-axis length (width when
    /// horizontal, height when vertical).
    pub fn with_main_axis_len(mut self, len: f64) -> Self {
        self.main_axis_len = len;
        self
    }

    /// Override the bar's cross-axis thickness.
    pub fn with_cross_axis_len(mut self, len: f64) -> Self {
        self.cross_axis_len = len;
        self
    }

    /// Reverse the colour gradient — high values become green,
    /// low values become red. See struct docs for full
    /// semantics. No-op on `Monochrome` style.
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
        this.widget.mono_tint = color;
        this.ctx.request_render();
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
    /// When `reversed`, the input is mirrored so 1.0 maps to
    /// the green end and 0.0 maps to the red end.
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

impl Widget for ProgressBarWidget {
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
    fn update(&mut self, _: &mut UpdateCtx<'_>, _: &mut PropertiesMut<'_>, _: &Update) {}

    fn measure(
        &mut self,
        _: &mut MeasureCtx<'_>,
        _: &PropertiesRef<'_>,
        axis: Axis,
        _: LenReq,
        _: Option<f64>,
    ) -> f64 {
        match (self.orientation, axis) {
            (ProgressOrientation::Horizontal, Axis::Horizontal) => self.main_axis_len,
            (ProgressOrientation::Horizontal, Axis::Vertical) => self.cross_axis_len,
            (ProgressOrientation::Vertical, Axis::Horizontal) => self.cross_axis_len,
            (ProgressOrientation::Vertical, Axis::Vertical) => self.main_axis_len,
        }
    }

    fn layout(&mut self, _: &mut LayoutCtx<'_>, _: &PropertiesRef<'_>, _: Size) {}

    fn paint(
        &mut self,
        ctx: &mut PaintCtx<'_>,
        _: &PropertiesRef<'_>,
        painter: &mut Painter<'_>,
    ) {
        let size = ctx.content_box_size();
        let norm = self.normalized();

        // Background — same dark fill regardless of style.
        let bg_rect = Rect::new(0.0, 0.0, size.width, size.height);
        painter.fill(&bg_rect, BG_COLOR).fill_rule(Fill::NonZero).draw();

        if norm < 0.001 {
            return;
        }

        match self.style {
            ProgressStyle::Monochrome => {
                paint_solid_fill(painter, &self.orientation, size, norm, self.mono_tint);
            }
            ProgressStyle::Tint => {
                let color = Self::interpolate_color(norm, self.reversed);
                paint_solid_fill(painter, &self.orientation, size, norm, color);
            }
            ProgressStyle::Gradient => {
                paint_gradient_zones(painter, &self.orientation, size, norm, self.reversed);
            }
        }
    }

    fn accessibility_role(&self) -> Role {
        Role::ProgressIndicator
    }
    fn accessibility(&mut self, _: &mut AccessCtx<'_>, _: &PropertiesRef<'_>, node: &mut Node) {
        node.set_numeric_value(self.value);
        node.set_min_numeric_value(self.min);
        node.set_max_numeric_value(self.max);
    }

    fn children_ids(&self) -> SmallVec<[WidgetId; 16]> {
        SmallVec::new()
    }

    fn make_trace_span(&self, id: WidgetId) -> tracing::Span {
        trace_span!("ProgressBar", id = id.trace())
    }
}

/// Paint a single-colour fill from origin to `norm * size`.
/// Used by `Monochrome` and `Tint` styles.
fn paint_solid_fill(
    painter: &mut Painter<'_>,
    orientation: &ProgressOrientation,
    size: Size,
    norm: f64,
    color: Color,
) {
    match orientation {
        ProgressOrientation::Horizontal => {
            let fill_w = norm * size.width;
            let r = Rect::new(0.0, 0.0, fill_w, size.height);
            painter.fill(&r, color).fill_rule(Fill::NonZero).draw();
        }
        ProgressOrientation::Vertical => {
            let fill_h = norm * size.height;
            let top = size.height - fill_h;
            let r = Rect::new(0.0, top, size.width, size.height);
            painter.fill(&r, color).fill_rule(Fill::NonZero).draw();
        }
    }
}

/// Paint the three-zone gradient. Vertical bars fill from the
/// bottom up; horizontal bars from the left.
///
/// `reversed = false` (default): zones along the value axis are
/// green (0..LOW), orange (LOW..HIGH), red (HIGH..1) — alarm
/// at the top.
///
/// `reversed = true`: zones flip — red (0..1−HIGH), orange
/// (1−HIGH..1−LOW), green (1−LOW..1) — alarm at the bottom.
fn paint_gradient_zones(
    painter: &mut Painter<'_>,
    orientation: &ProgressOrientation,
    size: Size,
    norm: f64,
    reversed: bool,
) {
    // Three zones along the [0..1] value axis as
    // (end_fraction, color) pairs, starting from the origin.
    // `last_end` is implicit at 0.0; each entry covers
    // `(prev_end .. end)`.
    let zones: [(f64, Color); 3] = if reversed {
        [
            (1.0 - ZONE_HIGH, RED),
            (1.0 - ZONE_LOW, ORANGE),
            (1.0, GREEN),
        ]
    } else {
        [
            (ZONE_LOW, GREEN),
            (ZONE_HIGH, ORANGE),
            (1.0, RED),
        ]
    };

    match orientation {
        ProgressOrientation::Horizontal => {
            let fill_w = norm * size.width;
            let mut prev = 0.0;
            for (end_frac, color) in zones {
                let zone_end = end_frac * size.width;
                let draw_left = prev;
                let draw_right = fill_w.min(zone_end);
                if draw_right > draw_left {
                    let r = Rect::new(draw_left, 0.0, draw_right, size.height);
                    painter.fill(&r, color).fill_rule(Fill::NonZero).draw();
                }
                if fill_w <= zone_end {
                    break;
                }
                prev = zone_end;
            }
        }
        ProgressOrientation::Vertical => {
            // Vertical bars fill bottom-up; the drawing
            // coordinate y grows downward, so we compute each
            // zone's vertical extent from the bottom of the
            // widget rect.
            let fill_h = norm * size.height;
            let top = size.height - fill_h;
            let mut prev_bottom = size.height;
            for (end_frac, color) in zones {
                let zone_top = size.height - end_frac * size.height;
                let draw_bottom = prev_bottom;
                let draw_top = top.max(zone_top);
                if draw_top < draw_bottom {
                    let r = Rect::new(0.0, draw_top, size.width, draw_bottom);
                    painter.fill(&r, color).fill_rule(Fill::NonZero).draw();
                }
                if top >= zone_top {
                    break;
                }
                prev_bottom = zone_top;
            }
        }
    }
}
