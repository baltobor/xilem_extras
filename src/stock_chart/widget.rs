//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Stock chart widget implementation.

use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, EventCtx, LayoutCtx, MeasureCtx, PaintCtx,
    PointerEvent, PointerUpdate, PropertiesMut, PropertiesRef,
    RegisterCtx, TextEvent, Update, UpdateCtx, Widget, WidgetId,
};
use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Axis, BezPath, Line, Point, Rect, Size, Stroke};
use xilem::masonry::layout::LenReq;
use xilem::masonry::peniko::Color;
use tracing::{trace_span, Span};

/// A single price bar with OHLCV data.
#[derive(Debug, Clone)]
pub struct StockBar {
    /// Date or time label for this bar.
    pub label: String,
    /// Opening price.
    pub open: f64,
    /// Highest price.
    pub high: f64,
    /// Lowest price.
    pub low: f64,
    /// Closing price.
    pub close: f64,
    /// Trading volume.
    pub volume: i64,
}

impl StockBar {
    /// Creates a new stock bar.
    pub fn new(
        label: impl Into<String>,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: i64,
    ) -> Self {
        Self {
            label: label.into(),
            open,
            high,
            low,
            close,
            volume,
        }
    }

    /// Returns true if this is a bullish (up) bar.
    pub fn is_bullish(&self) -> bool {
        self.close >= self.open
    }
}

/// Chart rendering mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StockChartMode {
    /// Candlestick chart with filled bodies.
    #[default]
    Candlestick,
    /// OHLC bar chart with tick marks.
    OhlcBar,
    /// Simple line chart connecting close prices.
    Line,
    /// Line chart with gradient fill below.
    Area,
}

/// Style configuration for stock charts.
#[derive(Debug, Clone)]
pub struct StockChartStyle {
    /// Color for bullish (up) bars.
    pub up_color: Color,
    /// Color for bearish (down) bars.
    pub down_color: Color,
    /// Color for line/area charts.
    pub line_color: Color,
    /// Fill color for area charts.
    pub area_fill_color: Color,
    /// Background color.
    pub background_color: Color,
    /// Grid line color.
    pub grid_color: Color,
    /// Text/label color.
    pub text_color: Color,
    /// Volume bar color.
    pub volume_color: Color,
    /// Crosshair/locator color.
    pub crosshair_color: Color,
    /// Whether to show volume bars.
    pub show_volume: bool,
    /// Whether to show grid lines.
    pub show_grid: bool,
    /// Whether to show crosshair on hover.
    pub show_crosshair: bool,
}

impl Default for StockChartStyle {
    fn default() -> Self {
        Self {
            up_color: Color::from_rgb8(0x22, 0xC5, 0x5E),
            down_color: Color::from_rgb8(0xE5, 0x3E, 0x3E),
            line_color: Color::from_rgb8(0x55, 0x99, 0xDD),
            area_fill_color: Color::from_rgba8(0x55, 0x99, 0xDD, 0x30),
            background_color: Color::from_rgb8(0x1E, 0x1E, 0x2E),
            grid_color: Color::from_rgb8(0x2A, 0x2A, 0x3A),
            text_color: Color::from_rgb8(0x7F, 0x84, 0x9C),
            volume_color: Color::from_rgb8(0x44, 0x55, 0x88),
            crosshair_color: Color::from_rgba8(0xFF, 0xFF, 0xFF, 0x60),
            show_volume: true,
            show_grid: true,
            show_crosshair: true,
        }
    }
}

/// Action emitted by stock chart on hover.
#[derive(Debug, Clone)]
pub struct StockChartHover {
    /// The hovered bar's label (date/time).
    pub label: String,
    /// The hovered bar's close price.
    pub close: f64,
    /// Index of the hovered bar.
    pub index: usize,
}

/// Stock chart widget for displaying OHLCV financial data.
pub struct StockChartWidget {
    /// Price bars to display.
    bars: Vec<StockBar>,
    /// Rendering mode.
    mode: StockChartMode,
    /// Style configuration.
    style: StockChartStyle,
    /// Current widget size.
    size: Size,
    /// Mouse position for crosshair.
    mouse_pos: Option<Point>,
    /// Currently hovered bar index.
    hovered_index: Option<usize>,
}

impl StockChartWidget {
    /// Creates a new stock chart widget.
    pub fn new(bars: Vec<StockBar>, mode: StockChartMode) -> Self {
        Self {
            bars,
            mode,
            style: StockChartStyle::default(),
            size: Size::ZERO,
            mouse_pos: None,
            hovered_index: None,
        }
    }

