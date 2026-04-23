//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Widgets demo page - styled_text_input, styled_checkbox, modal overlay.

use masonry::layout::AsUnit;
use masonry::peniko::color::AlphaColor;
use xilem::masonry::peniko::Color;
use xilem::style::Style;
use xilem::view::{button, flex_col, label, CrossAxisAlignment};
use xilem::WidgetView;
use xilem_extras::{
    styled_text_input_with_placeholder, styled_text_input_colored,
    styled_checkbox_colored,
    TextInputColors, CheckboxColors,
};

use crate::app_model::AppModel;

// Demo page colors
const TEXT_COLOR: Color = Color::from_rgb8(220, 218, 214);
const TEXT_SECONDARY: Color = Color::from_rgb8(160, 156, 150);
const BG_CONTENT: Color = Color::from_rgb8(35, 33, 30);
const BG_SECTION: Color = Color::from_rgb8(45, 43, 40);

pub fn widgets_demo(model: &mut AppModel) -> impl WidgetView<AppModel> + use<> {
    flex_col((
        label("Styled Widgets")
            .text_size(16.0)
            .weight(xilem::FontWeight::BOLD)
            .color(TEXT_COLOR),

        // Text Input Section
        flex_col((
            label("Text Input").text_size(13.0).color(TEXT_COLOR),
            label("Light mode (explicit colors)").text_size(10.0).color(TEXT_SECONDARY),
            styled_text_input_with_placeholder(
                model.widgets_text_light.clone(),
                "Enter text...",
                |model: &mut AppModel, value: String| {
                    model.widgets_text_light = value;
                },
            ),
            label("Dark mode (explicit colors)").text_size(10.0).color(TEXT_SECONDARY),
            styled_text_input_colored(
                model.widgets_text_dark.clone(),
                "Dark input...",
                TextInputColors::dark(),
                |model: &mut AppModel, value: String| {
                    model.widgets_text_dark = value;
                },
            ),
        ))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .gap(6.0_f64.px())
        .padding(10.0)
        .background_color(BG_SECTION)
        .corner_radius(6.0),

        // Checkbox Section
        flex_col((
            label("Checkbox").text_size(13.0).color(TEXT_COLOR),
            label("Light mode (black checkmark, black label)").text_size(10.0).color(TEXT_SECONDARY),
            styled_checkbox_colored(
                "Enable feature",
                model.widgets_checkbox_1,
                CheckboxColors::custom(
                    AlphaColor::new([0.0, 0.0, 0.0, 0.0]),  // transparent background
                    AlphaColor::new([0.3, 0.3, 0.3, 1.0]),  // dark gray border
                    AlphaColor::new([0.0, 0.0, 0.0, 1.0]),  // black checkmark
                    Color::BLACK,
                ),
                |model: &mut AppModel, checked: bool| {
                    model.widgets_checkbox_1 = checked;
                },
            ),
            label("Dark mode (white checkmark, white label)").text_size(10.0).color(TEXT_SECONDARY),
            styled_checkbox_colored(
                "Dark mode option",
                model.widgets_checkbox_2,
                CheckboxColors::dark(),
                |model: &mut AppModel, checked: bool| {
                    model.widgets_checkbox_2 = checked;
                },
            ),
        ))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .gap(6.0_f64.px())
        .padding(10.0)
        .background_color(BG_SECTION)
        .corner_radius(6.0),

        // Modal Section
        flex_col((
            label("Modal Overlay").text_size(13.0).color(TEXT_COLOR),
            label("Click to show modal overlay").text_size(10.0).color(TEXT_SECONDARY),
            button(
                label("Show Modal").text_size(12.0),
                |model: &mut AppModel| {
                    model.widgets_show_sheet = true;
                },
            ),
        ))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .gap(6.0_f64.px())
        .padding(10.0)
        .background_color(BG_SECTION)
        .corner_radius(6.0),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Start)
    .gap(12.0_f64.px())
    .padding(12.0)
    .background_color(BG_CONTENT)
}
