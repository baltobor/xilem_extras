//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Time picker masonry widget.

use std::any::TypeId;

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
const WIDGET_WIDTH: f64 = 120.0;
const WIDGET_HEIGHT: f64 = 80.0;
const STEPPER_SIZE: f64 = 24.0;

// Colors
const BG_COLOR: Color = Color::WHITE;
const STEPPER_BG: Color = Color::from_rgba8(0xF0, 0xF0, 0xF0, 0xFF);
const STEPPER_HOVER: Color = Color::from_rgba8(0xE0, 0xE0, 0xE0, 0xFF);
const ARROW_COLOR: Color = Color::from_rgba8(0x00, 0x7A, 0xFF, 0xFF);
const BORDER_COLOR: Color = Color::from_rgba8(0xCC, 0xCC, 0xCC, 0xFF);

/// Action emitted by the time picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeAction {
    pub hour: u8,
    pub minute: u8,
}

/// Hit test result for pointer events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HitArea {
    None,
    HourUp,
    HourDown,
    MinuteUp,
    MinuteDown,
}

/// Simple time picker widget.
///
/// Displays hour:minute with stepper buttons for each.
pub struct TimePickerWidget {
    hour: u8,
    minute: u8,
    minute_step: u8,
    hovered_area: HitArea,
    size: Size,
}

impl TimePickerWidget {
    /// Creates a new time picker.
    ///
    /// - `hour`: Initial hour (0-23).
    /// - `minute`: Initial minute (0-59).
    pub fn new(hour: u8, minute: u8) -> Self {
        Self {
            hour: hour.min(23),
            minute: minute.min(59),
            minute_step: 5,
            hovered_area: HitArea::None,
            size: Size::ZERO,
        }
    }

    /// Sets the minute step (default 5).
    pub fn with_minute_step(mut self, step: u8) -> Self {
        self.minute_step = step.max(1);
        self
    }

    fn hit_test(&self, pos: Point) -> HitArea {
        let half_width = self.size.width / 2.0;
        let top_row_y = 0.0;
        let bottom_row_y = self.size.height - STEPPER_SIZE;

        // Hour up (left side, top)
        let hour_up_rect = Rect::new(0.0, top_row_y, half_width, top_row_y + STEPPER_SIZE);
        if hour_up_rect.contains(pos) {
            return HitArea::HourUp;
        }

        // Hour down (left side, bottom)
        let hour_down_rect = Rect::new(0.0, bottom_row_y, half_width, bottom_row_y + STEPPER_SIZE);
        if hour_down_rect.contains(pos) {
            return HitArea::HourDown;
        }

        // Minute up (right side, top)
        let minute_up_rect =
            Rect::new(half_width, top_row_y, self.size.width, top_row_y + STEPPER_SIZE);
        if minute_up_rect.contains(pos) {
            return HitArea::MinuteUp;
        }

        // Minute down (right side, bottom)
        let minute_down_rect = Rect::new(
            half_width,
            bottom_row_y,
            self.size.width,
            bottom_row_y + STEPPER_SIZE,
        );
        if minute_down_rect.contains(pos) {
            return HitArea::MinuteDown;
        }

        HitArea::None
    }

    fn increment_hour(&mut self) {
        self.hour = (self.hour + 1) % 24;
    }

    fn decrement_hour(&mut self) {
        self.hour = if self.hour == 0 { 23 } else { self.hour - 1 };
    }

    fn increment_minute(&mut self) {
        self.minute = (self.minute + self.minute_step) % 60;
    }

    fn decrement_minute(&mut self) {
        let step = self.minute_step as i16;
        self.minute = ((self.minute as i16 - step).rem_euclid(60)) as u8;
    }
}

impl TimePickerWidget {
    /// Sets the time.
    pub fn set_time(this: &mut WidgetMut<'_, Self>, hour: u8, minute: u8) {
        this.widget.hour = hour.min(23);
        this.widget.minute = minute.min(59);
        this.ctx.request_paint_only();
    }
}