    /// Creates a new stock chart with custom style.
    pub fn with_style(bars: Vec<StockBar>, mode: StockChartMode, style: StockChartStyle) -> Self {
        Self {
            bars,
            mode,
            style,
            size: Size::ZERO,
            mouse_pos: None,
            hovered_index: None,
        }
    }

    /// Sets the price bars.
    pub fn set_bars(&mut self, bars: Vec<StockBar>) {
        self.bars = bars;
    }

    /// Sets the chart mode.
    pub fn set_mode(&mut self, mode: StockChartMode) {
        self.mode = mode;
    }

    /// Sets the style.
    pub fn set_style(&mut self, style: StockChartStyle) {
        self.style = style;
    }

    /// Calculates the price range with padding.
    fn price_range(&self) -> (f64, f64) {
        if self.bars.is_empty() {
            return (0.0, 100.0);
        }

        let mut min_price = f64::MAX;
        let mut max_price = f64::MIN;

        for bar in &self.bars {
            if bar.low < min_price && bar.low.is_finite() {
                min_price = bar.low;
            }
            if bar.high > max_price && bar.high.is_finite() {
                max_price = bar.high;
            }
        }

        if min_price == f64::MAX || max_price == f64::MIN {
            return (0.0, 100.0);
        }

        let range = max_price - min_price;
        if range < 0.001 {
            return (min_price - 1.0, max_price + 1.0);
        }

        let padding = range * 0.05;
        (min_price - padding, max_price + padding)
    }

    /// Calculates the maximum volume.
    fn max_volume(&self) -> i64 {
        self.bars.iter().map(|b| b.volume).max().unwrap_or(1)
    }

    /// Returns chart area dimensions.
    fn chart_area(&self) -> Rect {
        let top_margin = 8.0;
        let chart_height = if self.style.show_volume {
            self.size.height * 0.78
        } else {
            self.size.height - top_margin - 8.0
        };
        Rect::new(0.0, top_margin, self.size.width, top_margin + chart_height)
    }

    /// Returns volume area dimensions.
    fn volume_area(&self) -> Rect {
        if !self.style.show_volume {
            return Rect::ZERO;
        }
        let chart = self.chart_area();
        let gap = self.size.height * 0.02;
        let vol_height = self.size.height * 0.15;
        Rect::new(0.0, chart.y1 + gap, self.size.width, chart.y1 + gap + vol_height)
    }

    /// Converts a price to Y coordinate.
    fn price_to_y(&self, price: f64, area: &Rect) -> f64 {
        let (min_price, max_price) = self.price_range();
        let range = max_price - min_price;
        if range < 0.001 {
            return area.center().y;
        }
        area.y0 + (1.0 - (price - min_price) / range) * area.height()
    }

    /// Gets bar X position (centered).
    fn bar_x(&self, index: usize, step: f64) -> f64 {
        (index as f64 + 0.5) * step
    }

    /// Hit tests which bar is at position.
    fn hit_test_bar(&self, pos: Point) -> Option<usize> {
        if self.bars.is_empty() {
            return None;
        }
        let chart = self.chart_area();
        if !chart.contains(pos) && !self.volume_area().contains(pos) {
            return None;
        }
        let step = self.size.width / self.bars.len() as f64;
        let index = (pos.x / step).floor() as usize;
        if index < self.bars.len() {
            Some(index)
        } else {
            None
        }
    }

    /// Paints the grid.
    fn paint_grid(&self, painter: &mut Painter<'_>) {
        if !self.style.show_grid {
            return;
        }

        let chart = self.chart_area();
        let stroke = Stroke::new(0.5);

        for i in 0..=4 {
            let y = chart.y0 + (i as f64 / 4.0) * chart.height();
            let line = Line::new(Point::new(0.0, y), Point::new(self.size.width, y));
            painter.stroke(&line, &stroke, self.style.grid_color).draw();
        }
    }

    /// Paints volume bars.
    fn paint_volume(&self, painter: &mut Painter<'_>) {
        if !self.style.show_volume || self.bars.is_empty() {
            return;
        }

        let area = self.volume_area();
        let max_vol = self.max_volume() as f64;
        if max_vol <= 0.0 {
            return;
        }

        let step = self.size.width / self.bars.len() as f64;
        let bar_width = (step * 0.8).clamp(1.0, 6.0);

        for (i, bar) in self.bars.iter().enumerate() {
            let x = self.bar_x(i, step);
            let height = (bar.volume as f64 / max_vol) * area.height();
            let rect = Rect::new(
                x - bar_width / 2.0,
                area.y1 - height,
                x + bar_width / 2.0,
                area.y1,
            );
            painter.fill(rect, self.style.volume_color).draw();
        }
    }

