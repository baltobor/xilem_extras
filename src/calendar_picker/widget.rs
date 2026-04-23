//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Calendar picker masonry widget.
//!
//! A standalone calendar widget that can be used directly with masonry
//! or wrapped by the xilem view layer.

use std::any::TypeId;

use chrono::{Datelike, Duration, Local, NaiveDate};
use tracing::{trace_span, Span};
use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, EventCtx, LayoutCtx, MeasureCtx, PaintCtx,
    PointerButtonEvent, PointerEvent, PropertiesMut, PropertiesRef, RegisterCtx, TextEvent,
    Update, UpdateCtx, Widget, WidgetId, WidgetMut,
};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Axis, BezPath, Point, Rect, RoundedRect, Size, Stroke};
use xilem::masonry::layout::LenReq;
use xilem::masonry::peniko::Color;

// Layout constants
const CELL_SIZE: f64 = 36.0;
const HEADER_HEIGHT: f64 = 40.0;
const WEEKDAY_HEIGHT: f64 = 24.0;
const KW_HEIGHT: f64 = 28.0;
const ARROW_SIZE: f64 = 24.0;
const ARROW_MARGIN: f64 = 8.0;
const GRID_COLS: usize = 7;
const GRID_ROWS: usize = 6;

// Colors
const TODAY_BG: Color = Color::from_rgba8(0x00, 0x7A, 0xFF, 0xFF);
const SELECTED_BORDER: Color = Color::from_rgba8(0x00, 0x7A, 0xFF, 0xFF);
const HOVER_BG: Color = Color::from_rgba8(0xE8, 0xF4, 0xFF, 0xFF);
const ARROW_COLOR: Color = Color::from_rgba8(0x00, 0x7A, 0xFF, 0xFF);
const BG_COLOR: Color = Color::WHITE;

/// Action emitted by the calendar picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalendarAction {
    /// A date was selected.
    DateSelected(NaiveDate),
    /// The displayed month changed (via navigation).
    MonthChanged(NaiveDate),
}

/// Hit test result for pointer events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HitArea {
    None,
    PrevMonth,
    NextMonth,
    Day(u32, u32), // row, col
}

/// Simple calendar picker widget (no child labels - uses paint for text).
///
/// This is a minimal implementation that draws everything in paint().
/// For a richer version with proper label widgets, use the view-based API.
pub struct CalendarPickerWidget {
    /// First day of the currently displayed month.
    displayed_month: NaiveDate,
    /// Currently selected date (if any).
    selected_date: Option<NaiveDate>,
    /// Today's date for highlighting.
    today: NaiveDate,
    /// Currently hovered day cell (row, col).
    hovered_cell: Option<(u32, u32)>,
    /// Widget size after layout.
    size: Size,
}

impl CalendarPickerWidget {
    /// Creates a new calendar picker.
    ///
    /// - `selected_date`: Initially selected date (defaults to today if None).
    pub fn new(selected_date: Option<NaiveDate>) -> Self {
        let today = Local::now().date_naive();
        let displayed_month = selected_date
            .unwrap_or(today)
            .with_day(1)
            .unwrap_or(today);

        Self {
            displayed_month,
            selected_date,
            today,
            hovered_cell: None,
            size: Size::ZERO,
        }
    }

    /// Returns the first day of the grid (may be from previous month).
    fn grid_start(&self) -> NaiveDate {
        let first_of_month = self.displayed_month;
        let weekday = first_of_month.weekday();
        let days_from_monday = weekday.num_days_from_monday() as i64;
        first_of_month - Duration::days(days_from_monday)
    }

    /// Returns the date at grid position (row, col).
    fn date_at_cell(&self, row: u32, col: u32) -> NaiveDate {
        let start = self.grid_start();
        let offset = (row * GRID_COLS as u32 + col) as i64;
        start + Duration::days(offset)
    }

