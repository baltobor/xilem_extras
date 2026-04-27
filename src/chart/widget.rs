//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Chart widget - renders bar and line charts.
//!
//! A masonry Widget that draws charts with:
//! - Y-axis with max value and 0 labels
//! - X-axis with custom labels
//! - Bar or line rendering
//! - Value labels above data points

use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::core::{
    AccessCtx, BrushIndex, EventCtx, LayoutCtx, MeasureCtx, PaintCtx, PointerEvent,
    PropertiesMut, PropertiesRef, RegisterCtx, StyleProperty,
    Update, UpdateCtx, Widget, WidgetMut, render_text, ChildrenIds,
};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Affine, Axis, BezPath, Line, Point, Rect, RoundedRect, Size, Stroke};
use xilem::masonry::layout::LenReq;
use xilem::masonry::parley::{Layout as ParleyLayout, StyleSet};
use xilem::masonry::peniko::{Brush, Fill};
use xilem::Color;

// Layout constants
const PADDING_LEFT: f64 = 40.0;   // Space for Y-axis labels
const PADDING_RIGHT: f64 = 10.0;
const PADDING_TOP: f64 = 38.0;    // Space for title and value labels
const PADDING_BOTTOM: f64 = 25.0; // Space for X-axis labels

// Colors
const BG_COLOR: Color = Color::from_rgba8(0xFF, 0xFF, 0xFF, 0x80);
const AXIS_COLOR: Color = Color::from_rgba8(0x99, 0x99, 0x99, 0xFF);
const BAR_COLOR: Color = Color::from_rgba8(0x4A, 0x90, 0xD9, 0xCC); // Blue with alpha
const LINE_COLOR: Color = Color::from_rgba8(0x4A, 0x90, 0xD9, 0xFF);
const TEXT_COLOR: Color = Color::from_rgba8(0x33, 0x33, 0x33, 0xFF);
const TEXT_DIM: Color = Color::from_rgba8(0x88, 0x88, 0x88, 0xFF);

/// Chart display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChartMode {
    /// Bar chart with filled rectangles.
    #[default]
    Bar,
    /// Line chart with connected points.
    Line,
}

/// Action emitted by the chart widget.
#[derive(Debug, Clone, PartialEq)]
pub enum ChartAction {
    /// A data point was clicked (index).
    PointClicked(usize),
}

/// Chart widget that renders bar or line charts.
pub struct ChartWidget {
    /// Chart title/caption.
    title: String,
    /// Data values.
    values: Vec<f64>,
    /// X-axis labels.
    labels: Vec<String>,
    /// Chart mode (bar or line).
    mode: ChartMode,
    /// Whether to show value labels.
    show_values: bool,
    /// Cached max value.
    max_value: f64,
    /// Cached text layouts.
    title_layout: Option<ParleyLayout<BrushIndex>>,
    label_layouts: Vec<ParleyLayout<BrushIndex>>,
    value_layouts: Vec<ParleyLayout<BrushIndex>>,
    max_label_layout: Option<ParleyLayout<BrushIndex>>,
    zero_label_layout: Option<ParleyLayout<BrushIndex>>,
    cached_font_size: f32,
}

impl ChartWidget {
    /// Creates a new chart widget.
    pub fn new(title: impl Into<String>, values: Vec<f64>, labels: Vec<String>, mode: ChartMode) -> Self {
        let max_value = values.iter().cloned().fold(0.0_f64, f64::max);
        Self {
            title: title.into(),
            values,
            labels,
            mode,
            show_values: true,
            max_value,
            title_layout: None,
            label_layouts: Vec::new(),
            value_layouts: Vec::new(),
            max_label_layout: None,
            zero_label_layout: None,
            cached_font_size: 0.0,
        }
    }

    /// Sets whether to show value labels.
    pub fn with_show_values(mut self, show: bool) -> Self {
        self.show_values = show;
        self
    }

