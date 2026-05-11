//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Group box container with an embossed header label.
//!
//! Adapted from xilem_synth_widgets' GroupBox: a rounded rectangle
//! framing arbitrary content, with a small label drawn in the top
//! padding strip. The label colour is derived from the background
//! using APCA perceptual contrast — the caller picks any tint and
//! the header stays readable without manual colour matching.
//!
//! Forms a natural building block for `form_section` and any
//! settings/inspector panel that needs visual grouping.

use std::any::TypeId;

use xilem::core::{MessageCtx, Mut, View, ViewMarker, ViewPathTracker, ViewId};
use xilem::core::MessageResult;
use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::core::{
    AccessCtx, BrushIndex, ChildrenIds, EventCtx, LayoutCtx, MeasureCtx, NewWidget, PaintCtx,
    PointerEvent, PropertiesMut, PropertiesRef, RegisterCtx, StyleProperty, Update, UpdateCtx,
    Widget, WidgetId, WidgetMut, WidgetPod, render_text,
};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Affine, Axis, Point, Rect, RoundedRect, Size, Stroke, Vec2};
use xilem::masonry::layout::{LenReq, Length};
use xilem::masonry::parley::Layout;
use xilem::masonry::peniko::{Color, Fill};
use xilem::{Pod, ViewCtx, WidgetView};

use tracing::{Span, trace_span};

const LABEL_HEIGHT: f64 = 16.0;
const PADDING: f64 = 8.0;
const BORDER_WIDTH: f64 = 0.5;
const CORNER_RADIUS: f64 = 6.0;
const LABEL_FONT_SIZE: f32 = 10.0;

const CHILD_VIEW_ID: ViewId = ViewId::new(0);

/// Default neutral background that suits dark themes.
const DEFAULT_BG: Color = Color::from_rgb8(0x2A, 0x28, 0x25);

// MARK: - APCA contrast helpers (adopted from xilem_synth_widgets)