    /// Paints candlestick chart.
    fn paint_candlestick(&self, painter: &mut Painter<'_>) {
        if self.bars.is_empty() {
            return;
        }

        let chart = self.chart_area();
        let step = self.size.width / self.bars.len() as f64;
        let body_width = (step * 0.7).clamp(1.0, 12.0);
        let stroke = Stroke::new(1.0);

        for (i, bar) in self.bars.iter().enumerate() {
            let x = self.bar_x(i, step);
            let color = if bar.is_bullish() {
                self.style.up_color
            } else {
                self.style.down_color
            };

            // Wick (high to low)
            let y_high = self.price_to_y(bar.high, &chart);
            let y_low = self.price_to_y(bar.low, &chart);
            let wick = Line::new(Point::new(x, y_high), Point::new(x, y_low));
            painter.stroke(&wick, &stroke, color).draw();

            // Body (open to close)
            let y_open = self.price_to_y(bar.open, &chart);
            let y_close = self.price_to_y(bar.close, &chart);
            let body = Rect::new(
                x - body_width / 2.0,
                y_open.min(y_close),
                x + body_width / 2.0,
                y_open.max(y_close),
            );
            painter.fill(body, color).draw();
        }
    }

    /// Paints OHLC bar chart.
    fn paint_ohlc_bar(&self, painter: &mut Painter<'_>) {
        if self.bars.is_empty() {
            return;
        }

        let chart = self.chart_area();
        let step = self.size.width / self.bars.len() as f64;
        let tick_width = (step * 0.3).clamp(2.0, 8.0);
        let stroke = Stroke::new(1.5);

        for (i, bar) in self.bars.iter().enumerate() {
            let x = self.bar_x(i, step);
            let color = if bar.is_bullish() {
                self.style.up_color
            } else {
                self.style.down_color
            };

            let y_high = self.price_to_y(bar.high, &chart);
            let y_low = self.price_to_y(bar.low, &chart);
            let y_open = self.price_to_y(bar.open, &chart);
            let y_close = self.price_to_y(bar.close, &chart);

            // Vertical line (high to low)
            let vert = Line::new(Point::new(x, y_high), Point::new(x, y_low));
            painter.stroke(&vert, &stroke, color).draw();

            // Left tick (open)
            let open_tick = Line::new(
                Point::new(x - tick_width, y_open),
                Point::new(x, y_open),
            );
            painter.stroke(&open_tick, &stroke, color).draw();

            // Right tick (close)
            let close_tick = Line::new(
                Point::new(x, y_close),
                Point::new(x + tick_width, y_close),
            );
            painter.stroke(&close_tick, &stroke, color).draw();
        }
    }

    /// Paints line chart.
    fn paint_line(&self, painter: &mut Painter<'_>) {
        if self.bars.len() < 2 {
            return;
        }

        let chart = self.chart_area();
        let step = self.size.width / self.bars.len() as f64;
        let stroke = Stroke::new(2.0);

        let mut path = BezPath::new();
        for (i, bar) in self.bars.iter().enumerate() {
            let x = self.bar_x(i, step);
            let y = self.price_to_y(bar.close, &chart);
            if i == 0 {
                path.move_to(Point::new(x, y));
            } else {
                path.line_to(Point::new(x, y));
            }
        }

        painter.stroke(&path, &stroke, self.style.line_color).draw();
    }

    /// Paints area chart.
    fn paint_area(&self, painter: &mut Painter<'_>) {
        if self.bars.len() < 2 {
            return;
        }

        let chart = self.chart_area();
        let step = self.size.width / self.bars.len() as f64;

        // Build fill path
        let mut fill_path = BezPath::new();
        let first_x = self.bar_x(0, step);
        fill_path.move_to(Point::new(first_x, chart.y1));

        for (i, bar) in self.bars.iter().enumerate() {
            let x = self.bar_x(i, step);
            let y = self.price_to_y(bar.close, &chart);
            fill_path.line_to(Point::new(x, y));
        }

        let last_x = self.bar_x(self.bars.len() - 1, step);
        fill_path.line_to(Point::new(last_x, chart.y1));
        fill_path.close_path();

        painter.fill(&fill_path, self.style.area_fill_color).draw();

        // Draw line on top
        self.paint_line(painter);
    }