    /// Updates the chart data.
    pub fn set_data(this: &mut WidgetMut<'_, Self>, values: Vec<f64>, labels: Vec<String>) {
        this.widget.values = values;
        this.widget.labels = labels;
        this.widget.max_value = this.widget.values.iter().cloned().fold(0.0_f64, f64::max);
        this.widget.cached_font_size = 0.0; // Force rebuild
        this.ctx.request_render();
    }

    /// Sets the chart mode.
    pub fn set_mode(this: &mut WidgetMut<'_, Self>, mode: ChartMode) {
        this.widget.mode = mode;
        this.ctx.request_render();
    }

    /// Sets whether to show value labels.
    pub fn set_show_values(this: &mut WidgetMut<'_, Self>, show: bool) {
        this.widget.show_values = show;
        this.widget.cached_font_size = 0.0; // Force rebuild of layouts
        this.ctx.request_render();
    }

    /// Formats a value for display.
    fn format_value(value: f64) -> String {
        if value == value.floor() {
            format!("{:.0}", value)
        } else if (value * 10.0).floor() == value * 10.0 {
            format!("{:.1}", value)
        } else {
            format!("{:.2}", value)
        }
    }

    /// Rebuilds cached text layouts.
    fn rebuild_text_layouts(&mut self, ctx: &mut PaintCtx<'_>, font_size: f32) {
        let (font_ctx, layout_ctx) = ctx.text_contexts();

        // Title layout
        {
            let mut styles: StyleSet<BrushIndex> = StyleSet::new(font_size * 1.1);
            styles.insert(StyleProperty::Brush(BrushIndex(0)));
            styles.insert(StyleProperty::FontWeight(xilem::masonry::parley::style::FontWeight::BOLD));
            let mut builder = layout_ctx.ranged_builder(font_ctx, &self.title, 1.0, true);
            for prop in styles.inner().values() {
                builder.push_default(prop.to_owned());
            }
            let mut layout: ParleyLayout<BrushIndex> = builder.build(&self.title);
            layout.break_all_lines(None);
            self.title_layout = Some(layout);
        }

        // X-axis label layouts
        self.label_layouts.clear();
        for label in &self.labels {
            let mut styles: StyleSet<BrushIndex> = StyleSet::new(font_size * 0.85);
            styles.insert(StyleProperty::Brush(BrushIndex(0)));
            let mut builder = layout_ctx.ranged_builder(font_ctx, label, 1.0, true);
            for prop in styles.inner().values() {
                builder.push_default(prop.to_owned());
            }
            let mut layout: ParleyLayout<BrushIndex> = builder.build(label);
            layout.break_all_lines(None);
            self.label_layouts.push(layout);
        }

        // Value label layouts
        self.value_layouts.clear();
        for &value in &self.values {
            let text = Self::format_value(value);
            let mut styles: StyleSet<BrushIndex> = StyleSet::new(font_size * 0.75);
            styles.insert(StyleProperty::Brush(BrushIndex(0)));
            let mut builder = layout_ctx.ranged_builder(font_ctx, &text, 1.0, true);
            for prop in styles.inner().values() {
                builder.push_default(prop.to_owned());
            }
            let mut layout: ParleyLayout<BrushIndex> = builder.build(&text);
            layout.break_all_lines(None);
            self.value_layouts.push(layout);
        }

        // Max value label
        {
            let text = Self::format_value(self.max_value);
            let mut styles: StyleSet<BrushIndex> = StyleSet::new(font_size * 0.85);
            styles.insert(StyleProperty::Brush(BrushIndex(0)));
            let mut builder = layout_ctx.ranged_builder(font_ctx, &text, 1.0, true);
            for prop in styles.inner().values() {
                builder.push_default(prop.to_owned());
            }
            let mut layout: ParleyLayout<BrushIndex> = builder.build(&text);
            layout.break_all_lines(None);
            self.max_label_layout = Some(layout);
        }

        // Zero label
        {
            let text = "0";
            let mut styles: StyleSet<BrushIndex> = StyleSet::new(font_size * 0.85);
            styles.insert(StyleProperty::Brush(BrushIndex(0)));
            let mut builder = layout_ctx.ranged_builder(font_ctx, text, 1.0, true);
            for prop in styles.inner().values() {
                builder.push_default(prop.to_owned());
            }
            let mut layout: ParleyLayout<BrushIndex> = builder.build(text);
            layout.break_all_lines(None);
            self.zero_label_layout = Some(layout);
        }

        self.cached_font_size = font_size;
    }

