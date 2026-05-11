//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! App Menu Bar demo showing the unified cross-platform menu API.

use masonry::layout::{AsUnit, Length};
use xilem::masonry::peniko::Color;
use xilem::style::Style;
use xilem::view::{flex_col, label};
use xilem::WidgetView;
use xilem_extras::Theme;

use crate::app_model::AppModel;

fn section_header(text: &str, theme: Theme) -> impl WidgetView<AppModel> + use<'_> {
    label(text.to_string())
        .text_size(14.0)
        .weight(xilem::FontWeight::BOLD)
        .color(theme.text())
}

fn body_text(text: &str, theme: Theme) -> impl WidgetView<AppModel> + use<'_> {
    label(text.to_string())
        .text_size(12.0)
        .color(theme.text_secondary())
}

fn code_block(code: &str, theme: Theme) -> impl WidgetView<AppModel> + use<'_> {
    label(code.to_string())
        .text_size(11.0)
        .color(Color::from_rgb8(180, 220, 180))
        .padding(Length::px(12.0))
        .background_color(theme.section_bg())
}

pub fn app_menu_demo(model: &mut AppModel) -> impl WidgetView<AppModel> + use<'_> {
    let theme = Theme::from_dark(model.dark_mode);

    let overview = flex_col((
        body_text("The app_menu module provides a declarative API for menus:", theme),
        body_text("  - macOS/Windows: Native menus via muda crate", theme),
        body_text("  - Linux: Fallback using menu_button widgets", theme),
    )).gap(2.px());

    let features = flex_col((
        body_text("  - Fluent builder API inspired by SwiftUI Commands", theme),
        body_text("  - Keyboard shortcuts: CMD + Key::N", theme),
        body_text("  - .enabled() and .checked() for state-driven items", theme),
        body_text("  - Nested submenus via .submenu()", theme),
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
))"#, theme);

    flex_col((
        label("App Menu Bar (xilem_muda)")
            .text_size(18.0)
            .weight(xilem::FontWeight::BOLD)
            .color(theme.text()),
        body_text("Unified cross-platform application menu bar API", theme),
        section_header("Overview", theme),
        overview,
        section_header("Usage", theme),
        usage_code,
        section_header("Features", theme),
        features,
        section_header("This Gallery", theme),
        body_text("The menu bar at the top uses menu_button widgets.", theme),
        body_text("Use the Gallery menu to navigate between pages.", theme),
        flex_col((
            body_text("Last menu action:", theme),
            label(model.menu_last_action.clone())
                .text_size(14.0)
                .color(theme.text()),
        )).gap(4.px()),
    ))
    .gap(8.px())
    .padding(Length::px(16.0))
    .background_color(theme.page_bg())
}
