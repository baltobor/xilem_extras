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
use xilem_extras::progress::{busy_hex, progress_bar, round_progress, ProgressStyle};
use xilem_extras::Theme;

use crate::app_model::AppModel;
const BLUE_TINT: Color = Color::from_rgb8(0x4A, 0x9E, 0xFF);
const VIOLET_TINT: Color = Color::from_rgb8(0xB0, 0x78, 0xFF);
const TEAL_TINT: Color = Color::from_rgb8(0x2A, 0xC8, 0xC8);

/// One labelled row showing a value rendered as a progress bar.
fn row(
    title: &'static str,
    value: f64,
    bar: impl WidgetView<AppModel> + 'static,
    theme: Theme,
) -> impl WidgetView<AppModel> + 'static {
    flex_row((
        label(title.to_string())
            .text_size(12.0)
            .color(theme.text()),
        bar,
        label(format!("{:.0}%", value * 100.0))
            .text_size(11.0)
            .color(theme.text_secondary()),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Center)
    .gap(10.0_f64.px())
}

/// Section header.
fn section(text: &'static str, theme: Theme) -> impl WidgetView<AppModel> + 'static {
    label(text.to_string())
        .text_size(13.0)
        .weight(xilem::FontWeight::BOLD)
        .color(theme.text())
}

/// One labelled row showing a round progress widget. Caller
/// builds the widget so it can pick size / style freely.
fn round_row(
    title: &'static str,
    widget: impl WidgetView<AppModel> + 'static,
    theme: Theme,
) -> impl WidgetView<AppModel> + 'static {
    flex_row((
        widget,
        label(title.to_string())
            .text_size(12.0)
            .color(theme.text()),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Center)
    .gap(10.0_f64.px())
}

/// Header + Gradient + Tint sections (one tuple-bound group).
fn group_a(theme: Theme) -> impl WidgetView<AppModel> + 'static {
    flex_col((
        label("Linear progress bars".to_string())
            .text_size(16.0)
            .weight(xilem::FontWeight::BOLD)
            .color(theme.text()),
        label(
            "Three render styles. All use linear scaling."
                .to_string(),
        )
        .text_size(11.0)
        .color(theme.text_secondary()),
        section("Gradient (default) — three zones", theme),
        row("0.20", 0.20, progress_bar::<AppModel, ()>(0.20, 0.0, 1.0), theme),
        row("0.55", 0.55, progress_bar::<AppModel, ()>(0.55, 0.0, 1.0), theme),
        row("0.80", 0.80, progress_bar::<AppModel, ()>(0.80, 0.0, 1.0), theme),
        row("0.95", 0.95, progress_bar::<AppModel, ()>(0.95, 0.0, 1.0), theme),
        section("Tint — single colour, level-driven", theme),
        row("0.20", 0.20, progress_bar::<AppModel, ()>(0.20, 0.0, 1.0).tint(), theme),
        row("0.55", 0.55, progress_bar::<AppModel, ()>(0.55, 0.0, 1.0).tint(), theme),
        row("0.80", 0.80, progress_bar::<AppModel, ()>(0.80, 0.0, 1.0).tint(), theme),
        row("0.95", 0.95, progress_bar::<AppModel, ()>(0.95, 0.0, 1.0).tint(), theme),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Start)
    .gap(8.0_f64.px())
}

/// Monochrome + custom-size sections (second tuple-bound group).
fn group_b(theme: Theme) -> impl WidgetView<AppModel> + 'static {
    flex_col((
        section("Monochrome — single fixed tint", theme),
        row("0.42 (blue)", 0.42, progress_bar::<AppModel, ()>(0.42, 0.0, 1.0).monochrome(BLUE_TINT), theme),
        row("0.66 (violet)", 0.66, progress_bar::<AppModel, ()>(0.66, 0.0, 1.0).monochrome(VIOLET_TINT), theme),
        row("0.88 (teal)", 0.88, progress_bar::<AppModel, ()>(0.88, 0.0, 1.0).monochrome(TEAL_TINT), theme),
        section("Custom size", theme),
        row(
            "200×10",
            0.5,
            progress_bar::<AppModel, ()>(0.5, 0.0, 1.0)
                .style(ProgressStyle::Tint)
                .main_axis_len(200.0)
                .cross_axis_len(10.0),
            theme,
        ),
        row(
            "60×3",
            0.5,
            progress_bar::<AppModel, ()>(0.5, 0.0, 1.0)
                .monochrome(BLUE_TINT)
                .main_axis_len(60.0)
                .cross_axis_len(3.0),
            theme,
        ),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Start)
    .gap(8.0_f64.px())
}

/// Round progress section — normal + small sizes, all three
/// styles, plus a busy spinner (animated rotating segment).
fn group_c(theme: Theme) -> impl WidgetView<AppModel> + 'static {
    flex_col((
        section("Round progress — normal size", theme),
        round_row(
            "0.20 (gradient/tint)",
            round_progress::<AppModel, ()>(0.20, 0.0, 1.0),
            theme,
        ),
        round_row(
            "0.55 (gradient/tint)",
            round_progress::<AppModel, ()>(0.55, 0.0, 1.0),
            theme,
        ),
        round_row(
            "0.85 (gradient/tint)",
            round_progress::<AppModel, ()>(0.85, 0.0, 1.0),
            theme,
        ),
        round_row(
            "0.42 monochrome (blue)",
            round_progress::<AppModel, ()>(0.42, 0.0, 1.0).monochrome(BLUE_TINT),
            theme,
        ),
        round_row(
            "0.66 monochrome (violet)",
            round_progress::<AppModel, ()>(0.66, 0.0, 1.0).monochrome(VIOLET_TINT),
            theme,
        ),
        section("Round progress — small (text height)", theme),
        round_row(
            "0.30",
            round_progress::<AppModel, ()>(0.30, 0.0, 1.0).small(),
            theme,
        ),
        round_row(
            "0.75",
            round_progress::<AppModel, ()>(0.75, 0.0, 1.0).small(),
            theme,
        ),
        round_row(
            "0.55 monochrome (teal)",
            round_progress::<AppModel, ()>(0.55, 0.0, 1.0)
                .small()
                .monochrome(TEAL_TINT),
            theme,
        ),
        section("Busy / indeterminate (hexagonal beehive)", theme),
        round_row("normal busy", busy_hex::<AppModel, ()>(), theme),
        round_row("small busy", busy_hex::<AppModel, ()>().small(), theme),
        round_row(
            "small busy (custom tint)",
            busy_hex::<AppModel, ()>().small().tint(VIOLET_TINT),
            theme,
        ),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Start)
    .gap(8.0_f64.px())
}

/// Reversed-gradient section — same value range as group_a's
/// gradient/tint rows, but with `.reversed()` so high values
/// are green and low values are red.
fn group_d(theme: Theme) -> impl WidgetView<AppModel> + 'static {
    flex_col((
        section("Reversed gradient — low is warning, high is good", theme),
        row("0.20", 0.20, progress_bar::<AppModel, ()>(0.20, 0.0, 1.0).reversed(), theme),
        row("0.55", 0.55, progress_bar::<AppModel, ()>(0.55, 0.0, 1.0).reversed(), theme),
        row("0.80", 0.80, progress_bar::<AppModel, ()>(0.80, 0.0, 1.0).reversed(), theme),
        row("0.95", 0.95, progress_bar::<AppModel, ()>(0.95, 0.0, 1.0).reversed(), theme),
        section("Reversed tint — same input transform, single colour", theme),
        row("0.20", 0.20, progress_bar::<AppModel, ()>(0.20, 0.0, 1.0).tint().reversed(), theme),
        row("0.95", 0.95, progress_bar::<AppModel, ()>(0.95, 0.0, 1.0).tint().reversed(), theme),
        section("Reversed round — small (text height)", theme),
        round_row(
            "0.20 (small reversed)",
            round_progress::<AppModel, ()>(0.20, 0.0, 1.0).small().reversed(),
            theme,
        ),
        round_row(
            "0.85 (small reversed)",
            round_progress::<AppModel, ()>(0.85, 0.0, 1.0).small().reversed(),
            theme,
        ),
        round_row(
            "0.85 (normal reversed)",
            round_progress::<AppModel, ()>(0.85, 0.0, 1.0).reversed(),
            theme,
        ),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Start)
    .gap(8.0_f64.px())
}

pub fn progress_demo(model: &mut AppModel) -> impl WidgetView<AppModel> + use<> {
    let theme = Theme::from_dark(model.dark_mode);
    portal(
        flex_col((group_a(theme), group_b(theme), group_c(theme), group_d(theme)))
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .gap(16.0_f64.px())
            .padding(16.0)
            .background_color(theme.page_bg()),
    )
    .constrain_horizontal(true)
    .must_fill(true)
}
