//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Widgets demo page — text input, secure text input, boolean
//! controls (classic / switch / radio), the SwiftUI-style Form,
//! a `param_selector` and the modal-overlay button. The page is
//! long; wrap in `portal` so it scrolls.
//!
//! All themable widgets read from `xilem_extras::Theme` derived
//! from `model.dark_mode` — there are no per-widget colour
//! patches in this file. The "Dark mode" switch in the form
//! flips `model.dark_mode`, which the gallery picks up on the
//! next render and threads through the entire view tree.

use masonry::layout::AsUnit;
use xilem::style::Style;
use xilem::view::{button, flex_col, label, portal, CrossAxisAlignment};
use xilem::WidgetView;
use xilem_extras::{
    form_themed, form_section_themed, form_row_themed,
    form_toggle_themed, form_radio_themed,
    param_selector, styled_check_colored,
    styled_text_input_with_placeholder, styled_text_input_colored,
    styled_secure_text_input,
    CheckboxStyle, LabelAlign, TextInputColors, Theme,
};

use crate::app_model::AppModel;

pub fn widgets_demo(model: &mut AppModel) -> impl WidgetView<AppModel> + use<> {
    let theme = Theme::from_dark(model.dark_mode);
    let text_fg = theme.text();
    let text_fg2 = theme.text_secondary();
    let bg_content = theme.page_bg();
    let bg_section = theme.section_bg();

    // `constrain_horizontal(true).must_fill(true)` makes the
    // portal hand the full pane width down to the inner column,
    // which then stretches its children — including the form —
    // to fill horizontally.
    portal(
        flex_col((
            label("Styled Widgets")
                .text_size(16.0)
                .weight(xilem::FontWeight::BOLD)
                .color(text_fg),

            // Text Input Section
            flex_col((
                label("Text Input").text_size(13.0).color(text_fg),
                label("Light mode (explicit colors)").text_size(10.0).color(text_fg2),
                styled_text_input_with_placeholder(
                    model.widgets_text_light.clone(),
                    "Enter text...",
                    |m: &mut AppModel, value: String| {
                        m.widgets_text_light = value;
                    },
                ),
                label("Dark mode (explicit colors)").text_size(10.0).color(text_fg2),
                styled_text_input_colored(
                    model.widgets_text_dark.clone(),
                    "Dark input...",
                    TextInputColors::dark(),
                    |m: &mut AppModel, value: String| {
                        m.widgets_text_dark = value;
                    },
                ),
                label("Secure (password) — bullets in place of characters")
                    .text_size(10.0).color(text_fg2),
                styled_secure_text_input(
                    model.widgets_password.clone(),
                    "Password",
                    |m: &mut AppModel, value: String| {
                        m.widgets_password = value;
                    },
                ),
                label(format!("Captured length: {}", model.widgets_password.chars().count()))
                    .text_size(10.0).color(text_fg2),
            ))
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .gap(6.0_f64.px())
            .padding(10.0)
            .background_color(bg_section)
            .corner_radius(6.0),

            // Boolean control styles, all theme-driven.
            flex_col((
                label("Boolean controls — Classic / Switch / Radio")
                    .text_size(13.0).color(text_fg),
                label("Same call shape, swappable visual style").text_size(10.0).color(text_fg2),

                styled_check_colored(
                    CheckboxStyle::Classic,
                    "Classic checkmark",
                    model.widgets_checkbox_1,
                    theme.checkbox_colors(),
                    |m: &mut AppModel, checked: bool| {
                        m.widgets_checkbox_1 = checked;
                    },
                ),

                styled_check_colored(
                    CheckboxStyle::Switch,
                    "On/off switch",
                    model.widgets_checkbox_2,
                    theme.checkbox_colors(),
                    |m: &mut AppModel, checked: bool| {
                        m.widgets_checkbox_2 = checked;
                    },
                ),

                styled_check_colored(
                    CheckboxStyle::Radio,
                    "Radio button",
                    model.widgets_radio_demo,
                    theme.checkbox_colors(),
                    |m: &mut AppModel, _checked: bool| {
                        m.widgets_radio_demo = !m.widgets_radio_demo;
                    },
                ),
            ))
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .gap(6.0_f64.px())
            .padding(10.0)
            .background_color(bg_section)
            .corner_radius(6.0),

            // Form
            label("Form")
                .text_size(13.0).color(text_fg),
            label("`form((...))` swaps in for `flex_col((...))` and adds form chrome")
                .text_size(10.0).color(text_fg2),
            form_themed(
                (
                    form_section_themed("Notifications", (
                        form_toggle_themed(
                            "Play notification sounds",
                            model.form_play_sounds,
                            theme,
                            |m: &mut AppModel, v| m.form_play_sounds = v,
                        ),
                        form_toggle_themed(
                            "Send read receipts",
                            model.form_read_receipts,
                            theme,
                            |m: &mut AppModel, v| m.form_read_receipts = v,
                        ),
                    ), theme),
                    form_section_themed("Notify Me About (radio group)", (
                        form_radio_themed(
                            "Direct messages",
                            model.form_notify_about == 0,
                            theme,
                            |m: &mut AppModel, _| m.form_notify_about = 0,
                        ),
                        form_radio_themed(
                            "Mentions",
                            model.form_notify_about == 1,
                            theme,
                            |m: &mut AppModel, _| m.form_notify_about = 1,
                        ),
                        form_radio_themed(
                            "Anything",
                            model.form_notify_about == 2,
                            theme,
                            |m: &mut AppModel, _| m.form_notify_about = 2,
                        ),
                    ), theme),
                    form_section_themed("Notify Me About (param selector)", (
                        param_selector(
                            vec![
                                "Direct messages".to_string(),
                                "Mentions".to_string(),
                                "Anything".to_string(),
                            ],
                            model.form_notify_about_alt,
                            |m: &mut AppModel, idx: usize| {
                                m.form_notify_about_alt = idx;
                            },
                        )
                        .label_align(LabelAlign::Left)
                        .label_colors(theme.text(), theme.text_secondary()),
                    ), theme),
                    form_section_themed("Notify Me About (param selector — fishgrid)", (
                        param_selector(
                            vec![
                                "Direct messages".to_string(),
                                "Mentions".to_string(),
                                "Anything".to_string(),
                            ],
                            model.form_notify_about_alt,
                            |m: &mut AppModel, idx: usize| {
                                m.form_notify_about_alt = idx;
                            },
                        )
                        .label_align(LabelAlign::Alternating)
                        .label_colors(theme.text(), theme.text_secondary()),
                    ), theme),
                    form_section_themed("Display", (
                        form_row_themed(
                            "Dark mode",
                            styled_check_colored(
                                CheckboxStyle::Switch,
                                "",
                                model.dark_mode,
                                theme.checkbox_colors(),
                                |m: &mut AppModel, v| m.dark_mode = v,
                            ),
                            theme,
                        ),
                    ), theme),
                ),
                theme,
            ),

            // Modal Section
            flex_col((
                label("Modal Overlay").text_size(13.0).color(text_fg),
                label("Click to show modal overlay").text_size(10.0).color(text_fg2),
                button(
                    label("Show Modal").text_size(12.0),
                    |m: &mut AppModel| {
                        m.widgets_show_sheet = true;
                    },
                ),
            ))
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .gap(6.0_f64.px())
            .padding(10.0)
            .background_color(bg_section)
            .corner_radius(6.0),
        ))
        .cross_axis_alignment(CrossAxisAlignment::Stretch)
        .gap(12.0_f64.px())
        .padding(12.0)
        .background_color(bg_content),
    )
    .constrain_horizontal(true)
    .must_fill(true)
}
