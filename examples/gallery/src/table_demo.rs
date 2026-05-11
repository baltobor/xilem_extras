//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Table widget demo - celebrating active mobility.

use std::sync::Arc;

use masonry::layout::AsUnit;
use xilem::masonry::peniko::Color;
use xilem::style::Style;
use xilem::view::{flex_col, flex_row, label, button, sized_box, CrossAxisAlignment};
use xilem::WidgetView;

use xilem_extras::{
    SelectionState, SelectionModifiers, SortOrder, SortDirection, Theme,
    row_button, row_button_with_modifiers, resizable_header,
};
use xilem_material_icons::{icons, FONT_FAMILY, ICON_SIZE_SM};

use crate::app_model::AppModel;
use crate::mock_data::Cyclist;

const ICON_COL_WIDTH: f64 = 24.0;

const BIKE_COLOR: Color = Color::from_rgb8(100, 180, 100);

fn sort_indicator(sort_order: &SortOrder, column: &str) -> &'static str {
    match sort_order.direction_for(column) {
        Some(SortDirection::Ascending) => icons::ARROW_UPWARD,
        Some(SortDirection::Descending) => icons::ARROW_DOWNWARD,
        None => "",
    }
}

fn column_header<'a>(
    title: &'a str,
    column_key: &'a str,
    width: f64,
    sort_order: &'a SortOrder,
    theme: Theme,
) -> impl WidgetView<AppModel> + use<'a> {
    let indicator = sort_indicator(sort_order, column_key);
    let col_key = column_key.to_string();

    let row = flex_row((
        label(title.to_string())
            .text_size(12.0)
            .weight(xilem::FontWeight::BOLD)
            .color(theme.text()),
        label(indicator.to_string())
            .font(FONT_FAMILY)
            .text_size(ICON_SIZE_SM)
            .color(theme.text()),
    ))
    .gap(4.px())
    .padding(8.0)
    .width((width as i32).px());

    row_button(row, move |model: &mut AppModel| {
        model.table_sort.toggle_column(&col_key, false);
    })
    .hover_bg(theme.hover_bg())
    .background_color(theme.nav_bg())
}

fn table_cell(value: String, width: f64, theme: Theme) -> impl WidgetView<AppModel> {
    label(value)
        .text_size(13.0)
        .color(theme.text())
        .padding(8.0)
        .width((width as i32).px())
}

fn cyclist_row<'a>(
    cyclist: &'a Cyclist,
    is_selected: bool,
    is_striped: bool,
    col_widths: &'a xilem_extras::ColumnWidths,
    theme: Theme,
) -> impl WidgetView<AppModel> + use<'a> {
    let id = cyclist.id;
    let row_bg = if is_selected {
        theme.active_bg()
    } else if is_striped {
        theme.section_bg()
    } else {
        Color::TRANSPARENT
    };

    let name_w = col_widths.get("name");
    let route_w = col_widths.get("route");
    let dist_w = col_widths.get("distance_km");
    let joy_w = col_widths.get("joy_level");
    let total_width = ICON_COL_WIDTH + name_w + route_w + dist_w + joy_w;

    let row = flex_row((
        label(icons::PEDAL_BIKE.to_string())
            .font(FONT_FAMILY)
            .text_size(ICON_SIZE_SM)
            .color(BIKE_COLOR)
            .width((ICON_COL_WIDTH as i32).px()),
        table_cell(cyclist.name.clone(), name_w, theme),
        table_cell(cyclist.route.clone(), route_w, theme),
        table_cell(format!("{:.1} km", cyclist.distance_km), dist_w, theme),
        table_cell(format!("{}/10", cyclist.joy_level), joy_w, theme),
    ))
    .gap(0.px());

    sized_box(
        row_button_with_modifiers(row, move |model: &mut AppModel, modifiers| {
            // Store modifier state for UI feedback
            model.last_click_mods = format!(
                "meta={}, ctrl={}, shift={}, alt={}",
                modifiers.meta(), modifiers.ctrl(), modifiers.shift(), modifiers.alt()
            );
            let sel_mods = SelectionModifiers::from_modifiers(modifiers);
            model.table_selection.select(id, sel_mods);
        })
        .hover_bg(theme.hover_bg())
        .background_color(row_bg)
    )
    .width((total_width as i32).px())
}