    /// Calculates the chart area rectangle.
    fn chart_rect(&self, size: Size) -> Rect {
        Rect::new(
            PADDING_LEFT,
            PADDING_TOP,
            size.width - PADDING_RIGHT,
            size.height - PADDING_BOTTOM,
        )
    }
}

impl Widget for ChartWidget {
    type Action = ChartAction;

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx<'_>, _: &mut PropertiesMut<'_>, _event: &PointerEvent) {
        // Could implement point clicking here
    }

    fn accepts_pointer_interaction(&self) -> bool { false }
    fn accepts_focus(&self) -> bool { false }

    fn register_children(&mut self, _: &mut RegisterCtx<'_>) {}

    fn update(&mut self, _: &mut UpdateCtx<'_>, _: &mut PropertiesMut<'_>, _: &Update) {}

    fn measure(&mut self, _: &mut MeasureCtx<'_>, _: &PropertiesRef<'_>, axis: Axis, _len_req: LenReq, _cross: Option<f64>) -> f64 {
        match axis {
            Axis::Horizontal => 200.0, // Minimum width
            Axis::Vertical => 150.0,   // Minimum height
        }
    }

    fn layout(&mut self, _: &mut LayoutCtx<'_>, _: &PropertiesRef<'_>, _size: Size) {
        // Layout is flexible, we use whatever size we're given
    }

    fn paint(&mut self, ctx: &mut PaintCtx<'_>, _: &PropertiesRef<'_>, painter: &mut Painter<'_>) {
        let size = ctx.content_box_size();
        let font_size = (size.height as f32 * 0.07).max(9.0).min(13.0);

        // Rebuild text layouts if needed
        if (font_size - self.cached_font_size).abs() > 0.5 || self.title_layout.is_none() {
            self.rebuild_text_layouts(ctx, font_size);
        }

        // Background
        let bg_rect = Rect::new(0.0, 0.0, size.width, size.height);
        let rounded_bg = RoundedRect::from_rect(bg_rect, 6.0);
        painter.fill(&rounded_bg, BG_COLOR).fill_rule(Fill::NonZero).draw();

        let chart_rect = self.chart_rect(size);

        // Draw title
        if let Some(ref layout) = self.title_layout {
            let tx = PADDING_LEFT;
            let ty = 3.0;
            let brushes = [Brush::Solid(TEXT_COLOR.into())];
            render_text(painter, Affine::translate((tx, ty)), layout, &brushes, true);
        }

        // Draw Y-axis labels (max at top, 0 at bottom)
        if let Some(ref layout) = self.max_label_layout {
            let tw = layout.width() as f64;
            let tx = PADDING_LEFT - tw - 5.0;
            let ty = chart_rect.min_y() - 3.0;
            let brushes = [Brush::Solid(TEXT_DIM.into())];
            render_text(painter, Affine::translate((tx, ty)), layout, &brushes, true);
        }

        if let Some(ref layout) = self.zero_label_layout {
            let tw = layout.width() as f64;
            let th = layout.height() as f64;
            let tx = PADDING_LEFT - tw - 5.0;
            let ty = chart_rect.max_y() - th;
            let brushes = [Brush::Solid(TEXT_DIM.into())];
            render_text(painter, Affine::translate((tx, ty)), layout, &brushes, true);
        }

        // Draw axis lines
        // Y-axis
        let y_axis = Line::new(
            Point::new(chart_rect.min_x(), chart_rect.min_y()),
            Point::new(chart_rect.min_x(), chart_rect.max_y()),
        );
        painter.stroke(&y_axis, &Stroke::new(1.0), AXIS_COLOR).draw();

        // X-axis
        let x_axis = Line::new(
            Point::new(chart_rect.min_x(), chart_rect.max_y()),
            Point::new(chart_rect.max_x(), chart_rect.max_y()),
        );
        painter.stroke(&x_axis, &Stroke::new(1.0), AXIS_COLOR).draw();

        // Calculate data positions
        if self.values.is_empty() || self.max_value <= 0.0 {
            return;
        }

        let chart_width = chart_rect.width();
        let chart_height = chart_rect.height();
        let bar_width = chart_width / self.values.len() as f64;

        match self.mode {
            ChartMode::Bar => {
                // Draw bars
                for (i, &value) in self.values.iter().enumerate() {
                    let normalized = value / self.max_value;
                    let bar_height = normalized * chart_height;

                    let bar_x = chart_rect.min_x() + i as f64 * bar_width + bar_width * 0.15;
                    let bar_y = chart_rect.max_y() - bar_height;
                    let bar_w = bar_width * 0.7;

                    if bar_height > 0.0 {
                        let bar_rect = Rect::new(bar_x, bar_y, bar_x + bar_w, chart_rect.max_y());
                        let rounded = RoundedRect::from_rect(bar_rect, 2.0);
                        painter.fill(&rounded, BAR_COLOR).fill_rule(Fill::NonZero).draw();
                    }

                    // Value label above bar
                    if self.show_values && i < self.value_layouts.len() {
                        let layout = &self.value_layouts[i];
                        let tw = layout.width() as f64;
                        let tx = bar_x + bar_w / 2.0 - tw / 2.0;
                        let ty = bar_y - 14.0;
                        let brushes = [Brush::Solid(TEXT_COLOR.into())];
                        render_text(painter, Affine::translate((tx, ty)), layout, &brushes, true);
                    }
                }
            }
            ChartMode::Line => {
                // Draw line connecting points
                let mut path = BezPath::new();
                let mut points: Vec<Point> = Vec::new();

                for (i, &value) in self.values.iter().enumerate() {
                    let normalized = value / self.max_value;
                    let x = chart_rect.min_x() + (i as f64 + 0.5) * bar_width;
                    let y = chart_rect.max_y() - normalized * chart_height;
                    points.push(Point::new(x, y));
                }

                if !points.is_empty() {
                    path.move_to(points[0]);
                    for point in points.iter().skip(1) {
                        path.line_to(*point);
                    }
                    painter.stroke(&path, &Stroke::new(2.0), LINE_COLOR).draw();

                    // Draw points and value labels
                    for (i, &point) in points.iter().enumerate() {
                        // Point circle
                        let circle = xilem::masonry::kurbo::Circle::new(point, 3.0);
                        painter.fill(&circle, LINE_COLOR).fill_rule(Fill::NonZero).draw();

                        // Value label above point
                        if self.show_values && i < self.value_layouts.len() {
                            let layout = &self.value_layouts[i];
                            let tw = layout.width() as f64;
                            let tx = point.x - tw / 2.0;
                            let ty = point.y - 14.0;
                            let brushes = [Brush::Solid(TEXT_COLOR.into())];
                            render_text(painter, Affine::translate((tx, ty)), layout, &brushes, true);
                        }
                    }
                }
            }
        }

        // Draw X-axis labels
        for (i, layout) in self.label_layouts.iter().enumerate() {
            let tw = layout.width() as f64;
            let tx = chart_rect.min_x() + (i as f64 + 0.5) * bar_width - tw / 2.0;
            let ty = chart_rect.max_y() + 5.0;
            let brushes = [Brush::Solid(TEXT_DIM.into())];
            render_text(painter, Affine::translate((tx, ty)), layout, &brushes, true);
        }
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::from(vec![])
    }

    fn accessibility_role(&self) -> Role { Role::Figure }

    fn accessibility(&mut self, _ctx: &mut AccessCtx<'_>, _: &PropertiesRef<'_>, node: &mut Node) {
        node.set_label(self.title.as_str());
    }
}
