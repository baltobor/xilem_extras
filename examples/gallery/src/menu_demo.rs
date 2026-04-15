//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Menu button demo showing pulldown menus.

use masonry::layout::AsUnit;
use xilem::masonry::peniko::Color;
use xilem::style::{Style, Padding};
use xilem::view::{flex_col, flex_row, label};
use xilem::WidgetView;

use xilem_extras::{menu_button, menu_item, separator, submenu, dropdown_select};
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
    // Menu bar with File, View, About menus using the new type-safe API
    let menu_bar = flex_row((
        menu_button(
            menu_bar_button("File"),
            (
                menu_item("New", |model: &mut AppModel| {
                    model.menu_last_action = "File > New".to_string();
                }),
                menu_item("Open...", |model: &mut AppModel| {
                    model.menu_last_action = "File > Open...".to_string();
                }),
                separator(),
                menu_item("Save", |model: &mut AppModel| {
                    model.menu_last_action = "File > Save".to_string();
                }),
                menu_item("Save As...", |model: &mut AppModel| {
                    model.menu_last_action = "File > Save As...".to_string();
                }),
                separator(),
                menu_item("Exit", |model: &mut AppModel| {
                    model.menu_last_action = "File > Exit".to_string();
                }),
            ),
        ),
        menu_button(
            menu_bar_button("Edit"),
            (
                menu_item("Undo", |model: &mut AppModel| {
                    model.menu_last_action = "Edit > Undo".to_string();
                }),
                menu_item("Redo", |model: &mut AppModel| {
                    model.menu_last_action = "Edit > Redo".to_string();
                }),
                separator(),
                menu_item("Cut", |model: &mut AppModel| {
                    model.menu_last_action = "Edit > Cut".to_string();
                }),
                menu_item("Copy", |model: &mut AppModel| {
                    model.menu_last_action = "Edit > Copy".to_string();
                }),
                menu_item("Paste", |model: &mut AppModel| {
                    model.menu_last_action = "Edit > Paste".to_string();
                }),
                separator(),
                // Submenu example (visual placeholder - click shows ">" indicator)
                submenu("Transform", (
                    menu_item("Uppercase", |model: &mut AppModel| {
                        model.menu_last_action = "Edit > Transform > Uppercase".to_string();
                    }),
                    menu_item("Lowercase", |model: &mut AppModel| {
                        model.menu_last_action = "Edit > Transform > Lowercase".to_string();
                    }),
                    menu_item("Title Case", |model: &mut AppModel| {
                        model.menu_last_action = "Edit > Transform > Title Case".to_string();
                    }),
                )),
            ),
        ),
        menu_button(
            menu_bar_button("View"),
            (
                menu_item("Zoom In", |model: &mut AppModel| {
                    model.menu_last_action = "View > Zoom In".to_string();
                }),
                menu_item("Zoom Out", |model: &mut AppModel| {
                    model.menu_last_action = "View > Zoom Out".to_string();
                }),
                menu_item("Reset Zoom", |model: &mut AppModel| {
                    model.menu_last_action = "View > Reset Zoom".to_string();
                }),
                separator(),
                // Checkmark items
                menu_item("Dark Mode", |model: &mut AppModel| {
                    model.dark_mode = !model.dark_mode;
                    model.menu_last_action = format!("View > Dark Mode: {}", model.dark_mode);
                }).checked(model.dark_mode),
                menu_item("Show Toolbar", |model: &mut AppModel| {
                    model.show_toolbar = !model.show_toolbar;
                    model.menu_last_action = format!("View > Show Toolbar: {}", model.show_toolbar);
                }).checked(model.show_toolbar),
                separator(),
                menu_item("Full Screen", |model: &mut AppModel| {
                    model.menu_last_action = "View > Full Screen".to_string();
                }),
            ),
        ),
        menu_button(
            menu_bar_button("Help"),
            (
                menu_item("Documentation", |model: &mut AppModel| {
                    model.menu_last_action = "Help > Documentation".to_string();
                }),
                separator(),
                menu_item("About", |model: &mut AppModel| {
                    model.menu_last_action = "Help > About".to_string();
                }),
            ),
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
