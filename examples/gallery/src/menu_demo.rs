//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Menu button demo showing pulldown menus.

use masonry::layout::AsUnit;
use xilem::masonry::vello::peniko::Color;
use xilem::style::{Style, Padding};
use xilem::view::{flex_col, flex_row, label};
use xilem::WidgetView;

use xilem_extras::{menu_button, dropdown_select};
use xilem_extras::menu_button::DEFAULT_ITEM_HEIGHT;

use crate::app_model::AppModel;

const TEXT_COLOR: Color = Color::from_rgb8(220, 218, 214);
const TEXT_SECONDARY: Color = Color::from_rgb8(160, 156, 150);
const MENU_BAR_BG: Color = Color::from_rgb8(45, 43, 40);

/// Font size derived from menu height.
const MENU_TEXT_SIZE: f32 = (DEFAULT_ITEM_HEIGHT * 0.43) as f32;
/// Vertical padding to achieve DEFAULT_ITEM_HEIGHT.
const MENU_PADDING_V: f64 = (DEFAULT_ITEM_HEIGHT - MENU_TEXT_SIZE as f64) / 2.0;
const MENU_PADDING_H: f64 = 8.0;

fn menu_bar_button(text: &str) -> impl WidgetView<AppModel> + use<'_> {
    label(text.to_string())
        .text_size(MENU_TEXT_SIZE)
        .color(TEXT_COLOR)
        .padding(Padding {
            top: MENU_PADDING_V,
            bottom: MENU_PADDING_V,
            left: MENU_PADDING_H,
            right: MENU_PADDING_H,
        })
}

pub fn menu_demo(model: &mut AppModel) -> impl WidgetView<AppModel> + use<'_> {
    // Menu bar with File, View, About menus
    let menu_bar = flex_row((
        menu_button(
            menu_bar_button("File"),
            vec![
                "New".to_string(),
                "Open...".to_string(),
                "---".to_string(),
                "Save".to_string(),
                "Save As...".to_string(),
                "---".to_string(),
                "Exit".to_string(),
            ],
            |model: &mut AppModel, index: usize| {
                model.menu_last_action = match index {
                    0 => "File > New".to_string(),
                    1 => "File > Open...".to_string(),
                    // index 2 is separator
                    3 => "File > Save".to_string(),
                    4 => "File > Save As...".to_string(),
                    // index 5 is separator
                    6 => "File > Exit".to_string(),
                    _ => format!("File > Item {}", index),
                };
            },
        ),
        menu_button(
            menu_bar_button("Edit"),
            vec![
                "Undo".to_string(),
                "Redo".to_string(),
                "---".to_string(),
                "Cut".to_string(),
                "Copy".to_string(),
                "Paste".to_string(),
            ],
            |model: &mut AppModel, index: usize| {
                model.menu_last_action = match index {
                    0 => "Edit > Undo".to_string(),
                    1 => "Edit > Redo".to_string(),
                    // index 2 is separator
                    3 => "Edit > Cut".to_string(),
                    4 => "Edit > Copy".to_string(),
                    5 => "Edit > Paste".to_string(),
                    _ => format!("Edit > Item {}", index),
                };
            },
        ),
        menu_button(
            menu_bar_button("View"),
            vec![
                "Zoom In".to_string(),
                "Zoom Out".to_string(),
                "Reset Zoom".to_string(),
                "Full Screen".to_string(),
            ],
            |model: &mut AppModel, index: usize| {
                model.menu_last_action = match index {
                    0 => "View > Zoom In".to_string(),
                    1 => "View > Zoom Out".to_string(),
                    2 => "View > Reset Zoom".to_string(),
                    3 => "View > Full Screen".to_string(),
                    _ => format!("View > Item {}", index),
                };
            },
        ),
        menu_button(
            menu_bar_button("Help"),
            vec![
                "Documentation".to_string(),
                "About".to_string(),
            ],
            |model: &mut AppModel, index: usize| {
                model.menu_last_action = match index {
                    0 => "Help > Documentation".to_string(),
                    1 => "Help > About".to_string(),
                    _ => format!("Help > Item {}", index),
                };
            },
        ),
    ))
    .gap(0.px())
    .height((DEFAULT_ITEM_HEIGHT as i32).px())
    .background_color(MENU_BAR_BG);

    flex_col((
        label("Menu Button Demo")
            .text_size(16.0)
            .weight(xilem::FontWeight::BOLD)
            .color(TEXT_COLOR),
        label("Click on File, Edit, View, or Help to open pulldown menus")
            .text_size(12.0)
            .color(TEXT_SECONDARY),

        // Menu bar
        menu_bar,

        // Dropdown Select section
        flex_col((
            label("Dropdown Select".to_string())
                .text_size(14.0)
                .weight(xilem::FontWeight::BOLD)
                .color(TEXT_COLOR),
            label("Select from a list of options:".to_string())
                .text_size(12.0)
                .color(TEXT_SECONDARY),
            dropdown_select(
                model.dropdown_selected_index,
                vec![
                    "Small".to_string(),
                    "Medium".to_string(),
                    "Large".to_string(),
                    "Extra Large".to_string(),
                ],
                |model: &mut AppModel, value: &str, index: usize| {
                    model.dropdown_selected_index = index;
                    model.menu_last_action = format!("Selected: {} (index {})", value, index);
                },
            ),
        ))
        .gap(8.px())
        .padding(16.0),

        // Status area
        flex_col((
            label("Last action:".to_string())
                .text_size(13.0)
                .color(TEXT_SECONDARY),
            label(model.menu_last_action.clone())
                .text_size(14.0)
                .color(TEXT_COLOR),
        ))
        .gap(4.px())
        .padding(16.0),
    ))
    .gap(8.px())
    .padding(16.0)
}