impl Widget for TimePickerWidget {
    type Action = TimeAction;

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        match event {
            PointerEvent::Move(e) => {
                let pos = ctx.local_position(e.current.position);
                let new_hover = self.hit_test(pos);
                if new_hover != self.hovered_area {
                    self.hovered_area = new_hover;
                    ctx.request_paint_only();
                }
            }
            PointerEvent::Leave(_) => {
                if self.hovered_area != HitArea::None {
                    self.hovered_area = HitArea::None;
                    ctx.request_paint_only();
                }
            }
            PointerEvent::Down(_) => {
                ctx.capture_pointer();
            }
            PointerEvent::Up(PointerButtonEvent { state, .. }) => {
                if ctx.is_active() && ctx.is_hovered() {
                    let pos = ctx.local_position(state.position);
                    let changed = match self.hit_test(pos) {
                        HitArea::HourUp => {
                            self.increment_hour();
                            true
                        }
                        HitArea::HourDown => {
                            self.decrement_hour();
                            true
                        }
                        HitArea::MinuteUp => {
                            self.increment_minute();
                            true
                        }
                        HitArea::MinuteDown => {
                            self.decrement_minute();
                            true
                        }
                        HitArea::None => false,
                    };
                    if changed {
                        ctx.submit_action::<TimeAction>(TimeAction {
                            hour: self.hour,
                            minute: self.minute,
                        });
                        ctx.request_paint_only();
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

    fn register_children(&mut self, _ctx: &mut RegisterCtx<'_>) {}

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
            Axis::Horizontal => WIDGET_WIDTH,
            Axis::Vertical => WIDGET_HEIGHT,
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
        let height = self.size.height;
        let half_width = width / 2.0;

        // Background
        let bg_rect = Rect::from_origin_size(Point::ZERO, self.size);
        painter
            .fill(RoundedRect::from_rect(bg_rect, 6.0), BG_COLOR)
            .draw();
        painter
            .stroke(RoundedRect::from_rect(bg_rect, 6.0), &Stroke::new(1.0), BORDER_COLOR)
            .draw();

        // Draw stepper buttons (up arrows at top, down arrows at bottom)
        let stepper_areas = [
            (0.0, 0.0, half_width, STEPPER_SIZE, HitArea::HourUp, true),
            (0.0, height - STEPPER_SIZE, half_width, height, HitArea::HourDown, false),
            (half_width, 0.0, width, STEPPER_SIZE, HitArea::MinuteUp, true),
            (half_width, height - STEPPER_SIZE, width, height, HitArea::MinuteDown, false),
        ];

        for (x1, y1, x2, y2, area, is_up) in stepper_areas {
            let rect = Rect::new(x1, y1, x2, y2);
            let bg = if self.hovered_area == area {
                STEPPER_HOVER
            } else {
                STEPPER_BG
            };
            painter.fill(rect, bg).draw();

            // Draw arrow
            let cx = (x1 + x2) / 2.0;
            let cy = (y1 + y2) / 2.0;
            self.draw_arrow(painter, cx, cy, is_up);
        }

        // Separator line between hour and minute
        let sep_x = half_width;
        painter
            .stroke(
                &{
                    let mut path = BezPath::new();
                    path.move_to(Point::new(sep_x, STEPPER_SIZE));
                    path.line_to(Point::new(sep_x, height - STEPPER_SIZE));
                    path
                },
                &Stroke::new(1.0),
                BORDER_COLOR,
            )
            .draw();

        // Note: Actual time display text (HH:MM) should be rendered via child Label widgets
        // or handled by the view layer. This widget only draws backgrounds and arrows.
    }

    fn accessibility_role(&self) -> Role {
        Role::SpinButton
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
        trace_span!("TimePickerWidget", id = id.trace())
    }
}

impl TimePickerWidget {
    fn draw_arrow(&self, painter: &mut Painter<'_>, cx: f64, cy: f64, is_up: bool) {
        let half = 5.0;
        let (y1, y2) = if is_up {
            (cy + half / 2.0, cy - half / 2.0)
        } else {
            (cy - half / 2.0, cy + half / 2.0)
        };

        let mut path = BezPath::new();
        path.move_to(Point::new(cx - half, y1));
        path.line_to(Point::new(cx, y2));
        path.line_to(Point::new(cx + half, y1));

        painter.stroke(&path, &Stroke::new(2.0), ARROW_COLOR).draw();
    }
}
