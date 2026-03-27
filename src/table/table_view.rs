//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Table view for displaying tabular data with columns, sorting, and selection.

use masonry::layout::AsUnit;
use xilem::masonry::core::PointerButton;
use xilem::masonry::vello::peniko::Color;
use xilem::style::Style;
use xilem::view::{flex_col, flex_row, label};
use xilem::{AnyWidgetView, WidgetView};

use crate::components::{row_button, row_button_with_press, RowButtonPress};
use crate::traits::{Identifiable, SelectionModifiers, SelectionState};
use super::{ColumnDef, SortDirection, SortOrder};

/// Actions that can occur on table rows or columns.
#[derive(Debug, Clone, PartialEq)]
pub enum TableAction<Id> {
    /// Column header clicked for sorting.
    Sort(String, SortDirection),
    /// Row selected with optional modifiers.
    Select(Id, SelectionModifiers),
    /// Row activated (double-click or Enter).
    Activate(Id),
    /// Column resized (column_key, new_width).
    ColumnResized(String, f64),
}

/// Style configuration for table.
#[derive(Debug, Clone)]
pub struct TableStyle {
    /// Background color on hover.
    pub hover_bg: Color,
    /// Background color for selected rows.
    pub selected_bg: Color,
    /// Background color for alternating rows (if striped).
    pub stripe_bg: Color,
    /// Header background color.
    pub header_bg: Color,
    /// Header text color.
    pub header_text_color: Color,
    /// Cell text color.
    pub text_color: Color,
    /// Row height in pixels.
    pub row_height: f64,
    /// Header height in pixels.
    pub header_height: f64,
    /// Whether to show alternating row backgrounds.
    pub striped: bool,
    /// Gap between columns.
    pub column_gap: f64,
}

impl Default for TableStyle {
    fn default() -> Self {
        Self {
            hover_bg: Color::from_rgba8(55, 53, 50, 255),
            selected_bg: Color::from_rgba8(65, 62, 58, 255),
            stripe_bg: Color::from_rgba8(45, 43, 40, 255),
            header_bg: Color::from_rgba8(50, 48, 45, 255),
            header_text_color: Color::from_rgba8(180, 178, 175, 255),
            text_color: Color::from_rgba8(220, 218, 214, 255),
            row_height: 28.0,
            header_height: 32.0,
            striped: false,
            column_gap: 8.0,
        }
    }
}

impl TableStyle {
    /// Creates a new TableStyle with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the hover background color.
    pub fn hover_bg(mut self, color: Color) -> Self {
        self.hover_bg = color;
        self
    }

    /// Sets the selected row background color.
    pub fn selected_bg(mut self, color: Color) -> Self {
        self.selected_bg = color;
        self
    }

    /// Sets the header background color.
    pub fn header_bg(mut self, color: Color) -> Self {
        self.header_bg = color;
        self
    }

    /// Sets the row height.
    pub fn row_height(mut self, height: f64) -> Self {
        self.row_height = height;
        self
    }

    /// Sets the header height.
    pub fn header_height(mut self, height: f64) -> Self {
        self.header_height = height;
        self
    }

    /// Enables alternating row backgrounds (zebra stripes).
    pub fn striped(mut self, striped: bool) -> Self {
        self.striped = striped;
        self
    }
}