    /// Hit test: determine which area was clicked.
    fn hit_test(&self, pos: Point) -> HitArea {
        // Check prev arrow
        let prev_rect = Rect::new(
            ARROW_MARGIN,
            (HEADER_HEIGHT - ARROW_SIZE) / 2.0,
            ARROW_MARGIN + ARROW_SIZE,
            (HEADER_HEIGHT + ARROW_SIZE) / 2.0,
        );
        if prev_rect.contains(pos) {
            return HitArea::PrevMonth;
        }

        // Check next arrow
        let next_rect = Rect::new(
            self.size.width - ARROW_MARGIN - ARROW_SIZE,
            (HEADER_HEIGHT - ARROW_SIZE) / 2.0,
            self.size.width - ARROW_MARGIN,
            (HEADER_HEIGHT + ARROW_SIZE) / 2.0,
        );
        if next_rect.contains(pos) {
            return HitArea::NextMonth;
        }

        // Check day grid
        let grid_top = HEADER_HEIGHT + WEEKDAY_HEIGHT;
        if pos.y >= grid_top && pos.y < grid_top + (GRID_ROWS as f64 * CELL_SIZE) {
            let col = (pos.x / CELL_SIZE).floor() as u32;
            let row = ((pos.y - grid_top) / CELL_SIZE).floor() as u32;
            if col < GRID_COLS as u32 && row < GRID_ROWS as u32 {
                return HitArea::Day(row, col);
            }
        }

        HitArea::None
    }

    fn prev_month(&mut self) {
        if let Some(prev) = self
            .displayed_month
            .checked_sub_signed(Duration::days(28))
        {
            self.displayed_month = prev.with_day(1).unwrap_or(prev);
        }
    }

    fn next_month(&mut self) {
        if let Some(next) = self.displayed_month.checked_add_signed(Duration::days(32)) {
            self.displayed_month = next.with_day(1).unwrap_or(next);
        }
    }
}

impl CalendarPickerWidget {
    /// Sets the selected date.
    pub fn set_selected_date(this: &mut WidgetMut<'_, Self>, date: Option<NaiveDate>) {
        if this.widget.selected_date != date {
            this.widget.selected_date = date;
            if let Some(d) = date {
                if let Some(first) = d.with_day(1) {
                    this.widget.displayed_month = first;
                }
            }
            this.ctx.request_paint_only();
        }
    }

    /// Sets the displayed month.
    pub fn set_displayed_month(this: &mut WidgetMut<'_, Self>, month: NaiveDate) {
        let first = month.with_day(1).unwrap_or(month);
        if this.widget.displayed_month != first {
            this.widget.displayed_month = first;
            this.ctx.request_paint_only();
        }
    }
}

