//! Grid-based calendar widget - paints everything directly.
//!
//! Uses masonry to render a pixel-exact calendar grid.
//! No child widgets - all text is rendered in paint().

use chrono::{Datelike, Duration, Local, NaiveDate};
use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::core::{
    AccessCtx, BrushIndex, EventCtx, LayoutCtx, MeasureCtx, PaintCtx, PointerEvent,
    PointerButtonEvent, PropertiesMut, PropertiesRef, RegisterCtx, StyleProperty,
    Update, UpdateCtx, Widget, WidgetMut, render_text, ChildrenIds,
};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Affine, Axis, Point, Rect, RoundedRect, Size};
use xilem::masonry::layout::LenReq;
use xilem::masonry::parley::{Layout as ParleyLayout, StyleSet};
use xilem::masonry::peniko::{Brush, Fill};
use xilem::Color;

// Grid dimensions
pub const NUM_COLS: usize = 7;
pub const NUM_DAY_ROWS: usize = 6;
pub const CELL_SIZE: f64 = 28.0;

// Colors
const BG_COLOR: Color = Color::WHITE;
const TEXT_COLOR: Color = Color::from_rgba8(0x33, 0x33, 0x33, 0xFF);
const TEXT_DIM: Color = Color::from_rgba8(0xAA, 0xAA, 0xAA, 0xFF);
const TEXT_WEEKEND: Color = Color::from_rgba8(0xCC, 0x55, 0x55, 0xFF);
const TODAY_BG: Color = Color::from_rgba8(0x00, 0x7A, 0xFF, 0xFF);
const SELECTED_BG: Color = Color::from_rgba8(0xE0, 0xEE, 0xFF, 0xFF);

const WEEKDAYS: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

/// Action emitted when a day cell is clicked.
#[derive(Debug, Clone)]
pub enum CalendarAction {
    DayClicked(NaiveDate),
}

/// Masonry widget that renders a complete calendar grid.
pub struct CalendarGridWidget {
    displayed_month: NaiveDate,
    selected: Option<NaiveDate>,
    today: NaiveDate,
    cell_size: f64,
    // Cached text layouts
    weekday_layouts: Vec<ParleyLayout<BrushIndex>>,
    day_layouts: Vec<ParleyLayout<BrushIndex>>,
    cached_font_size: f32,
}

impl CalendarGridWidget {
    pub fn new(displayed_month: NaiveDate, selected: Option<NaiveDate>) -> Self {
        Self {
            displayed_month,
            selected,
            today: Local::now().date_naive(),
            cell_size: CELL_SIZE,
            weekday_layouts: Vec::new(),
            day_layouts: Vec::new(),
            cached_font_size: 0.0,
        }
    }

    pub fn set_state(this: &mut WidgetMut<'_, Self>, displayed_month: NaiveDate, selected: Option<NaiveDate>) {
        this.widget.displayed_month = displayed_month;
        this.widget.selected = selected;
        this.widget.today = Local::now().date_naive();
        this.ctx.request_render();
    }

    fn grid_start(&self) -> NaiveDate {
        let month_start = self.displayed_month.with_day(1).unwrap_or(self.displayed_month);
        let wd = month_start.weekday().num_days_from_monday() as i64;
        month_start - Duration::days(wd)
    }

    fn date_at(&self, col: usize, row: usize) -> NaiveDate {
        let grid_start = self.grid_start();
        grid_start + Duration::days((row * NUM_COLS + col) as i64)
    }

    fn hit_test(&self, pos: Point) -> Option<(usize, usize)> {
        // Row 0 is header
        if pos.y < self.cell_size {
            return None;
        }
        let col = (pos.x / self.cell_size) as usize;
        let row = ((pos.y - self.cell_size) / self.cell_size) as usize;
        if col < NUM_COLS && row < NUM_DAY_ROWS {
            Some((col, row))
        } else {
            None
        }
    }