/// Creates a table view for tabular data.
///
/// # Arguments
///
/// * `data` - The collection of rows (must implement `Identifiable`)
/// * `columns` - Column definitions
/// * `selection` - Selection state
/// * `sort_order` - Current sort state
/// * `cell_builder` - Function that builds a view for each cell: `(row, column_key) -> View`
/// * `handler` - Function that handles table actions
///
/// # Example
///
/// ```ignore
/// use xilem_extras::{table, column, TableAction, Alignment};
///
/// table(
///     &model.employees,
///     &[
///         column("name", "Name").flex(2.0).build(),
///         column("department", "Department").flex(1.5).build(),
///         column("salary", "Salary").fixed(100.0).align(Alignment::End).build(),
///     ],
///     &model.selection,
///     &model.sort_order,
///     |employee, column_key| {
///         match column_key {
///             "name" => label(employee.name.clone()),
///             "department" => label(employee.department.clone()),
///             "salary" => label(format!("${:.0}", employee.salary)),
///             _ => label(""),
///         }
///     },
///     |state, action| {
///         match action {
///             TableAction::Sort(column, direction) => {
///                 state.sort_order.toggle_sort(&column, false);
///             }
///             TableAction::Select(id, mods) => {
///                 state.selection.select(id, mods);
///             }
///             TableAction::Activate(id) => {
///                 state.edit_employee(&id);
///             }
///             TableAction::ColumnResized(_, _) => {}
///         }
///     },
/// )
/// ```
pub fn table<'a, State, R, C, Sel, F, H>(
    data: &'a [R],
    columns: &'a [ColumnDef],
    selection: &'a Sel,
    sort_order: &'a SortOrder,
    cell_builder: F,
    handler: H,
) -> impl WidgetView<State, ()> + use<'a, State, R, C, Sel, F, H>
where
    State: 'static,
    R: Identifiable + 'a,
    R::Id: Clone + Send + Sync + 'static,
    C: WidgetView<State, ()> + 'static,
    F: Fn(&R, &str) -> C + Clone + 'a,
    H: Fn(&mut State, TableAction<R::Id>) + Clone + Send + Sync + 'static,
    Sel: SelectionState<R::Id> + 'a,
{
    table_styled(data, columns, selection, sort_order, TableStyle::default(), cell_builder, handler)
}

