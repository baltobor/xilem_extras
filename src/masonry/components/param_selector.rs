//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Parameter selector widget.

use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::core::{
    AccessCtx, BrushIndex, EventCtx, LayoutCtx, MeasureCtx, PaintCtx, PointerButtonEvent,
    PointerEvent, PropertiesMut, PropertiesRef, RegisterCtx, StyleProperty, Update, UpdateCtx,
    Widget, WidgetId, WidgetMut, render_text,
};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Affine, Axis, Circle, Point, Rect, RoundedRect, Size, Stroke, Vec2};
use xilem::masonry::layout::{LenReq, Length};
use xilem::masonry::parley::Layout;
use xilem::masonry::peniko::{Color, Fill};

use smallvec::SmallVec;
use tracing::trace_span;

const ROW_HEIGHT: f64 = 16.0;
const DOT_RADIUS: f64 = 4.0;
const DOT_MARGIN: f64 = 2.0;
const LABEL_GAP: f64 = 4.0;
const FONT_SIZE: f32 = 11.0;

const DEFAULT_TINT: Color = Color::from_rgb8(0xEE, 0xE6, 0xD8);
const FRAME_R: f64 = 6.0;

/// How the control + text assembly is laid out within the widget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelAlign {
    Left,
    Right,
    Alternating,
}

const DEFAULT_LABEL_SELECTED: Color = Color::from_rgb8(0xEE, 0xEE, 0xEE);
const DEFAULT_LABEL_UNSELECTED: Color = Color::from_rgb8(0x99, 0x99, 0x99);

pub struct ParamSelectorWidget {
    selected: usize,
    count: usize,
    labels: Vec<String>,
    label_align: LabelAlign,
    tint: Color,
    label_selected: Color,
    label_unselected: Color,
    text_layouts: Vec<Layout<BrushIndex>>,
    needs_layout: bool,
}

impl ParamSelectorWidget {
    pub fn new(labels: Vec<String>, selected: usize, label_align: LabelAlign) -> Self {
        let count = labels.len();
        Self {
            selected: selected.min(count.saturating_sub(1)),
            count,
            labels,
            label_align,
            tint: DEFAULT_TINT,
            label_selected: DEFAULT_LABEL_SELECTED,
            label_unselected: DEFAULT_LABEL_UNSELECTED,
            text_layouts: Vec::new(),
            needs_layout: true,
        }
    }

    pub fn with_tint(mut self, color: Color) -> Self {
        self.tint = color;
        self
    }

    pub fn with_label_colors(mut self, selected: Color, unselected: Color) -> Self {
        self.label_selected = selected;
        self.label_unselected = unselected;
        self
    }

    pub fn set_label_colors(this: &mut WidgetMut<'_, Self>, selected: Color, unselected: Color) {
        if this.widget.label_selected != selected || this.widget.label_unselected != unselected {
            this.widget.label_selected = selected;
            this.widget.label_unselected = unselected;
            this.ctx.request_render();
        }
    }

    pub fn set_selected(this: &mut WidgetMut<'_, Self>, selected: usize) {
        let s = selected.min(this.widget.count.saturating_sub(1));
        if this.widget.selected != s {
            this.widget.selected = s;
            this.ctx.request_render();
        }
    }

    pub fn set_labels(this: &mut WidgetMut<'_, Self>, labels: Vec<String>) {
        this.widget.count = labels.len();
        this.widget.labels = labels;
        this.widget.selected = this
            .widget
            .selected
            .min(this.widget.count.saturating_sub(1));
        this.widget.needs_layout = true;
        this.ctx.request_layout();
    }

    pub fn set_tint(this: &mut WidgetMut<'_, Self>, color: Color) {
        this.widget.tint = color;
        this.ctx.request_render();
    }

    pub fn set_label_align(this: &mut WidgetMut<'_, Self>, align: LabelAlign) {
        if this.widget.label_align != align {
            this.widget.label_align = align;
            this.ctx.request_render();
        }
    }

    fn row_rect(&self, index: usize, size: Size) -> (f64, f64) {
        let y = index as f64 * ROW_HEIGHT;
        (y, (y + ROW_HEIGHT).min(size.height))
    }

    fn hit_test(&self, pos: Point, size: Size) -> Option<usize> {
        for i in 0..self.count {
            let (y0, y1) = self.row_rect(i, size);
            if pos.y >= y0 && pos.y < y1 && pos.x >= 0.0 && pos.x <= size.width {
                return Some(i);
            }
        }
        None
    }

    fn dot_col_w() -> f64 {
        DOT_RADIUS * 2.0 + DOT_MARGIN * 2.0
    }

    fn ensure_text_layouts(
        &mut self,
        (font_ctx, layout_ctx): (
            &mut xilem::masonry::parley::FontContext,
            &mut xilem::masonry::parley::LayoutContext<BrushIndex>,
        ),
    ) {
        if self.needs_layout || self.text_layouts.len() != self.labels.len() {
            self.text_layouts.clear();
            for label in &self.labels {
                let mut builder = layout_ctx.ranged_builder(font_ctx, label, 1.0, true);
                builder.push_default(StyleProperty::FontSize(FONT_SIZE));
                let mut layout = builder.build(label);
                layout.break_all_lines(None);
                self.text_layouts.push(layout);
            }
            self.needs_layout = false;
        }
    }
}

