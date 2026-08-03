//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Table demo - virtualized table, testing with 10,000 rows.

use masonry::layout::{AsUnit, Length};
use xilem::masonry::peniko::Color;
use xilem::style::Style;
use xilem::WidgetView;
use xilem::view::{button, checkbox, flex_col, flex_row, label};

use xilem_extras::FlowDirection;
use xilem_extras::masonry::table::ColumnResizeMode;
use xilem_extras::xilem::table::{
    SortDirection, SortOrder, TableAction, TableStyle, row_cells, table_cell, table_styled,
};
use xilem_extras::xilem::theme::Theme;
use xilem_extras::xilem::traits::SelectionState;

use crate::app_model::AppModel;
use crate::mock_data::{Language, cyclist_display_name, cyclist_display_route};

/// English/Arabic title for each column key, used by the header-language
/// toggle. Keys stay stable (data lookups depend on them); only the title
/// swaps, exercising `detect_direction`'s live auto-detection from real
/// header content.
fn column_title(key: &str, lang: Language) -> &'static str {
    match (key, lang) {
        ("name", Language::Latin) => "Name",
        ("name", Language::Arabic) => "الاسم",
        ("route", Language::Latin) => "Route",
        ("route", Language::Arabic) => "المسار",
        ("distance_km", Language::Latin) => "Distance",
        ("distance_km", Language::Arabic) => "المسافة",
        ("joy_level", Language::Latin) => "Joy",
        ("joy_level", Language::Arabic) => "البهجة",
        _ => "?",
    }
}

