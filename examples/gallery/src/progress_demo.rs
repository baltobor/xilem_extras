//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Progress controls demo — linear progress bars in three styles.
//!
//! The round progress bar is added in the next phase; this file
//! grows to host it then.

use masonry::layout::AsUnit;
use xilem::masonry::peniko::Color;
use xilem::style::Style;
use xilem::view::{flex_col, flex_row, label, portal, CrossAxisAlignment};
use xilem::WidgetView;
use xilem_extras::progress::{progress_bar, round_progress, ProgressStyle};

use crate::app_model::AppModel;

const TEXT_COLOR: Color = Color::from_rgb8(220, 218, 214);
const TEXT_SECONDARY: Color = Color::from_rgb8(160, 156, 150);
const BLUE_TINT: Color = Color::from_rgb8(0x4A, 0x9E, 0xFF);
const VIOLET_TINT: Color = Color::from_rgb8(0xB0, 0x78, 0xFF);
const TEAL_TINT: Color = Color::from_rgb8(0x2A, 0xC8, 0xC8);

/// One labelled row showing a value rendered as a progress bar.
fn row(
    title: &'static str,
    value: f64,
    bar: impl WidgetView<AppModel> + 'static,
) -> impl WidgetView<AppModel> + 'static {
    flex_row((
        label(title.to_string())
            .text_size(12.0)
            .color(TEXT_COLOR),
        bar,
        label(format!("{:.0}%", value * 100.0))
            .text_size(11.0)
            .color(TEXT_SECONDARY),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Center)
    .gap(10.0_f64.px())
}

