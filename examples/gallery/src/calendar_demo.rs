//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Calendar and Time Picker demo page.

use chrono::{Datelike, Duration, Local, NaiveDate};
use masonry::layout::AsUnit;
use xilem::masonry::peniko::Color;
use xilem::style::Style;
use xilem::view::{button, flex_col, flex_row, label, CrossAxisAlignment, MainAxisAlignment};
use xilem::{WidgetView};
use xilem_extras::{calendar_picker, CalendarLocale};

use crate::app_model::AppModel;

// Demo page colors
const TEXT_COLOR: Color = Color::from_rgb8(220, 218, 214);
const TEXT_SECONDARY: Color = Color::from_rgb8(160, 156, 150);
const BG_CONTENT: Color = Color::from_rgb8(35, 33, 30);

// Calendar colors
const CAL_TEXT: Color = Color::from_rgba8(0x33, 0x33, 0x33, 0xFF);
const CAL_BG: Color = Color::WHITE;
const CAL_ARROW: Color = Color::from_rgba8(0x00, 0x7A, 0xFF, 0xFF);

// Time picker colors
const TIME_BG: Color = Color::from_rgba8(0xF5, 0xF5, 0xF5, 0xFF);
const TIME_TEXT: Color = Color::from_rgba8(0x33, 0x33, 0x33, 0xFF);
const TIME_ARROW: Color = Color::from_rgba8(0x00, 0x7A, 0xFF, 0xFF);

// Grid cell size
const CELL: f64 = 28.0;

// Locale for the demo
const LOCALE: CalendarLocale = CalendarLocale::English;

pub fn calendar_demo(model: &mut AppModel) -> impl WidgetView<AppModel> + use<> {
    let today = Local::now().date_naive();
    let display_date = model.calendar_selected_date.unwrap_or(today);
    let date_text = display_date.format("%d.%m.%Y").to_string();
    let kw_text = format!("{} {}", LOCALE.week_label(), display_date.iso_week().week());
    let selected_text = format!("Selected: {}", date_text);

    let time_text = format!("{:02}:{:02}", model.calendar_hour, model.calendar_minute);

    // Get displayed month from selected date or today
    let displayed_month = model.calendar_selected_date.unwrap_or(today);

    // Navigation dates
    let month_start = displayed_month.with_day(1).unwrap_or(displayed_month);
    let prev = (month_start - Duration::days(15)).with_day(1).unwrap_or(month_start);
    let next = (month_start + Duration::days(35)).with_day(1).unwrap_or(month_start);

    let months = LOCALE.months();
    let month_name = months[(month_start.month() - 1) as usize];
    let header = format!("{} {}", month_name, month_start.year());

    flex_col((
        label("Calendar & Time Picker")
            .text_size(16.0)
            .weight(xilem::FontWeight::BOLD)
            .color(TEXT_COLOR),

        // Selected date display
        label(selected_text)
            .text_size(12.0)
            .color(TEXT_COLOR),

        flex_row((
            // Calendar section
            flex_col((
                // Header with navigation
                flex_row((
                    arrow_btn("<", prev),
                    label(header).text_size(12.0).weight(xilem::FontWeight::MEDIUM).color(CAL_TEXT),
                    arrow_btn(">", next),
                ))
                .main_axis_alignment(MainAxisAlignment::SpaceBetween)
                .width(((CELL * 7.0) as i32).px()),

                // Grid-based calendar (using xilem_extras reusable widget)
                calendar_picker(
                    displayed_month,
                    model.calendar_selected_date,
                    |model: &mut AppModel, date| {
                        model.calendar_selected_date = Some(date);
                        () // Return unit action to trigger rebuild
                    },
                ),

                // Date, week display, and today button
                flex_row((
                    flex_row((
                        label(date_text).text_size(10.0).color(TEXT_SECONDARY),
                        label(kw_text).text_size(10.0).color(TEXT_SECONDARY),
                    )).gap(8.0_f64.px()),
                    today_btn(),
                ))
                .main_axis_alignment(MainAxisAlignment::SpaceBetween)
                .width(((CELL * 7.0) as i32).px()),
            ))
            .gap(4.0_f64.px())
            .padding(6.0)
            .background_color(CAL_BG)
            .corner_radius(6.0),

            // Time picker section
            flex_col((
                build_time_picker(model.calendar_hour, model.calendar_minute),
                label(time_text).text_size(10.0).color(TEXT_SECONDARY),
            ))
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .gap(4.0_f64.px()),
        ))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .gap(16.0_f64.px()),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Start)
    .gap(10.0_f64.px())
    .padding(12.0)
    .background_color(BG_CONTENT)
}

fn arrow_btn(text: &'static str, target: NaiveDate) -> impl WidgetView<AppModel> {
    button(
        label(text).text_size(11.0).color(CAL_ARROW),
        move |m: &mut AppModel| { m.calendar_selected_date = Some(target); },
    )
    .background_color(Color::TRANSPARENT)
    .border_color(Color::TRANSPARENT)
}

fn today_btn() -> impl WidgetView<AppModel> {
    button(
        label("*").text_size(10.0).color(CAL_ARROW),
        |m: &mut AppModel| {
            m.calendar_selected_date = Some(Local::now().date_naive());
        },
    )
    .background_color(Color::TRANSPARENT)
    .border_color(Color::TRANSPARENT)
}

fn build_time_picker(hour: u8, minute: u8) -> impl WidgetView<AppModel> {
    flex_row((
        flex_col((
            time_btn("^", true, true),
            label(format!("{:02}", hour)).text_size(13.0).color(TIME_TEXT),
            time_btn("v", true, false),
        ))
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .gap(0.0_f64.px()),
        label(":").text_size(13.0).color(TIME_TEXT),
        flex_col((
            time_btn("^", false, true),
            label(format!("{:02}", minute)).text_size(13.0).color(TIME_TEXT),
            time_btn("v", false, false),
        ))
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .gap(0.0_f64.px()),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Center)
    .gap(2.0_f64.px())
    .padding(4.0)
    .background_color(TIME_BG)
    .corner_radius(4.0)
}

fn time_btn(text: &'static str, is_hour: bool, is_up: bool) -> impl WidgetView<AppModel> {
    button(
        label(text).text_size(8.0).color(TIME_ARROW),
        move |m: &mut AppModel| {
            if is_hour {
                if is_up { m.calendar_hour = (m.calendar_hour + 1) % 24; }
                else { m.calendar_hour = if m.calendar_hour == 0 { 23 } else { m.calendar_hour - 1 }; }
            } else {
                if is_up { m.calendar_minute = (m.calendar_minute + 5) % 60; }
                else { m.calendar_minute = if m.calendar_minute < 5 { 55 } else { m.calendar_minute - 5 }; }
            }
        },
    )
    .background_color(Color::TRANSPARENT)
    .border_color(Color::TRANSPARENT)
}
