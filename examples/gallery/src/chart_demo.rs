//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Simple bar/line chart demo for the gallery.

use masonry::layout::AsUnit;
use xilem::masonry::peniko::Color;
use xilem::style::Style;
use xilem::view::{button, checkbox, flex_col, flex_row, label, CrossAxisAlignment, FlexExt};
use xilem::WidgetView;
use xilem_extras::chart::{chart, ChartMode};

use crate::app_model::AppModel;

const TEXT_COLOR: Color = Color::from_rgb8(220, 218, 214);
const TEXT_SECONDARY: Color = Color::from_rgb8(160, 156, 150);

/// Sample monthly sales data.
fn sample_monthly_data() -> (Vec<f64>, Vec<String>) {
    let values = vec![12.0, 19.0, 15.0, 25.0, 22.0, 30.0, 28.0, 35.0, 32.0, 40.0, 38.0, 45.0];
    let labels = vec!["J", "F", "M", "A", "M", "J", "J", "A", "S", "O", "N", "D"]
        .into_iter()
        .map(String::from)
        .collect();
    (values, labels)
}

/// Sample quarterly data.
fn sample_quarterly_data() -> (Vec<f64>, Vec<String>) {
    let values = vec![150.0, 220.0, 180.0, 310.0];
    let labels = vec!["Q1", "Q2", "Q3", "Q4"]
        .into_iter()
        .map(String::from)
        .collect();
    (values, labels)
}

/// Sample yearly data.
fn sample_yearly_data() -> (Vec<f64>, Vec<String>) {
    let values = vec![420.0, 580.0, 650.0, 720.0, 890.0];
    let labels = vec!["2021", "2022", "2023", "2024", "2025"]
        .into_iter()
        .map(String::from)
        .collect();
    (values, labels)
}

fn mode_from_index(index: usize) -> ChartMode {
    match index {
        0 => ChartMode::Bar,
        1 => ChartMode::Line,
        _ => ChartMode::Bar,
    }
}

fn mode_name(index: usize) -> &'static str {
    match index {
        0 => "Bar",
        1 => "Line",
        _ => "Bar",
    }
}

pub fn chart_demo(model: &mut AppModel) -> impl WidgetView<AppModel> + use<> {
    let mode = mode_from_index(model.chart_mode);
    let show_values = model.chart_show_values;

    let (monthly_values, monthly_labels) = sample_monthly_data();
    let (quarterly_values, quarterly_labels) = sample_quarterly_data();
    let (yearly_values, yearly_labels) = sample_yearly_data();

    flex_col((
        // Header
        label("Chart Widget")
            .text_size(18.0)
            .weight(xilem::FontWeight::BOLD)
            .color(TEXT_COLOR),
        label("Simple bar and line charts for data visualization")
            .text_size(12.0)
            .color(TEXT_SECONDARY),

        // Controls row
        flex_row((
            button(
                label("Bar").text_size(11.0),
                |m: &mut AppModel| m.chart_mode = 0,
            ),
            button(
                label("Line").text_size(11.0),
                |m: &mut AppModel| m.chart_mode = 1,
            ),
            checkbox("Show Values", model.chart_show_values, |m: &mut AppModel, checked| {
                m.chart_show_values = checked;
            }),
        ))
        .gap(8.px()),

        // Mode indicator
        label(format!("Mode: {}", mode_name(model.chart_mode)))
            .text_size(12.0)
            .color(TEXT_SECONDARY),

        // Charts grid - 3 charts showing different data
        flex_row((
            chart::<AppModel, ()>(
                "Monthly Sales",
                &monthly_values,
                monthly_labels,
                mode,
            )
            .show_values(show_values)
            .flex(1.0),

            chart::<AppModel, ()>(
                "Quarterly Revenue",
                &quarterly_values,
                quarterly_labels,
                mode,
            )
            .show_values(show_values)
            .flex(1.0),

            chart::<AppModel, ()>(
                "Yearly Growth",
                &yearly_values,
                yearly_labels,
                mode,
            )
            .show_values(show_values)
            .flex(1.0),
        ))
        .cross_axis_alignment(CrossAxisAlignment::Stretch)
        .gap(8.px())
        .flex(1.0),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Stretch)
    .gap(12.px())
    .padding(16.0)
}