    /// Paints crosshair at mouse position.
    fn paint_crosshair(&self, painter: &mut Painter<'_>) {
        if !self.style.show_crosshair {
            return;
        }

        let Some(pos) = self.mouse_pos else { return };
        let Some(index) = self.hovered_index else { return };
        let Some(bar) = self.bars.get(index) else { return };

        let chart = self.chart_area();
        let stroke = Stroke::new(1.0);

        // Vertical line
        let vert = Line::new(
            Point::new(pos.x, chart.y0),
            Point::new(pos.x, self.size.height),
        );
        painter.stroke(&vert, &stroke, self.style.crosshair_color).draw();

        // Horizontal line at close price
        let y_close = self.price_to_y(bar.close, &chart);
        let horiz = Line::new(
            Point::new(0.0, y_close),
            Point::new(self.size.width, y_close),
        );
        painter.stroke(&horiz, &stroke, self.style.crosshair_color).draw();

        // Dot at intersection
        let dot = Rect::new(pos.x - 3.0, y_close - 3.0, pos.x + 3.0, y_close + 3.0);
        painter.fill(dot, self.style.crosshair_color).draw();
    }
}

impl Widget for StockChartWidget {
    type Action = Option<StockChartHover>;

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        match event {
            PointerEvent::Move(PointerUpdate { current, .. }) => {
                let pos = ctx.local_position(current.position);
                self.mouse_pos = Some(pos);
                let new_index = self.hit_test_bar(pos);

                if new_index != self.hovered_index {
                    self.hovered_index = new_index;

                    if let Some(idx) = new_index {
                        if let Some(bar) = self.bars.get(idx) {
                            ctx.submit_action::<Self::Action>(Some(StockChartHover {
                                label: bar.label.clone(),
                                close: bar.close,
                                index: idx,
                            }));
                        }
                    } else {
                        ctx.submit_action::<Self::Action>(None);
                    }
                }
                ctx.request_render();
            }
            PointerEvent::Leave(..) => {
                self.mouse_pos = None;
                self.hovered_index = None;
                ctx.submit_action::<Self::Action>(None);
                ctx.request_render();
            }
            _ => {}
        }
    }

    fn on_text_event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &TextEvent,
    ) {
    }

    fn on_access_event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &AccessEvent,
    ) {
    }

    fn update(&mut self, _ctx: &mut UpdateCtx<'_>, _props: &mut PropertiesMut<'_>, _event: &Update) {
    }

    fn register_children(&mut self, _ctx: &mut RegisterCtx<'_>) {
    }

    fn measure(
        &mut self,
        _ctx: &mut MeasureCtx<'_>,
        _props: &PropertiesRef<'_>,
        axis: Axis,
        len_req: LenReq,
        _cross_length: Option<f64>,
    ) -> f64 {
        match axis {
            Axis::Horizontal => match len_req {
                LenReq::FitContent(available) => available.min(800.0).max(200.0),
                LenReq::MinContent => 200.0,
                LenReq::MaxContent => 800.0,
            },
            Axis::Vertical => match len_req {
                LenReq::FitContent(available) => available.min(400.0).max(150.0),
                LenReq::MinContent => 150.0,
                LenReq::MaxContent => 400.0,
            },
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, size: Size) {
        self.size = size;
    }

    fn paint(&mut self, _ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, painter: &mut Painter<'_>) {
        // Background
        let bg = Rect::from_origin_size(Point::ZERO, self.size);
        painter.fill(bg, self.style.background_color).draw();

        // Grid
        self.paint_grid(painter);

        // Volume
        self.paint_volume(painter);

        // Chart data
        match self.mode {
            StockChartMode::Candlestick => self.paint_candlestick(painter),
            StockChartMode::OhlcBar => self.paint_ohlc_bar(painter),
            StockChartMode::Line => self.paint_line(painter),
            StockChartMode::Area => self.paint_area(painter),
        }

        // Crosshair
        self.paint_crosshair(painter);
    }

    fn accessibility_role(&self) -> Role {
        Role::Canvas
    }

    fn accessibility(&mut self, _ctx: &mut AccessCtx<'_>, _props: &PropertiesRef<'_>, node: &mut Node) {
        node.set_label("Stock chart");
    }

    fn accepts_focus(&self) -> bool {
        false
    }

    fn accepts_pointer_interaction(&self) -> bool {
        true
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::from_slice(&[])
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("StockChartWidget", id = id.trace())
    }
}