pub fn table_demo(model: &mut AppModel) -> impl WidgetView<AppModel> + use<'_> {
    let theme = Theme::from_dark(model.dark_mode);
    let row_count = model.virtual_cyclists.len();
    let selection_count = model.virtual_table_selection.count();
    // Row content follows the header language directly — there's no
    // independent content toggle; an Arabic header means Arabic content.
    let content_language = model.virtual_table_header_language;

    // Compute sorted IDs for shift-selection to work
    use xilem_extras::xilem::traits::Keyed;
    let sorted_indices = model
        .virtual_table_sort
        .sort_indices(&model.virtual_cyclists);
    let sorted_ids: Vec<u64> = sorted_indices
        .iter()
        .map(|&idx| model.virtual_cyclists[idx].key())
        .collect();
    model.virtual_table_selection.set_items(sorted_ids);

    // Column titles follow the header-language toggle independently of the
    // row content language below. Updated in place on the model-owned Vec
    // (rather than building a new temporary one) so the borrow passed to
    // `table_styled` below lives as long as the returned view needs it to.
    let header_language = model.virtual_table_header_language;
    for col in model.virtual_table_columns.iter_mut() {
        col.title = column_title(&col.key, header_language).to_string();
    }

    // Build the virtual table using columns from model
    let table = table_styled(
        &model.virtual_cyclists,
        &model.virtual_table_columns,
        &model.virtual_table_column_widths,
        &model.virtual_table_selection,
        &model.virtual_table_sort,
        TableStyle::default()
            .resize_mode(model.virtual_table_resize_mode)
            .column_divider(model.virtual_table_column_dividers),
        // Row builder: (state, idx, is_selected, is_striped, column_widths, column_x_offsets, direction) -> RowView
        move |state: &mut AppModel,
              idx: usize,
              is_selected: bool,
              is_striped: bool,
              widths: &[f64],
              x_offsets: &[f64],
              _direction: FlowDirection| {
            let cyclist = &state.virtual_cyclists[idx];

            let row_bg = if is_selected {
                theme.active_bg()
            } else if is_striped {
                theme.section_bg()
            } else {
                Color::TRANSPARENT
            };

            // Use column widths from the table (supports resize).
            let w0 = widths.first().copied().unwrap_or(200.0);
            let w1 = widths.get(1).copied().unwrap_or(200.0);
            let w2 = widths.get(2).copied().unwrap_or(100.0);
            let w3 = widths.get(3).copied().unwrap_or(60.0);

            let txt = theme.text();
            let name = cyclist_display_name(cyclist, content_language);
            let route = cyclist_display_route(cyclist, content_language);

            // Cells are placed at their absolute x-offsets — the exact
            // mechanism the header uses for its own cells (see
            // `row_cells`'s doc comment) — in natural data order; no
            // reordering needed since absolute placement makes order
            // irrelevant for both LTR and RTL.
            let cells = vec![
                table_cell(label(name).text_size(13.0).color(txt).padding(Length::px(4.0)), w0),
                table_cell(
                    label(route).text_size(13.0).color(txt).padding(Length::px(4.0)),
                    w1,
                ),
                table_cell(
                    label(format!("{:.1} km", cyclist.distance_km))
                        .text_size(13.0)
                        .color(txt)
                        .padding(Length::px(4.0)),
                    w2,
                ),
                table_cell(
                    label(format!("{}/10", cyclist.joy_level))
                        .text_size(13.0)
                        .color(txt)
                        .padding(Length::px(4.0)),
                    w3,
                ),
            ];

            row_cells(cells, widths, x_offsets)
                .background_color(row_bg)
                .height(28.px())
        },
        // Action handler
        |state: &mut AppModel, action| {
            match action {
                TableAction::Select(id, mods) => {
                    state.virtual_table_selection.select(id, mods);
                }
                TableAction::Activate(id) => {
                    // Double-click: could open details
                    state.last_click_mods = format!("Activated cyclist #{}", id);
                }
                TableAction::Sort(column, direction) => {
                    state.virtual_table_sort = SortOrder::single(&*column, direction);
                }
                TableAction::ColumnResized(widths) => {
                    for (column_key, new_width) in widths {
                        state.virtual_table_column_widths.set(&column_key, new_width);
                    }
                }
            }
        },
    );

    // `table_styled` always wraps the table in its own internal `Portal`
    // now (needed so it can compensate the scroll position when RTL's
    // live mirror anchor shifts every column on growth) — `Overflow` mode
    // leaves both axes free to scroll; `FixedViewport` mode constrains
    // both, making it behave as a plain passthrough.
    flex_col((
        // Header
        label("Table Demo")
            .text_size(16.0)
            .weight(xilem::FontWeight::BOLD)
            .color(theme.text()),
        label(format!(
            "{} rows - only visible rows are rendered",
            row_count
        ))
        .text_size(12.0)
        .color(theme.text_secondary()),
        // The virtualized table
        table,
        // Info
        flex_col((flex_row((
            label(format!(
                "Sort: {} {}",
                model.virtual_table_sort.primary_column().unwrap_or("none"),
                match model.virtual_table_sort.direction() {
                    Some(SortDirection::Ascending) => "(asc)",
                    Some(SortDirection::Descending) => "(desc)",
                    None => "",
                }
            ))
            .text_size(12.0)
            .color(theme.text_secondary()),
            label(format!("Selected: {} cyclists", selection_count))
                .text_size(12.0)
                .color(theme.text_secondary()),
        ))
        .gap(16.px()),))
        .gap(4.px()),
        // Actions
        flex_row((
            button(label("Clear Selection"), |model: &mut AppModel| {
                model.virtual_table_selection.clear();
            }),
            button(
                label(match model.virtual_table_resize_mode {
                    ColumnResizeMode::Overflow => "Resize: Overflow",
                    ColumnResizeMode::FixedViewport => "Resize: Fixed Viewport",
                }),
                |model: &mut AppModel| {
                    model.virtual_table_resize_mode = match model.virtual_table_resize_mode {
                        ColumnResizeMode::Overflow => ColumnResizeMode::FixedViewport,
                        ColumnResizeMode::FixedViewport => ColumnResizeMode::Overflow,
                    };
                },
            ),
            button(
                label(match model.virtual_table_header_language {
                    Language::Latin => "Header: Latin",
                    Language::Arabic => "Header: Arabic",
                }),
                |model: &mut AppModel| {
                    model.virtual_table_header_language = match model.virtual_table_header_language
                    {
                        Language::Latin => Language::Arabic,
                        Language::Arabic => Language::Latin,
                    };
                },
            ),
            checkbox(
                "Divider",
                model.virtual_table_column_dividers,
                |model: &mut AppModel, checked: bool| {
                    model.virtual_table_column_dividers = checked;
                },
            ),
        ))
        .gap(8.px()),
    ))
    .gap(8.px())
    .padding(Length::px(16.0))
    .background_color(theme.page_bg())
}
