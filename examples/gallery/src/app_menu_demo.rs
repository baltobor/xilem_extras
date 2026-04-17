//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! App Menu Bar demo showing the unified cross-platform menu API.

use masonry::layout::AsUnit;
use xilem::masonry::peniko::Color;
use xilem::style::Style;
use xilem::view::{flex_col, label};
use xilem::WidgetView;

use crate::app_model::AppModel;

const TEXT_COLOR: Color = Color::from_rgb8(220, 218, 214);
const TEXT_SECONDARY: Color = Color::from_rgb8(160, 156, 150);
const CODE_BG: Color = Color::from_rgb8(35, 33, 30);

fn section_header(text: &str) -> impl WidgetView<AppModel> + use<'_> {
    label(text.to_string())
        .text_size(14.0)
        .weight(xilem::FontWeight::BOLD)
        .color(TEXT_COLOR)
}

fn body_text(text: &str) -> impl WidgetView<AppModel> + use<'_> {
    label(text.to_string())
        .text_size(12.0)
        .color(TEXT_SECONDARY)
}

fn code_block(code: &str) -> impl WidgetView<AppModel> + use<'_> {
    label(code.to_string())
        .text_size(11.0)
        .color(Color::from_rgb8(180, 220, 180))
        .padding(12.0)
        .background_color(CODE_BG)
}

pub fn app_menu_demo(model: &mut AppModel) -> impl WidgetView<AppModel> + use<'_> {
    let overview = flex_col((
        body_text("The app_menu module provides a declarative API for menus:"),
        body_text("  - macOS/Windows: Native menus via muda crate"),
        body_text("  - Linux: Fallback using menu_button widgets"),
    )).gap(2.px());

    let features = flex_col((
        body_text("  - Fluent builder API inspired by SwiftUI Commands"),
        body_text("  - Keyboard shortcuts: CMD + Key::N"),
        body_text("  - .enabled() and .checked() for state-driven items"),
        body_text("  - Nested submenus via .submenu()"),
    )).gap(2.px());

    let usage_code = code_block(r#"use xilem_extras::{menu_button, menu_item, separator, submenu};

// Build menu bar with menu_button widgets
flex_row((
    menu_button(label("File"), (
        menu_item("New", |s| s.new_file()),
        separator(),
        menu_item("Quit", |s| s.quit()),
    )),
    menu_button(label("Edit"), (
        menu_item("Undo", |s| s.undo()),
        menu_item("Redo", |s| s.redo()),
    )),
))"#);

    flex_col((
        label("App Menu Bar (xilem_muda)")
            .text_size(18.0)
            .weight(xilem::FontWeight::BOLD)
            .color(TEXT_COLOR),
        body_text("Unified cross-platform application menu bar API"),
        section_header("Overview"),
        overview,
        section_header("Usage"),
        usage_code,
        section_header("Features"),
        features,
        section_header("This Gallery"),
        body_text("The menu bar at the top uses menu_button widgets."),
        body_text("Use the Gallery menu to navigate between pages."),
        flex_col((
            body_text("Last menu action:"),
            label(model.menu_last_action.clone())
                .text_size(14.0)
                .color(TEXT_COLOR),
        )).gap(4.px()),
    ))
    .gap(8.px())
    .padding(16.0)
}