impl Widget for CalendarPickerWidget {
    type Action = CalendarAction;

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        match event {
            PointerEvent::Move(e) => {
                let pos = ctx.local_position(e.current.position);
                let hit = self.hit_test(pos);
                let new_hover = match hit {
                    HitArea::Day(row, col) => Some((row, col)),
                    _ => None,
                };
                if new_hover != self.hovered_cell {
                    self.hovered_cell = new_hover;
                    ctx.request_paint_only();
                }
            }
            PointerEvent::Leave(_) => {
                if self.hovered_cell.is_some() {
                    self.hovered_cell = None;
                    ctx.request_paint_only();
                }
            }
            PointerEvent::Down(_) => {
                ctx.capture_pointer();
            }
            PointerEvent::Up(PointerButtonEvent { state, .. }) => {
                if ctx.is_active() && ctx.is_hovered() {
                    let pos = ctx.local_position(state.position);
                    match self.hit_test(pos) {
                        HitArea::PrevMonth => {
                            self.prev_month();
                            ctx.submit_action::<CalendarAction>(CalendarAction::MonthChanged(
                                self.displayed_month,
                            ));
                            ctx.request_paint_only();
                        }
                        HitArea::NextMonth => {
                            self.next_month();
                            ctx.submit_action::<CalendarAction>(CalendarAction::MonthChanged(
                                self.displayed_month,
                            ));
                            ctx.request_paint_only();
                        }
                        HitArea::Day(row, col) => {
                            let date = self.date_at_cell(row, col);
                            self.selected_date = Some(date);
                            if let Some(first) = date.with_day(1) {
                                if first != self.displayed_month {
                                    self.displayed_month = first;
                                }
                            }
                            ctx.submit_action::<CalendarAction>(CalendarAction::DateSelected(
                                date,
                            ));
                            ctx.request_paint_only();
                        }
                        HitArea::None => {}
                    }
                }
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

    fn update(&mut self, ctx: &mut UpdateCtx<'_>, _props: &mut PropertiesMut<'_>, event: &Update) {
        match event {
            Update::HoveredChanged(_) | Update::ActiveChanged(_) | Update::FocusChanged(_) => {
                ctx.request_paint_only();
            }
            _ => {}
        }
    }

    fn register_children(&mut self, _ctx: &mut RegisterCtx<'_>) {
        // No children - all rendering in paint()
    }

    fn property_changed(&mut self, _ctx: &mut UpdateCtx<'_>, _property_type: TypeId) {}

    fn measure(
        &mut self,
        _ctx: &mut MeasureCtx<'_>,
        _props: &PropertiesRef<'_>,
        axis: Axis,
        _len_req: LenReq,
        _cross_length: Option<f64>,
    ) -> f64 {
        match axis {
            Axis::Horizontal => CELL_SIZE * GRID_COLS as f64,
            Axis::Vertical => {
                HEADER_HEIGHT + WEEKDAY_HEIGHT + CELL_SIZE * GRID_ROWS as f64 + KW_HEIGHT
            }
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, size: Size) {
        self.size = size;
    }

    fn paint(
        &mut self,
        _ctx: &mut PaintCtx<'_>,
        _props: &PropertiesRef<'_>,
        painter: &mut Painter<'_>,
    ) {
        let width = self.size.width;

        // Background
        let bg_rect = Rect::from_origin_size(Point::ZERO, self.size);
        painter
            .fill(RoundedRect::from_rect(bg_rect, 8.0), BG_COLOR)
            .draw();

        // Draw navigation arrows
        self.draw_arrow(
            painter,
            ARROW_MARGIN + ARROW_SIZE / 2.0,
            HEADER_HEIGHT / 2.0,
            true,
        );
        self.draw_arrow(
            painter,
            width - ARROW_MARGIN - ARROW_SIZE / 2.0,
            HEADER_HEIGHT / 2.0,
            false,
        );

        // Draw cell backgrounds (hover, today, selected)
        let grid_top = HEADER_HEIGHT + WEEKDAY_HEIGHT;

        for row in 0..GRID_ROWS as u32 {
            for col in 0..GRID_COLS as u32 {
                let date = self.date_at_cell(row, col);
                let is_today = date == self.today;
                let is_selected = self.selected_date == Some(date);
                let is_hovered = self.hovered_cell == Some((row, col));

                let cell_x = col as f64 * CELL_SIZE;
                let cell_y = grid_top + row as f64 * CELL_SIZE;
                let cell_center = Point::new(cell_x + CELL_SIZE / 2.0, cell_y + CELL_SIZE / 2.0);

                if is_hovered && !is_today {
                    let hover_rect = Rect::from_center_size(
                        cell_center,
                        Size::new(CELL_SIZE - 4.0, CELL_SIZE - 4.0),
                    );
                    painter
                        .fill(RoundedRect::from_rect(hover_rect, 4.0), HOVER_BG)
                        .draw();
                }

                if is_today {
                    let circle_rect = Rect::from_center_size(
                        cell_center,
                        Size::new(CELL_SIZE - 6.0, CELL_SIZE - 6.0),
                    );
                    painter
                        .fill(
                            RoundedRect::from_rect(circle_rect, (CELL_SIZE - 6.0) / 2.0),
                            TODAY_BG,
                        )
                        .draw();
                }

                if is_selected && !is_today {
                    let select_rect = Rect::from_center_size(
                        cell_center,
                        Size::new(CELL_SIZE - 6.0, CELL_SIZE - 6.0),
                    );
                    painter
                        .stroke(
                            RoundedRect::from_rect(select_rect, (CELL_SIZE - 6.0) / 2.0),
                            &Stroke::new(2.0),
                            SELECTED_BORDER,
                        )
                        .draw();
                }
            }
        }

        // Note: Text rendering is not done here as masonry's Painter doesn't have
        // a simple draw_text API. The view layer should handle text via label widgets.
    }

    fn accessibility_role(&self) -> Role {
        Role::Grid
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        _node: &mut Node,
    ) {
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::from_slice(&[])
    }

    fn accepts_focus(&self) -> bool {
        true
    }

    fn accepts_text_input(&self) -> bool {
        false
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("CalendarPickerWidget", id = id.trace())
    }
}

impl CalendarPickerWidget {
    fn draw_arrow(&self, painter: &mut Painter<'_>, cx: f64, cy: f64, is_left: bool) {
        let half = 6.0;
        let (x1, x2) = if is_left {
            (cx + half / 2.0, cx - half / 2.0)
        } else {
            (cx - half / 2.0, cx + half / 2.0)
        };

        let mut path = BezPath::new();
        path.move_to(Point::new(x1, cy - half));
        path.line_to(Point::new(x2, cy));
        path.line_to(Point::new(x1, cy + half));

        painter.stroke(&path, &Stroke::new(2.0), ARROW_COLOR).draw();
    }
}
