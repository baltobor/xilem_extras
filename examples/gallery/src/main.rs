//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Gallery example demonstrating xilem_extras widgets.

mod app_model;
mod mock_data;
mod tree_demo;
mod list_demo;
mod table_demo;
mod tabs_demo;
mod menu_demo;

use masonry::layout::AsUnit;
use xilem::masonry::vello::peniko::Color;
use xilem::style::Style;
use xilem::view::{CrossAxisAlignment, split, flex_col, label};
use xilem::{EventLoop, WidgetView, WindowOptions, Xilem};

use xilem_extras::row_button;
use app_model::{AppModel, Page};

// Material Symbols font
use xilem_material_icons::FONT_DATA;

const TEXT_COLOR: Color = Color::from_rgb8(220, 218, 214);
const BG_NAV: Color = Color::from_rgb8(45, 43, 40);
const BG_HOVER: Color = Color::from_rgb8(55, 52, 48);
const BG_ACTIVE: Color = Color::from_rgb8(65, 62, 58);

fn nav_button(text: &str, page: Page, current: Page) -> impl WidgetView<AppModel> + use<'_> {
    let is_active = page == current;
    let bg = if is_active { BG_ACTIVE } else { BG_NAV };

    row_button(
        label(text.to_string())
            .text_size(13.0)
            .color(TEXT_COLOR)
            .padding(8.0),
        move |model: &mut AppModel| {
            model.page = page;
        },
    )
    .hover_bg(BG_HOVER)
    .background_color(bg)
}

fn app_logic(model: &mut AppModel) -> impl WidgetView<AppModel> + use<> {
    let current_page = model.page;

    split(
        // Navigation sidebar
        flex_col((
            label("xilem_extras")
                .text_size(14.0)
                .weight(xilem::FontWeight::BOLD)
                .color(TEXT_COLOR),
            label("Gallery")
                .text_size(12.0)
                .color(Color::from_rgb8(160, 156, 150)),
            nav_button("Tree", Page::Tree, current_page),
            nav_button("List", Page::List, current_page),
            nav_button("Table", Page::Table, current_page),
            nav_button("Tabs", Page::Tabs, current_page),
            nav_button("Menu", Page::Menu, current_page),
        ))
        .cross_axis_alignment(CrossAxisAlignment::Stretch)
        .gap(4.px())
        .padding(12.0)
        .background_color(BG_NAV),

        // Demo content
        match model.page {
            Page::Tree => tree_demo::tree_demo(model).boxed(),
            Page::List => list_demo::list_demo(model).boxed(),
            Page::Table => table_demo::table_demo(model).boxed(),
            Page::Tabs => tabs_demo::tabs_demo(model).boxed(),
            Page::Menu => menu_demo::menu_demo(model).boxed(),
        },
    )
    .split_point_from_start(160.px())
    .min_lengths(120.px(), 200.px())
    .bar_thickness(1.px())
    .solid_bar(true)
}

fn main() {
    let app = Xilem::new_simple(
        AppModel::new(),
        app_logic,
        WindowOptions::new("xilem_extras Gallery")
            .with_initial_inner_size(xilem::winit::dpi::LogicalSize::new(900.0, 600.0)),
    )
    .with_font(FONT_DATA.to_vec());
    app.run_in(EventLoop::with_user_event()).unwrap();
}