impl Widget for ParamSelectorWidget {
    type Action = usize;

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        if ctx.is_disabled() {
            return;
        }
        if let PointerEvent::Up(PointerButtonEvent { state, .. }) = event {
            let pos = ctx.local_position(state.position);
            if let Some(idx) = self.hit_test(pos, ctx.content_box().size()) {
                if self.selected != idx {
                    self.selected = idx;
                    ctx.submit_action::<usize>(idx);
                    ctx.request_render();
                }
            }
        }
    }

    fn accepts_pointer_interaction(&self) -> bool {
        true
    }

    fn register_children(&mut self, _ctx: &mut RegisterCtx<'_>) {}

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &Update,
    ) {
    }

    fn measure(
        &mut self,
        ctx: &mut MeasureCtx<'_>,
        _props: &PropertiesRef<'_>,
        axis: Axis,
        _len_req: LenReq,
        _cross_length: Option<Length>,
    ) -> Length {
        self.ensure_text_layouts(ctx.text_contexts());

        match axis {
            Axis::Horizontal => {
                let dot_col_w = Self::dot_col_w();
                let width = match self.label_align {
                    LabelAlign::Left | LabelAlign::Right => {
                        let max_text_w = self
                            .text_layouts
                            .iter()
                            .map(|l| l.width() as f64)
                            .fold(0.0_f64, f64::max);
                        max_text_w + dot_col_w + LABEL_GAP
                    }
                    LabelAlign::Alternating => {
                        let max_left_w = self
                            .text_layouts
                            .iter()
                            .enumerate()
                            .filter(|(i, _)| i % 2 == 0)
                            .map(|(_, l)| l.width() as f64)
                            .fold(0.0_f64, f64::max);
                        let max_right_w = self
                            .text_layouts
                            .iter()
                            .enumerate()
                            .filter(|(i, _)| i % 2 == 1)
                            .map(|(_, l)| l.width() as f64)
                            .fold(0.0_f64, f64::max);
                        max_left_w + LABEL_GAP + dot_col_w + LABEL_GAP + max_right_w
                    }
                };
                Length::px(width)
            }
            Axis::Vertical => Length::px(self.count as f64 * ROW_HEIGHT),
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, _size: Size) {
        self.ensure_text_layouts(ctx.text_contexts());
    }

    fn paint(
        &mut self,
        ctx: &mut PaintCtx<'_>,
        _props: &PropertiesRef<'_>,
        painter: &mut Painter<'_>,
    ) {
        let size = ctx.content_box().size();
        let dot_col_w = Self::dot_col_w();

        let frame_pad = 2.0;
        let frame_w = DOT_RADIUS * 2.0 + 6.0;
        let dot_center_x = match self.label_align {
            LabelAlign::Left => dot_col_w / 2.0,
            LabelAlign::Right => size.width - dot_col_w / 2.0,
            LabelAlign::Alternating => size.width / 2.0,
        };
        let frame_rect = Rect::new(
            dot_center_x - frame_w / 2.0,
            frame_pad,
            dot_center_x + frame_w / 2.0,
            size.height - frame_pad,
        );
        let frame_rr = RoundedRect::from_rect(frame_rect, FRAME_R);
        painter
            .fill(frame_rr, Color::from_rgb8(0x2A, 0x2A, 0x2A))
            .fill_rule(Fill::NonZero)
            .draw();
        painter
            .stroke(
                frame_rr,
                &Stroke::new(1.0),
                Color::from_rgb8(0x55, 0x55, 0x55),
            )
            .draw();

        for i in 0..self.count {
            let (y0, _) = self.row_rect(i, size);
            let cy = y0 + ROW_HEIGHT / 2.0;
            let is_selected = i == self.selected;

            let center = Point::new(dot_center_x, cy);
            if is_selected {
                let dot = Circle::new(center, DOT_RADIUS + 1.5);
                painter.fill(dot, self.tint).fill_rule(Fill::NonZero).draw();
            }

            if let Some(layout) = self.text_layouts.get(i) {
                let text_color = if is_selected {
                    self.label_selected
                } else {
                    self.label_unselected
                };

                let text_w = layout.width() as f64;
                let text_h = layout.height() as f64;

                let text_x = match self.label_align {
                    LabelAlign::Left => dot_col_w + LABEL_GAP,
                    LabelAlign::Right => size.width - dot_col_w - LABEL_GAP - text_w,
                    LabelAlign::Alternating => {
                        if i % 2 == 0 {
                            dot_center_x - dot_col_w / 2.0 - LABEL_GAP - text_w
                        } else {
                            dot_center_x + dot_col_w / 2.0 + LABEL_GAP
                        }
                    }
                };
                let text_y = cy - text_h / 2.0;

                render_text(
                    painter,
                    Affine::translate(Vec2::new(text_x, text_y)),
                    layout,
                    &[text_color.into()],
                    true,
                );
            }
        }
    }

    fn accessibility_role(&self) -> Role {
        Role::RadioGroup
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        node: &mut Node,
    ) {
        if self.selected < self.labels.len() {
            node.set_description(self.labels[self.selected].clone());
        }
    }

    fn children_ids(&self) -> SmallVec<[WidgetId; 16]> {
        SmallVec::new()
    }

    fn make_trace_span(&self, id: WidgetId) -> tracing::Span {
        trace_span!("ParamSelector", id = id.trace())
    }
}
