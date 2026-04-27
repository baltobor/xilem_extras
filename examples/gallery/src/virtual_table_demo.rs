//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Virtual Table demo - testing with 10,000 rows.

use masonry::layout::AsUnit;
use xilem::masonry::peniko::Color;
use xilem::style::Style;
use xilem::view::{flex_col, flex_row, label, button};
use xilem::WidgetView;

use xilem_extras::{
    SelectionState, SortOrder, SortDirection,
    table, table_cell, TableAction,
};

use crate::app_model::AppModel;

const TEXT_COLOR: Color = Color::from_rgb8(220, 218, 214);
const TEXT_SECONDARY: Color = Color::from_rgb8(160, 156, 150);
const BG_SELECTED: Color = Color::from_rgb8(65, 62, 58);
const BG_STRIPE: Color = Color::from_rgb8(38, 36, 34);

pub fn virtual_table_demo(model: &mut AppModel) -> impl WidgetView<AppModel> + use<'_> {
    let row_count = model.virtual_cyclists.len();
    let selection_count = model.virtual_table_selection.count();

    // Compute sorted IDs for shift-selection to work
    use xilem_extras::Identifiable;
    let sorted_indices = model.virtual_table_sort.sort_indices(&model.virtual_cyclists);
    let sorted_ids: Vec<u64> = sorted_indices
        .iter()
        .map(|&idx| model.virtual_cyclists[idx].id())
        .collect();
    model.virtual_table_selection.set_items(sorted_ids);

    // Build the virtual table using columns from model
    let table = table(
        &model.virtual_cyclists,
        &model.virtual_table_columns,
        &model.virtual_table_column_widths,
        &model.virtual_table_selection,
        &model.virtual_table_sort,
        // Row builder: (state, idx, is_selected, is_striped, column_widths) -> RowView
        |state: &mut AppModel, idx: usize, is_selected: bool, is_striped: bool, widths: &[f64]| {
            let cyclist = &state.virtual_cyclists[idx];

            let row_bg = if is_selected {
                BG_SELECTED
            } else if is_striped {
                BG_STRIPE
            } else {
                Color::TRANSPARENT
            };

            // Use column widths from the table (supports resize)
            let w0 = widths.get(0).copied().unwrap_or(200.0);
            let w1 = widths.get(1).copied().unwrap_or(200.0);
            let w2 = widths.get(2).copied().unwrap_or(100.0);
            let w3 = widths.get(3).copied().unwrap_or(60.0);

            // Build row with clipped cells to prevent text overflow
            flex_row((
                table_cell(label(cyclist.name.clone()).text_size(13.0).color(TEXT_COLOR).padding(4.0), w0),
                table_cell(label(cyclist.route.clone()).text_size(13.0).color(TEXT_COLOR).padding(4.0), w1),
                table_cell(label(format!("{:.1} km", cyclist.distance_km)).text_size(13.0).color(TEXT_COLOR).padding(4.0), w2),
                table_cell(label(format!("{}/10", cyclist.joy_level)).text_size(13.0).color(TEXT_COLOR).padding(4.0), w3),
            ))
            .gap(2.px())
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
                    state.virtual_table_sort = SortOrder::single(&column, direction);
                }
                TableAction::ColumnResized(column_key, new_width) => {
                    state.virtual_table_column_widths.set(&column_key, new_width);
                }
            }
        },
    );

    flex_col((
        // Header
        label("Virtual Table Demo")
            .text_size(16.0)
            .weight(xilem::FontWeight::BOLD)
            .color(TEXT_COLOR),
        label(format!("{} rows - only visible rows are rendered", row_count))
            .text_size(12.0)
            .color(TEXT_SECONDARY),

        // The virtualized table
        table,

        // Info
        flex_col((
            flex_row((
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
                .color(TEXT_SECONDARY),

                label(format!("Selected: {} cyclists", selection_count))
                    .text_size(12.0)
                    .color(TEXT_SECONDARY),
            ))
            .gap(16.px()),
        ))
        .gap(4.px()),

        // Actions
        flex_row((
            button(label("Clear Selection"), |model: &mut AppModel| {
                model.virtual_table_selection.clear();
            }),
        ))
        .gap(8.px()),
    ))
    .gap(8.px())
    .padding(16.0)
}