/// Section header.
fn section(text: &'static str) -> impl WidgetView<AppModel> + 'static {
    label(text.to_string())
        .text_size(13.0)
        .weight(xilem::FontWeight::BOLD)
        .color(TEXT_COLOR)
}

/// One labelled row showing a round progress widget. Caller
/// builds the widget so it can pick size / style freely.
fn round_row(
    title: &'static str,
    widget: impl WidgetView<AppModel> + 'static,
) -> impl WidgetView<AppModel> + 'static {
    flex_row((
        widget,
        label(title.to_string())
            .text_size(12.0)
            .color(TEXT_COLOR),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Center)
    .gap(10.0_f64.px())
}

/// Header + Gradient + Tint sections (one tuple-bound group).
fn group_a() -> impl WidgetView<AppModel> + 'static {
    flex_col((
        label("Linear progress bars".to_string())
            .text_size(16.0)
            .weight(xilem::FontWeight::BOLD)
            .color(TEXT_COLOR),
        label(
            "Three render styles. All use linear scaling — no log/dB transform."
                .to_string(),
        )
        .text_size(11.0)
        .color(TEXT_SECONDARY),
        section("Gradient (default) — three zones"),
        row("0.20", 0.20, progress_bar::<AppModel, ()>(0.20, 0.0, 1.0)),
        row("0.55", 0.55, progress_bar::<AppModel, ()>(0.55, 0.0, 1.0)),
        row("0.80", 0.80, progress_bar::<AppModel, ()>(0.80, 0.0, 1.0)),
        row("0.95", 0.95, progress_bar::<AppModel, ()>(0.95, 0.0, 1.0)),
        section("Tint — single colour, level-driven"),
        row("0.20", 0.20, progress_bar::<AppModel, ()>(0.20, 0.0, 1.0).tint()),
        row("0.55", 0.55, progress_bar::<AppModel, ()>(0.55, 0.0, 1.0).tint()),
        row("0.80", 0.80, progress_bar::<AppModel, ()>(0.80, 0.0, 1.0).tint()),
        row("0.95", 0.95, progress_bar::<AppModel, ()>(0.95, 0.0, 1.0).tint()),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Start)
    .gap(8.0_f64.px())
}

/// Monochrome + custom-size sections (second tuple-bound group).
fn group_b() -> impl WidgetView<AppModel> + 'static {
    flex_col((
        section("Monochrome — single fixed tint"),
        row("0.42 (blue)", 0.42, progress_bar::<AppModel, ()>(0.42, 0.0, 1.0).monochrome(BLUE_TINT)),
        row("0.66 (violet)", 0.66, progress_bar::<AppModel, ()>(0.66, 0.0, 1.0).monochrome(VIOLET_TINT)),
        row("0.88 (teal)", 0.88, progress_bar::<AppModel, ()>(0.88, 0.0, 1.0).monochrome(TEAL_TINT)),
        section("Custom size"),
        row(
            "200×10",
            0.5,
            progress_bar::<AppModel, ()>(0.5, 0.0, 1.0)
                .style(ProgressStyle::Tint)
                .main_axis_len(200.0)
                .cross_axis_len(10.0),
        ),
        row(
            "60×3",
            0.5,
            progress_bar::<AppModel, ()>(0.5, 0.0, 1.0)
                .monochrome(BLUE_TINT)
                .main_axis_len(60.0)
                .cross_axis_len(3.0),
        ),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Start)
    .gap(8.0_f64.px())
}

/// Round progress section — normal + small sizes, all three
/// styles, plus a busy spinner (animated rotating segment).
fn group_c() -> impl WidgetView<AppModel> + 'static {
    flex_col((
        section("Round progress — normal size"),
        round_row(
            "0.20 (gradient/tint)",
            round_progress::<AppModel, ()>(0.20, 0.0, 1.0),
        ),
        round_row(
            "0.55 (gradient/tint)",
            round_progress::<AppModel, ()>(0.55, 0.0, 1.0),
        ),
        round_row(
            "0.85 (gradient/tint)",
            round_progress::<AppModel, ()>(0.85, 0.0, 1.0),
        ),
        round_row(
            "0.42 monochrome (blue)",
            round_progress::<AppModel, ()>(0.42, 0.0, 1.0).monochrome(BLUE_TINT),
        ),
        round_row(
            "0.66 monochrome (violet)",
            round_progress::<AppModel, ()>(0.66, 0.0, 1.0).monochrome(VIOLET_TINT),
        ),
        section("Round progress — small (text height)"),
        round_row(
            "0.30",
            round_progress::<AppModel, ()>(0.30, 0.0, 1.0).small(),
        ),
        round_row(
            "0.75",
            round_progress::<AppModel, ()>(0.75, 0.0, 1.0).small(),
        ),
        round_row(
            "0.55 monochrome (teal)",
            round_progress::<AppModel, ()>(0.55, 0.0, 1.0)
                .small()
                .monochrome(TEAL_TINT),
        ),
        section("Busy / indeterminate"),
        round_row("normal busy", round_progress::<AppModel, ()>(0.0, 0.0, 1.0).busy()),
        round_row(
            "small busy",
            round_progress::<AppModel, ()>(0.0, 0.0, 1.0).small().busy(),
        ),
        round_row(
            "small busy (custom tint)",
            round_progress::<AppModel, ()>(0.0, 0.0, 1.0)
                .small()
                .tint(VIOLET_TINT)
                .busy(),
        ),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Start)
    .gap(8.0_f64.px())
}

/// Reversed-gradient section — same value range as group_a's
/// gradient/tint rows, but with `.reversed()` so high values
/// are green and low values are red. The motivating use case
/// is quality scores like SOLID, where 0.95 means "good" and
/// 0.20 means "bad" — the opposite of a level meter.
fn group_d() -> impl WidgetView<AppModel> + 'static {
    flex_col((
        section("Reversed gradient — low is warning, high is good"),
        row(
            "0.20",
            0.20,
            progress_bar::<AppModel, ()>(0.20, 0.0, 1.0).reversed(),
        ),
        row(
            "0.55",
            0.55,
            progress_bar::<AppModel, ()>(0.55, 0.0, 1.0).reversed(),
        ),
        row(
            "0.80",
            0.80,
            progress_bar::<AppModel, ()>(0.80, 0.0, 1.0).reversed(),
        ),
        row(
            "0.95",
            0.95,
            progress_bar::<AppModel, ()>(0.95, 0.0, 1.0).reversed(),
        ),
        section("Reversed tint — same input transform, single colour"),
        row(
            "0.20",
            0.20,
            progress_bar::<AppModel, ()>(0.20, 0.0, 1.0).tint().reversed(),
        ),
        row(
            "0.95",
            0.95,
            progress_bar::<AppModel, ()>(0.95, 0.0, 1.0).tint().reversed(),
        ),
        section("Reversed round — small (text height)"),
        round_row(
            "0.20 (small reversed)",
            round_progress::<AppModel, ()>(0.20, 0.0, 1.0).small().reversed(),
        ),
        round_row(
            "0.85 (small reversed)",
            round_progress::<AppModel, ()>(0.85, 0.0, 1.0).small().reversed(),
        ),
        round_row(
            "0.85 (normal reversed)",
            round_progress::<AppModel, ()>(0.85, 0.0, 1.0).reversed(),
        ),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Start)
    .gap(8.0_f64.px())
}

pub fn progress_demo(_model: &mut AppModel) -> impl WidgetView<AppModel> + use<> {
    // Wrap in `portal` so the long demo (linear sections + round
    // sections + busy spinners + reversed sections) can scroll
    // vertically when the gallery window is shorter than the
    // content.
    portal(
        flex_col((group_a(), group_b(), group_c(), group_d()))
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .gap(16.0_f64.px())
            .padding(16.0),
    )
}
