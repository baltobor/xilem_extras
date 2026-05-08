//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Hexagonal busy / indeterminate indicator.
//!
//! Seven hexagons arranged in a beehive cluster — one centre cell
//! plus six neighbours sharing edges with it. Each cell breathes
//! its alpha on a sine cycle with a phase offset around the ring,
//! so the cluster pulses like a wave running around the comb.
//!
//! Replaces the rotating-arc spinner that previously inhabited
//! `round_progress.busy()`. The round widget stays determinate
//! only; busy state moves out into its own widget so the two
//! shapes can have independent visual languages.

use std::f64::consts::{PI, TAU};

use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::core::{
    AccessCtx, EventCtx, LayoutCtx, MeasureCtx, PaintCtx, PointerEvent, PropertiesMut,
    PropertiesRef, RegisterCtx, Update, UpdateCtx, Widget, WidgetId, WidgetMut,
};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Axis, BezPath, Point, Size};
use xilem::masonry::layout::LenReq;
use xilem::masonry::peniko::Fill;
use xilem::Color;

use smallvec::SmallVec;
use tracing::trace_span;

/// Two predefined sizes, mirroring [`RoundProgressSize`] so the
/// hex spinner slots in next to round progress without surprise.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BusyHexSize {
    #[default]
    Normal,
    /// Sized to sit inline with body text.
    Small,
}

impl BusyHexSize {
    fn cell_radius(self) -> f64 {
        match self {
            BusyHexSize::Normal => 6.5,
            BusyHexSize::Small => 2.6,
        }
    }
    /// Outer bounding box side. Three cell radii covers the
    /// neighbour offset (`r * √3`) plus the neighbour's own
    /// circumradius `r`, with a hairline of antialiasing padding.
    fn box_side(self) -> f64 {
        let r = self.cell_radius();
        // Two neighbours opposite each other span 2*(r*√3) centre
        // distance plus 2r outer reach — i.e. 2r(1 + √3). Add a
        // small padding so the stroke / fill doesn't clip.
        2.0 * r * (1.0 + 3.0_f64.sqrt()) + 4.0
    }
}

/// Honey-amber default tint — reads as "comb" without needing a
/// caller decision. The widget tints all seven cells from this
/// colour; only the per-cell alpha animates.
const DEFAULT_TINT: Color = Color::from_rgb8(0xE0, 0xA8, 0x30);
/// Period of the breathing cycle in seconds. Chosen long enough
/// that the wave reads as "patience" rather than "loading bar".
const BREATH_PERIOD_SEC: f64 = 1.6;
/// Minimum alpha for a cell at the trough of its breath. Keeping
/// it above zero stops the cluster from disappearing entirely
/// during the wave's quiet phase, so the shape remains visible
/// as a passive "still working" cue.
const ALPHA_MIN: f32 = 0.18;
/// Maximum alpha for a cell at the peak of its breath.
const ALPHA_MAX: f32 = 0.95;

/// Hexagonal busy indicator widget.
pub struct BusyHexWidget {
    size: BusyHexSize,
    tint: Color,
    /// Accumulated breath phase in radians, advanced each frame.
    phase: f64,
    /// When `false`, the widget stops animating. If
    /// `hide_when_not_busy` is also `true`, it disappears from
    /// the layout entirely (zero footprint); otherwise it stays
    /// visible at a steady dim state.
    busy: bool,
    /// When `true`, the widget reports a zero size and skips
    /// painting whenever `busy` is `false` — useful when the
    /// indicator should appear only while a task is running.
    hide_when_not_busy: bool,
}

impl BusyHexWidget {
    pub fn new(size: BusyHexSize) -> Self {
        Self {
            size,
            tint: DEFAULT_TINT,
            phase: 0.0,
            busy: true,
            hide_when_not_busy: false,
        }
    }

    pub fn with_tint(mut self, color: Color) -> Self {
        self.tint = color;
        self
    }

    pub fn with_busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    pub fn with_hide_when_not_busy(mut self, hide: bool) -> Self {
        self.hide_when_not_busy = hide;
        self
    }

    pub fn set_size(this: &mut WidgetMut<'_, Self>, size: BusyHexSize) {
        if this.widget.size != size {
            this.widget.size = size;
            this.ctx.request_layout();
            this.ctx.request_render();
        }
    }

    pub fn set_tint(this: &mut WidgetMut<'_, Self>, color: Color) {
        this.widget.tint = color;
        this.ctx.request_render();
    }

    /// Toggle the busy state. Turning busy on kicks off a fresh
    /// animation tick; turning it off lets the cycle settle and,
    /// if `hide_when_not_busy`, releases the layout slot.
    pub fn set_busy(this: &mut WidgetMut<'_, Self>, busy: bool) {
        if this.widget.busy != busy {
            this.widget.busy = busy;
            if busy {
                this.ctx.request_anim_frame();
            }
            // Layout footprint flips whenever busy changes and
            // we hide on idle, so a layout pass is needed in
            // either direction.
            if this.widget.hide_when_not_busy {
                this.ctx.request_layout();
            }
            this.ctx.request_render();
        }
    }