    fn rebuild_text_layouts(&mut self, ctx: &mut PaintCtx<'_>, font_size: f32) {
        let (font_ctx, layout_ctx) = ctx.text_contexts();

        // Weekday layouts
        self.weekday_layouts.clear();
        for wd in &WEEKDAYS {
            let mut styles: StyleSet<BrushIndex> = StyleSet::new(font_size * 0.8);
            styles.insert(StyleProperty::Brush(BrushIndex(0)));
            let mut builder = layout_ctx.ranged_builder(font_ctx, wd, 1.0, true);
            for prop in styles.inner().values() {
                builder.push_default(prop.to_owned());
            }
            let mut layout: ParleyLayout<BrushIndex> = builder.build(wd);
            layout.break_all_lines(None);
            self.weekday_layouts.push(layout);
        }

        // Day number layouts (1-31)
        self.day_layouts.clear();
        for day in 1..=31 {
            let text = day.to_string();
            let mut styles: StyleSet<BrushIndex> = StyleSet::new(font_size);
            styles.insert(StyleProperty::Brush(BrushIndex(0)));
            let mut builder = layout_ctx.ranged_builder(font_ctx, &text, 1.0, true);
            for prop in styles.inner().values() {
                builder.push_default(prop.to_owned());
            }
            let mut layout: ParleyLayout<BrushIndex> = builder.build(&text);
            layout.break_all_lines(None);
            self.day_layouts.push(layout);
        }

        self.cached_font_size = font_size;
    }
}

impl Widget for CalendarGridWidget {
    type Action = CalendarAction;

    fn on_pointer_event(&mut self, ctx: &mut EventCtx<'_>, _: &mut PropertiesMut<'_>, event: &PointerEvent) {
        match event {
            PointerEvent::Down(_) => {
                ctx.capture_pointer();
            }
            PointerEvent::Up(PointerButtonEvent { state, .. }) => {
                let pos = ctx.local_position(state.position);
                if let Some((col, row)) = self.hit_test(pos) {
                    let date = self.date_at(col, row);
                    self.selected = Some(date);
                    ctx.submit_action::<CalendarAction>(CalendarAction::DayClicked(date));
                    ctx.request_render();
                }
            }
            _ => {}
        }
    }

    fn accepts_pointer_interaction(&self) -> bool { true }
    fn accepts_focus(&self) -> bool { false }

    fn register_children(&mut self, _: &mut RegisterCtx<'_>) {}

    fn update(&mut self, _: &mut UpdateCtx<'_>, _: &mut PropertiesMut<'_>, _: &Update) {}

    fn measure(&mut self, _: &mut MeasureCtx<'_>, _: &PropertiesRef<'_>, axis: Axis, _len_req: LenReq, _cross: Option<f64>) -> f64 {
        match axis {
            Axis::Horizontal => self.cell_size * NUM_COLS as f64,
            Axis::Vertical => self.cell_size * (NUM_DAY_ROWS + 1) as f64,
        }
    }

    fn layout(&mut self, _: &mut LayoutCtx<'_>, _: &PropertiesRef<'_>, size: Size) {
        let cell_w = size.width / NUM_COLS as f64;
        let cell_h = size.height / (NUM_DAY_ROWS + 1) as f64;
        self.cell_size = cell_w.min(cell_h);
    }

    fn paint(&mut self, ctx: &mut PaintCtx<'_>, _: &PropertiesRef<'_>, painter: &mut Painter<'_>) {
        let size = ctx.content_box_size();
        let font_size = (self.cell_size as f32 * 0.45).max(9.0).min(14.0);

        // Rebuild text layouts if font size changed
        if (font_size - self.cached_font_size).abs() > 0.5 || self.weekday_layouts.is_empty() {
            self.rebuild_text_layouts(ctx, font_size);
        }

        // Background
        let bg_rect = Rect::new(0.0, 0.0, size.width, size.height);
        painter.fill(&bg_rect, BG_COLOR).fill_rule(Fill::NonZero).draw();

        let displayed_month = self.displayed_month.month();
        let displayed_year = self.displayed_month.year();

        // Paint weekday headers (row 0)
        for col in 0..NUM_COLS {
            let x = col as f64 * self.cell_size;
            let y = 0.0;

            if let Some(layout) = self.weekday_layouts.get(col) {
                let tw = layout.width() as f64;
                let th = layout.height() as f64;
                let tx = x + (self.cell_size - tw) / 2.0;
                let ty = y + (self.cell_size - th) / 2.0;

                let color = if col >= 5 { TEXT_WEEKEND } else { TEXT_DIM };
                let brushes = [Brush::Solid(color.into())];
                render_text(painter, Affine::translate((tx, ty)), layout, &brushes, true);
            }
        }

        // Paint day cells (rows 1-6)
        for row in 0..NUM_DAY_ROWS {
            for col in 0..NUM_COLS {
                let date = self.date_at(col, row);
                let x = col as f64 * self.cell_size;
                let y = (row + 1) as f64 * self.cell_size;

                let in_month = date.month() == displayed_month && date.year() == displayed_year;
                let is_today = date == self.today;
                let is_selected = self.selected == Some(date);
                let is_weekend = col >= 5;

                // Background
                let cell_rect = Rect::new(x + 2.0, y + 2.0, x + self.cell_size - 2.0, y + self.cell_size - 2.0);
                let bg = if is_today {
                    TODAY_BG
                } else if is_selected {
                    SELECTED_BG
                } else {
                    Color::TRANSPARENT
                };

                if bg != Color::TRANSPARENT {
                    let rounded = RoundedRect::from_rect(cell_rect, 4.0);
                    painter.fill(&rounded, bg).fill_rule(Fill::NonZero).draw();
                }

                // Day number text
                let day = date.day() as usize;
                if day >= 1 && day <= 31 {
                    if let Some(layout) = self.day_layouts.get(day - 1) {
                        let tw = layout.width() as f64;
                        let th = layout.height() as f64;
                        let tx = x + (self.cell_size - tw) / 2.0;
                        let ty = y + (self.cell_size - th) / 2.0;

                        let color = if is_today {
                            Color::WHITE
                        } else if !in_month {
                            TEXT_DIM
                        } else if is_weekend {
                            TEXT_WEEKEND
                        } else {
                            TEXT_COLOR
                        };
                        let brushes = [Brush::Solid(color.into())];
                        render_text(painter, Affine::translate((tx, ty)), layout, &brushes, true);
                    }
                }
            }
        }
    }

    fn accessibility_role(&self) -> Role { Role::Grid }
    fn accessibility(&mut self, _: &mut AccessCtx<'_>, _: &PropertiesRef<'_>, _: &mut Node) {}

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::from(vec![])
    }
}