pub fn table_demo(model: &mut AppModel) -> impl WidgetView<AppModel> + use<'_> {
    let theme = Theme::from_dark(model.dark_mode);
    // Sort the data
    let sorted_cyclists = model.table_sort.sorted(&model.cyclists);

    // Update selection item order for shift+click range selection
    let sorted_ids: Vec<u64> = sorted_cyclists.iter().map(|c| c.id).collect();
    model.table_selection.set_items(sorted_ids);

    // Get column widths
    let name_w = model.table_column_widths.get("name");
    let route_w = model.table_column_widths.get("route");
    let dist_w = model.table_column_widths.get("distance_km");
    let joy_w = model.table_column_widths.get("joy_level");
    let total_width = ICON_COL_WIDTH + name_w + route_w + dist_w + joy_w;

    // Build rows
    let rows: Vec<_> = sorted_cyclists.iter().enumerate().map(|(idx, cyclist)| {
        let is_selected = model.table_selection.is_selected(&cyclist.id);
        let is_striped = idx % 2 == 1;
        cyclist_row(cyclist, is_selected, is_striped, &model.table_column_widths, theme).boxed()
    }).collect();

    // Build resizable header columns
    let header_columns = vec![
        column_header("Name", "name", name_w, &model.table_sort, theme).boxed(),
        column_header("Route", "route", route_w, &model.table_sort, theme).boxed(),
        column_header("Distance", "distance_km", dist_w, &model.table_sort, theme).boxed(),
        column_header("Joy", "joy_level", joy_w, &model.table_sort, theme).boxed(),
    ];

    let resizable_hdr = resizable_header(
        &[
            ("name", name_w),
            ("route", route_w),
            ("distance_km", dist_w),
            ("joy_level", joy_w),
        ],
        header_columns,
        |model: &mut AppModel, column_key: Arc<str>, new_width: f64| {
            model.table_column_widths.set(&column_key, new_width);
        },
    );

    flex_col((
        label("Cyclists")
            .text_size(16.0)
            .weight(xilem::FontWeight::BOLD)
            .color(theme.text()),
        label("Click column headers to sort, drag dividers to resize")
            .text_size(12.0)
            .color(theme.text_secondary()),
        label("Outdated. Please use the virtualized table.")
            .text_size(12.0)
            .color(theme.text_secondary()),            
            

        // Table (header + rows)
        flex_col((
            // Table header with icon column + resizable columns
            sized_box(
                flex_row((
                    label(icons::PEDAL_BIKE.to_string())
                        .font(FONT_FAMILY)
                        .text_size(ICON_SIZE_SM)
                        .color(BIKE_COLOR)
                        .width((ICON_COL_WIDTH as i32).px())
                        .background_color(theme.nav_bg()),
                    resizable_hdr,
                ))
                .gap(0.px())
                .background_color(theme.nav_bg())
            )
            .width((total_width as i32).px()),

            // Table rows
            flex_col(rows)
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .gap(0.px()),
        ))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .gap(0.px()),

        // Info and actions
        flex_col((
            flex_row((
                label(format!(
                    "Sort: {} {}",
                    model.table_sort.primary_column().unwrap_or("none"),
                    match model.table_sort.direction() {
                        Some(SortDirection::Ascending) => "(asc)",
                        Some(SortDirection::Descending) => "(desc)",
                        None => "",
                    }
                ))
                .text_size(12.0)
                .color(theme.text_secondary()),

                label(format!("Selected: {} cyclists", model.table_selection.count()))
                    .text_size(12.0)
                    .color(theme.text_secondary()),
            ))
            .gap(16.px()),

            label(format!("Last click modifiers: {}", model.last_click_mods))
                .text_size(12.0)
                .color(theme.text_secondary()),
        ))
        .gap(4.px()),

        flex_row((
            button(label("Clear Sort"), |model: &mut AppModel| {
                model.table_sort.clear();
            }),
            button(label("Clear Selection"), |model: &mut AppModel| {
                model.table_selection.clear();
            }),
        ))
        .gap(8.px()),
    ))
    .gap(8.px())
    .padding(16.0)
    .background_color(theme.page_bg())
}