    pub fn set_hide_when_not_busy(this: &mut WidgetMut<'_, Self>, hide: bool) {
        if this.widget.hide_when_not_busy != hide {
            this.widget.hide_when_not_busy = hide;
            this.ctx.request_layout();
            this.ctx.request_render();
        }
    }

    /// Build a regular pointy-top hexagon at `(cx, cy)` with
    /// circumradius `r`.
    fn hex_path(cx: f64, cy: f64, r: f64) -> BezPath {
        let mut p = BezPath::new();
        for i in 0..6 {
            // Pointy-top: first vertex at the top (-π/2), step
            // 60° clockwise.
            let theta = -PI / 2.0 + i as f64 * (PI / 3.0);
            let x = cx + r * theta.cos();
            let y = cy + r * theta.sin();
            if i == 0 {
                p.move_to(Point::new(x, y));
            } else {
                p.line_to(Point::new(x, y));
            }
        }
        p.close_path();
        p
    }

    /// Centres of the seven cells: one at the origin plus six
    /// neighbours sharing edges with it. Pointy-top hexagons place
    /// their neighbours at angles 30°, 90°, …, 330° from the
    /// centre, at a distance of `r * √3` (the inradius doubled).
    fn cell_offsets(r: f64) -> [(f64, f64); 7] {
        let d = r * 3.0_f64.sqrt();
        let mut offsets = [(0.0, 0.0); 7];
        for i in 0..6 {
            let theta = PI / 6.0 + i as f64 * (PI / 3.0);
            offsets[i + 1] = (d * theta.cos(), d * theta.sin());
        }
        offsets
    }
}

impl Widget for BusyHexWidget {
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

    fn update(&mut self, ctx: &mut UpdateCtx<'_>, _: &mut PropertiesMut<'_>, event: &Update) {
        if matches!(event, Update::WidgetAdded) && self.busy {
            // Kick the animation cycle on first add.
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
            // Let the cycle die when busy is off; restart on the
            // next `set_busy(true)`.
            return;
        }
        // Advance the breath phase. `interval` is nanoseconds
        // since the last frame; converting to a seconds-fraction
        // of `BREATH_PERIOD_SEC` keeps the cycle wall-clock-stable
        // regardless of frame rate.
        let dt_secs = interval as f64 / 1_000_000_000.0;
        self.phase = (self.phase + dt_secs * TAU / BREATH_PERIOD_SEC) % TAU;
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
        if !self.busy && self.hide_when_not_busy {
            0.0
        } else {
            self.size.box_side()
        }
    }

    fn layout(&mut self, _: &mut LayoutCtx<'_>, _: &PropertiesRef<'_>, _: Size) {}

    fn paint(
        &mut self,
        ctx: &mut PaintCtx<'_>,
        _: &PropertiesRef<'_>,
        painter: &mut Painter<'_>,
    ) {
        if !self.busy && self.hide_when_not_busy {
            return;
        }

        let size = ctx.content_box_size();
        let cx = size.width / 2.0;
        let cy = size.height / 2.0;
        let r = self.size.cell_radius();
        let offsets = Self::cell_offsets(r);

        // Per-cell phase offset distributes the wave around the
        // seven cells. Subtracting the offset (rather than adding)
        // makes the peak march from cell 1 → 2 → 3 …, which on
        // screen — where y grows downward and the neighbours are
        // laid out at 30°, 90°, 150° … — reads as a clockwise
        // sweep. When idle we still render the cells but at the
        // trough alpha so the shape stays visible without motion.
        let static_alpha = ALPHA_MIN;
        for (i, (dx, dy)) in offsets.iter().enumerate() {
            let alpha = if self.busy {
                let local_phase = self.phase - (i as f64) * (TAU / 7.0);
                let wave = (local_phase.sin() + 1.0) * 0.5; // 0..1
                ALPHA_MIN + (ALPHA_MAX - ALPHA_MIN) * wave as f32
            } else {
                static_alpha
            };

            let cell_color = self.tint.with_alpha(alpha);
            let path = Self::hex_path(cx + dx, cy + dy, r * 0.92);
            painter.fill(path, cell_color).fill_rule(Fill::NonZero).draw();
        }
    }

    fn accessibility_role(&self) -> Role {
        Role::ProgressIndicator
    }

    fn accessibility(&mut self, _: &mut AccessCtx<'_>, _: &PropertiesRef<'_>, node: &mut Node) {
        node.set_label("Busy");
    }

    fn children_ids(&self) -> SmallVec<[WidgetId; 16]> {
        SmallVec::new()
    }

    fn make_trace_span(&self, id: WidgetId) -> tracing::Span {
        trace_span!("BusyHex", id = id.trace())
    }
}