// ============================================================================
// Xilem View Wrapper
// ============================================================================

use xilem::core::{MessageCtx, Mut, View, ViewMarker, MessageResult};
use xilem::{Pod, ViewCtx};

/// Xilem view for the calendar grid.
pub struct CalendarGrid<State, Action, F> {
    displayed_month: NaiveDate,
    selected: Option<NaiveDate>,
    on_select: F,
    phantom: std::marker::PhantomData<fn(State) -> Action>,
}

pub fn calendar_grid<State, Action, F>(
    displayed_month: NaiveDate,
    selected: Option<NaiveDate>,
    on_select: F,
) -> CalendarGrid<State, Action, F>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, NaiveDate) -> Action + 'static,
{
    CalendarGrid {
        displayed_month,
        selected,
        on_select,
        phantom: std::marker::PhantomData,
    }
}

#[derive(Clone)]
pub struct CalendarGridState {
    displayed_month: NaiveDate,
    selected: Option<NaiveDate>,
}

impl<State, Action, F> ViewMarker for CalendarGrid<State, Action, F> {}

impl<State, Action, F> View<State, Action, ViewCtx> for CalendarGrid<State, Action, F>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, NaiveDate) -> Action + 'static,
{
    type Element = Pod<CalendarGridWidget>;
    type ViewState = CalendarGridState;

    fn build(&self, ctx: &mut ViewCtx, _: &mut State) -> (Self::Element, Self::ViewState) {
        let widget = CalendarGridWidget::new(self.displayed_month, self.selected);
        let pod = ctx.with_action_widget(|ctx| ctx.create_pod(widget));
        let state = CalendarGridState {
            displayed_month: self.displayed_month,
            selected: self.selected,
        };
        (pod, state)
    }

    fn rebuild(
        &self, _prev: &Self, view_state: &mut Self::ViewState,
        _ctx: &mut ViewCtx, mut element: Mut<'_, Self::Element>, _: &mut State,
    ) {
        if self.displayed_month != view_state.displayed_month
            || self.selected != view_state.selected
        {
            CalendarGridWidget::set_state(&mut element, self.displayed_month, self.selected);
            view_state.displayed_month = self.displayed_month;
            view_state.selected = self.selected;
        }
    }

    fn teardown(&self, _: &mut Self::ViewState, ctx: &mut ViewCtx, element: Mut<'_, Self::Element>) {
        ctx.teardown_action_source(element);
    }

    fn message(
        &self, _: &mut Self::ViewState, message: &mut MessageCtx,
        _: Mut<'_, Self::Element>, state: &mut State,
    ) -> MessageResult<Action> {
        if let Some(action) = message.take_message::<CalendarAction>() {
            match *action {
                CalendarAction::DayClicked(date) => {
                    let result = (self.on_select)(state, date);
                    return MessageResult::Action(result);
                }
            }
        }
        MessageResult::Stale
    }
}