/// Creates a table view with custom styling.
///
/// Same as [`table`] but accepts a [`TableStyle`] for customization.
pub fn table_styled<'a, State, R, C, Sel, F, H>(
    data: &'a [R],
    columns: &'a [ColumnDef],
    selection: &'a Sel,
    sort_order: &'a SortOrder,
    style: TableStyle,
    cell_builder: F,
    handler: H,
) -> impl WidgetView<State, ()> + use<'a, State, R, C, Sel, F, H>
where
    State: 'static,
    R: Identifiable + 'a,
    R::Id: Clone + Send + Sync + 'static,
    C: WidgetView<State, ()> + 'static,
    F: Fn(&R, &str) -> C + Clone + 'a,
    H: Fn(&mut State, TableAction<R::Id>) + Clone + Send + Sync + 'static,
    Sel: SelectionState<R::Id> + 'a,
{
    // Build header row
    let header_cells: Vec<Box<AnyWidgetView<State, ()>>> = columns
        .iter()
        .map(|col| {
            let col_key = col.key.clone();
            let handler = handler.clone();
            let sortable = col.sortable;

            // Check if this column is currently sorted
            let sort_indicator = sort_order.direction_for(&col.key).map(|dir| {
                match dir {
                    SortDirection::Ascending => " ▲",
                    SortDirection::Descending => " ▼",
                }
            }).unwrap_or("");

            let header_text = format!("{}{}", col.title, sort_indicator);
            let text_color = style.header_text_color;

            let header_label = label(header_text)
                .text_size(13.0)
                .color(text_color);

            if sortable {
                // Get current direction before closure (to avoid capturing sort_order reference)
                let current_dir = sort_order.direction_for(&col_key);
                let new_dir = current_dir
                    .map(|dir| dir.toggle())
                    .unwrap_or(SortDirection::Ascending);

                row_button(header_label, move |state: &mut State| {
                    handler(state, TableAction::Sort(col_key.clone(), new_dir));
                })
                .hover_bg(style.hover_bg)
                .boxed()
            } else {
                header_label.boxed()
            }
        })
        .collect();

    let header_row = flex_row(header_cells)
        .gap(style.column_gap.px())
        .padding(4.0)
        .background_color(style.header_bg)
        .height(style.header_height.px());

    // Build data rows
    let data_rows: Vec<Box<AnyWidgetView<State, ()>>> = data
        .iter()
        .enumerate()
        .map(|(row_idx, row)| {
            let is_selected = selection.is_selected(&row.id());
            let row_id = row.id();
            let handler = handler.clone();
            let hover_bg = style.hover_bg;

            // Determine row background
            let row_bg = if is_selected {
                style.selected_bg
            } else if style.striped && row_idx % 2 == 1 {
                style.stripe_bg
            } else {
                Color::TRANSPARENT
            };

            // Build cells for this row
            let cells: Vec<Box<AnyWidgetView<State, ()>>> = columns
                .iter()
                .map(|col| {
                    let cell_view = cell_builder(row, &col.key);
                    cell_view.boxed()
                })
                .collect();

            let row_content = flex_row(cells)
                .gap(style.column_gap.px())
                .padding(4.0)
                .background_color(row_bg)
                .height(style.row_height.px());

            row_button_with_press(row_content, move |state: &mut State, press: &RowButtonPress| {
                match press.button {
                    None | Some(PointerButton::Primary) => {
                        let sel_mods = SelectionModifiers::from_modifiers(press.modifiers);
                        let action = if press.click_count >= 2 {
                            TableAction::Activate(row_id.clone())
                        } else {
                            TableAction::Select(row_id.clone(), sel_mods)
                        };
                        handler(state, action);
                    }
                    _ => {}
                }
            })
            .hover_bg(hover_bg)
            .boxed()
        })
        .collect();

    // Combine header and data rows
    let mut all_rows: Vec<Box<AnyWidgetView<State, ()>>> = Vec::with_capacity(data_rows.len() + 1);
    all_rows.push(header_row.boxed());
    all_rows.extend(data_rows);

    flex_col(all_rows).gap(0.px())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_action_sort() {
        let action = TableAction::<u64>::Sort("name".into(), SortDirection::Ascending);
        if let TableAction::Sort(col, dir) = action {
            assert_eq!(col, "name");
            assert_eq!(dir, SortDirection::Ascending);
        } else {
            panic!("Expected Sort action");
        }
    }

    #[test]
    fn table_action_select() {
        let action = TableAction::Select(42u64, SelectionModifiers::COMMAND);
        if let TableAction::Select(id, mods) = action {
            assert_eq!(id, 42);
            assert!(mods.command);
        } else {
            panic!("Expected Select action");
        }
    }

    #[test]
    fn table_action_activate() {
        let action = TableAction::<u64>::Activate(42);
        if let TableAction::Activate(id) = action {
            assert_eq!(id, 42);
        } else {
            panic!("Expected Activate action");
        }
    }

    #[test]
    fn table_action_equality() {
        let a1 = TableAction::<u64>::Sort("name".into(), SortDirection::Ascending);
        let a2 = TableAction::<u64>::Sort("name".into(), SortDirection::Ascending);
        let a3 = TableAction::<u64>::Sort("name".into(), SortDirection::Descending);

        assert_eq!(a1, a2);
        assert_ne!(a1, a3);
    }

    #[test]
    fn table_action_column_resized() {
        let action = TableAction::<u64>::ColumnResized("name".into(), 150.0);
        if let TableAction::ColumnResized(col, width) = action {
            assert_eq!(col, "name");
            assert!((width - 150.0).abs() < f64::EPSILON);
        } else {
            panic!("Expected ColumnResized action");
        }
    }

    #[test]
    fn table_style_builder() {
        let style = TableStyle::new()
            .hover_bg(Color::from_rgb8(50, 50, 50))
            .row_height(32.0)
            .striped(true);

        assert_eq!(style.hover_bg, Color::from_rgb8(50, 50, 50));
        assert_eq!(style.row_height, 32.0);
        assert!(style.striped);
    }

    #[test]
    fn table_style_default() {
        let style = TableStyle::default();
        assert_eq!(style.row_height, 28.0);
        assert_eq!(style.header_height, 32.0);
        assert!(!style.striped);
    }
}