fn color_rgb(c: Color) -> (u8, u8, u8) {
    let rgba = c.to_rgba8();
    (rgba.r, rgba.g, rgba.b)
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (f64, f64, f64) {
    if s == 0.0 {
        return (l, l, l);
    }
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let hue_to_rgb = |t: f64| {
        let t = ((t % 1.0) + 1.0) % 1.0;
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };
    (hue_to_rgb(h + 1.0 / 3.0), hue_to_rgb(h), hue_to_rgb(h - 1.0 / 3.0))
}

fn srgb_to_y(r: u8, g: u8, b: u8) -> f64 {
    const MAIN_TRC: f64 = 2.4;
    const SR_CO: f64 = 0.2126729;
    const SG_CO: f64 = 0.7151522;
    const SB_CO: f64 = 0.0721750;
    let lin = |c: u8| (c as f64 / 255.0).powf(MAIN_TRC);
    SR_CO * lin(r) + SG_CO * lin(g) + SB_CO * lin(b)
}

fn apca_contrast(txt_y: f64, bg_y: f64) -> f64 {
    const BLK_THRS: f64 = 0.022;
    const BLK_CLMP: f64 = 1.414;
    const NORM_BG: f64 = 0.56;
    const NORM_TXT: f64 = 0.57;
    const REV_TXT: f64 = 0.62;
    const REV_BG: f64 = 0.65;
    const SCALE_BOW: f64 = 1.14;
    const SCALE_WOB: f64 = 1.14;
    const LO_BOW_OFFSET: f64 = 0.027;
    const LO_WOB_OFFSET: f64 = 0.027;
    const DELTA_Y_MIN: f64 = 0.0005;
    const LO_CLIP: f64 = 0.1;

    let ty = if txt_y > BLK_THRS { txt_y } else { txt_y + (BLK_THRS - txt_y).powf(BLK_CLMP) };
    let by = if bg_y > BLK_THRS { bg_y } else { bg_y + (BLK_THRS - bg_y).powf(BLK_CLMP) };

    if (by - ty).abs() < DELTA_Y_MIN { return 0.0; }

    if by > ty {
        let sapc = (by.powf(NORM_BG) - ty.powf(NORM_TXT)) * SCALE_BOW;
        if sapc < LO_CLIP { 0.0 } else { (sapc - LO_BOW_OFFSET) * 100.0 }
    } else {
        let sapc = (by.powf(REV_BG) - ty.powf(REV_TXT)) * SCALE_WOB;
        if sapc > -LO_CLIP { 0.0 } else { (sapc + LO_WOB_OFFSET) * 100.0 }
    }
}

/// Pick a label colour that reads cleanly on top of `bg`. Uses
/// hue rotation plus an APCA contrast verification step that
/// boosts lightness if the first candidate is too close.
pub fn inverse_contrast_color(bg: Color) -> Color {
    let (r8, g8, b8) = color_rgb(bg);
    let r = r8 as f64 / 255.0;
    let g = g8 as f64 / 255.0;
    let b = b8 as f64 / 255.0;

    let min = r.min(g).min(b);
    let max = r.max(g).max(b);
    let l = (min + max) / 2.0;

    let mut s = 0.0;
    if max > 0.0 || min > 0.0 {
        if l <= 0.5 {
            s = (max - min) / (max + min);
        } else {
            s = (max - min) / (2.0 - max - min);
        }
    }

    let mut h = 0.0;
    if max != min {
        if max == r {
            h = (g - b) / (max - min);
        } else if max == g {
            h = 2.0 + (b - r) / (max - min);
        } else {
            h = 4.0 + (r - g) / (max - min);
        }
    }

    let h_deg = h * 60.0;
    let h2 = ((h_deg + 180.0) % 360.0) / 360.0;

    let contrast = 0.6;
    let mut l2 = (l * (1.0 - contrast)) / (contrast + 1.0);
    if l < 0.382 && (l - l2).abs() < 0.382 {
        l2 = 1.0 - l2;
        if l2 < 0.5 { l2 = 0.5; }
    }
    l2 = l2.min(0.55);

    if s > 0.5 {
        s = 1.0 - (s * (1.0 - 0.141592653589));
        s *= 0.9;
    } else if s > 0.15 {
        s = (1.0 - s) * 0.9;
    } else {
        s *= 0.5;
    }

    let (ro, go, bo) = hsl_to_rgb(h2, s, l2);
    let to_u8 = |v: f64| (v * 255.0).round().clamp(0.0, 255.0) as u8;
    let (cr, cg, cb) = (to_u8(ro), to_u8(go), to_u8(bo));

    let bg_y = srgb_to_y(r8, g8, b8);
    let txt_y = srgb_to_y(cr, cg, cb);
    let lc = apca_contrast(txt_y, bg_y);

    if lc.abs() < 60.0 {
        let mut adj_l = l2;
        for _ in 0..20 {
            adj_l = (adj_l + 0.05).min(1.0);
            let (ar, ag, ab) = hsl_to_rgb(h2, s, adj_l);
            let (tr, tg, tb) = (to_u8(ar), to_u8(ag), to_u8(ab));
            let adj_lc = apca_contrast(srgb_to_y(tr, tg, tb), bg_y);
            if adj_lc.abs() >= 60.0 {
                return Color::from_rgb8(tr, tg, tb);
            }
        }
        let white_lc = apca_contrast(srgb_to_y(255, 255, 255), bg_y);
        if white_lc.abs() > lc.abs() {
            return Color::from_rgb8(0xEE, 0xEE, 0xEE);
        } else {
            return Color::from_rgb8(0x11, 0x11, 0x11);
        }
    }

    Color::from_rgb8(cr, cg, cb)
}

fn border_from_tint(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgba8(
        (r as u16 + (255 - r as u16) * 40 / 100) as u8,
        (g as u16 + (255 - g as u16) * 40 / 100) as u8,
        (b as u16 + (255 - b as u16) * 40 / 100) as u8,
        0x80,
    )
}

// MARK: - GroupBox widget

/// Group box: rounded frame with header label that auto-picks
/// a readable colour from the background tint.
pub struct GroupBox {
    child: WidgetPod<dyn Widget>,
    label: String,
    bg_color: Color,
    border_color: Color,
    text_layout: Layout<BrushIndex>,
    needs_layout: bool,
}

impl GroupBox {
    pub fn new(label: impl Into<String>, child: NewWidget<impl Widget + ?Sized>) -> Self {
        let (r, g, b) = color_rgb(DEFAULT_BG);
        Self {
            child: child.erased().to_pod(),
            label: label.into(),
            bg_color: DEFAULT_BG,
            border_color: border_from_tint(r, g, b),
            text_layout: Layout::new(),
            needs_layout: true,
        }
    }

    pub fn with_tint(mut self, color: Color) -> Self {
        self.bg_color = color;
        let (r, g, b) = color_rgb(color);
        self.border_color = border_from_tint(r, g, b);
        self
    }

    pub fn child_mut<'t>(this: &'t mut WidgetMut<'_, Self>) -> WidgetMut<'t, dyn Widget> {
        this.ctx.get_mut(&mut this.widget.child)
    }

    pub fn set_label(this: &mut WidgetMut<'_, Self>, label: impl Into<String>) {
        this.widget.label = label.into();
        this.widget.needs_layout = true;
        this.ctx.request_layout();
    }

    pub fn set_tint(this: &mut WidgetMut<'_, Self>, color: Color) {
        this.widget.bg_color = color;
        let (r, g, b) = color_rgb(color);
        this.widget.border_color = border_from_tint(r, g, b);
        this.ctx.request_render();
    }

    fn ensure_text_layout(
        &mut self,
        (font_ctx, layout_ctx): (
            &mut xilem::masonry::parley::FontContext,
            &mut xilem::masonry::parley::LayoutContext<BrushIndex>,
        ),
    ) {
        if self.needs_layout {
            let mut builder =
                layout_ctx.ranged_builder(font_ctx, &self.label, 1.0, true);
            builder.push_default(StyleProperty::FontSize(LABEL_FONT_SIZE));
            builder.push_default(StyleProperty::Brush(BrushIndex(0)));
            builder.build_into(&mut self.text_layout, &self.label);
            self.text_layout.break_all_lines(None);
            self.needs_layout = false;
        }
    }
}

impl Widget for GroupBox {
    type Action = ();

    fn on_pointer_event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &PointerEvent,
    ) {
    }

    fn register_children(&mut self, ctx: &mut RegisterCtx<'_>) {
        ctx.register_child(&mut self.child);
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &Update,
    ) {
    }

    fn property_changed(&mut self, _ctx: &mut UpdateCtx<'_>, _property_type: TypeId) {}

    fn measure(
        &mut self,
        ctx: &mut MeasureCtx<'_>,
        _props: &PropertiesRef<'_>,
        axis: Axis,
        _len_req: LenReq,
        cross_length: Option<Length>,
    ) -> Length {
        self.ensure_text_layout(ctx.text_contexts());

        match axis {
            Axis::Horizontal => {
                let child_cross = cross_length
                    .map(|c| Length::px((c.get() - LABEL_HEIGHT - PADDING * 2.0).max(0.0)));
                let child_w = ctx.redirect_measurement(&mut self.child, axis, child_cross);
                Length::px(child_w.get() + PADDING * 2.0)
            }
            Axis::Vertical => {
                let child_cross =
                    cross_length.map(|c| Length::px((c.get() - PADDING * 2.0).max(0.0)));
                let child_h = ctx.redirect_measurement(&mut self.child, axis, child_cross);
                Length::px(child_h.get() + LABEL_HEIGHT + PADDING * 2.0)
            }
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, size: Size) {
        self.ensure_text_layout(ctx.text_contexts());

        let child_w = (size.width - PADDING * 2.0).max(0.0);
        let child_h = (size.height - LABEL_HEIGHT - PADDING * 2.0).max(0.0);
        let child_size = Size::new(child_w, child_h);
        ctx.run_layout(&mut self.child, child_size);
        ctx.place_child(&mut self.child, Point::new(PADDING, LABEL_HEIGHT + PADDING));
    }

    fn paint(
        &mut self,
        ctx: &mut PaintCtx<'_>,
        _props: &PropertiesRef<'_>,
        painter: &mut Painter<'_>,
    ) {
        let size = ctx.content_box_size();
        let rect = Rect::from_origin_size(Point::ZERO, size);
        let rr = RoundedRect::from_rect(rect, CORNER_RADIUS);

        painter.fill(rr, self.bg_color).fill_rule(Fill::NonZero).draw();
        painter.stroke(rr, &Stroke::new(BORDER_WIDTH), self.border_color).draw();

        let label_color = inverse_contrast_color(self.bg_color);
        let text_h = self.text_layout.height() as f64;
        let text_y = (LABEL_HEIGHT - text_h) / 2.0;
        render_text(
            painter,
            Affine::translate(Vec2::new(PADDING, text_y)),
            &self.text_layout,
            &[label_color.into()],
            true,
        );
    }

    fn accessibility_role(&self) -> Role {
        Role::Group
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        node: &mut Node,
    ) {
        node.set_label(self.label.clone());
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::from_slice(&[self.child.id()])
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("GroupBox", id = id.trace())
    }
}

// MARK: - View

/// Xilem view for a [`GroupBox`].
pub struct GroupBoxView<V> {
    label: String,
    child: V,
    tint: Option<Color>,
}

/// Wrap a child in a labeled [`GroupBox`]. The label colour is
/// derived from the tint via APCA contrast — pick any background
/// and the header stays readable.
///
/// # Example
///
/// ```ignore
/// use xilem_extras::group_box;
/// use xilem::view::flex_col;
///
/// group_box("Notifications", flex_col((row1, row2, row3)))
/// ```
pub fn group_box<State, Action, V>(
    label: impl Into<String>,
    child: V,
) -> GroupBoxView<V>
where
    State: 'static,
    Action: 'static,
    V: WidgetView<State, Action>,
{
    GroupBoxView {
        label: label.into(),
        child,
        tint: None,
    }
}

impl<V> GroupBoxView<V> {
    /// Override the background tint. The label colour follows
    /// automatically.
    pub fn tint(mut self, color: Color) -> Self {
        self.tint = Some(color);
        self
    }
}

impl<V> ViewMarker for GroupBoxView<V> {}

impl<V, State, Action> View<State, Action, ViewCtx> for GroupBoxView<V>
where
    V: WidgetView<State, Action>,
    State: 'static,
    Action: 'static,
{
    type Element = Pod<GroupBox>;
    type ViewState = V::ViewState;

    fn build(
        &self,
        ctx: &mut ViewCtx,
        app_state: &mut State,
    ) -> (Self::Element, Self::ViewState) {
        let (child_pod, child_state) = ctx.with_id(CHILD_VIEW_ID, |ctx| {
            self.child.build(ctx, app_state)
        });
        let pod = ctx.with_action_widget(|ctx| {
            let mut widget = GroupBox::new(self.label.clone(), child_pod.new_widget);
            if let Some(tint) = self.tint {
                widget = widget.with_tint(tint);
            }
            ctx.create_pod(widget)
        });
        (pod, child_state)
    }

    fn rebuild(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) {
        if prev.label != self.label {
            GroupBox::set_label(&mut element, self.label.clone());
        }
        if prev.tint != self.tint {
            GroupBox::set_tint(&mut element, self.tint.unwrap_or(DEFAULT_BG));
        }
        ctx.with_id(CHILD_VIEW_ID, |ctx| {
            self.child.rebuild(
                &prev.child,
                view_state,
                ctx,
                GroupBox::child_mut(&mut element).downcast(),
                app_state,
            );
        });
    }

    fn teardown(
        &self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
    ) {
        ctx.with_id(CHILD_VIEW_ID, |ctx| {
            self.child.teardown(
                view_state,
                ctx,
                GroupBox::child_mut(&mut element).downcast(),
            );
        });
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        message: &mut MessageCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) -> MessageResult<Action> {
        match message.take_first() {
            Some(CHILD_VIEW_ID) => self.child.message(
                view_state,
                message,
                GroupBox::child_mut(&mut element).downcast(),
                app_state,
            ),
            _ => MessageResult::Stale,
        }
    }
}
