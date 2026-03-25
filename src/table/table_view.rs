//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

use std::marker::PhantomData;

use crate::traits::{SelectionModifiers, TableRow};
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

/// A table view for displaying tabular data.
///
/// # Type Parameters
///
/// - `State`: Application state type
/// - `Action`: Action type returned by callbacks
/// - `R`: Table row type
/// - `C`: Cell view type
/// - `F`: Cell builder function type
/// - `H`: Handler function type
pub struct TableView<State, Action, R, C, F, H> {
    _phantom: PhantomData<(State, Action, R, C)>,
    cell_builder: F,
    handler: H,
    row_height: f64,
    header_height: f64,
    striped: bool,
}

impl<State, Action, R, C, F, H> TableView<State, Action, R, C, F, H> {
    /// Sets the row height (default: 24.0).
    pub fn row_height(mut self, height: f64) -> Self {
        self.row_height = height;
        self
    }

    /// Sets the header row height (default: 28.0).
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
/// * `data` - The collection of rows
/// * `columns` - Column definitions
/// * `sort_order` - Current sort state
/// * `cell_builder` - Function that builds a view for each cell: `(row, column_key, is_selected) -> View`
/// * `handler` - Function that handles table actions
///
/// # Example
///
/// ```ignore
/// table(
///     &model.employees,
///     &[
///         column("name", "Name").flex(2.0).build(),
///         column("department", "Department").flex(1.5).build(),
///         column("salary", "Salary").fixed(100.0).align(Alignment::End).build(),
///     ],
///     &model.sort_order,
///     |employee, column_key, is_selected| {
///         let text = match column_key {
///             "name" => employee.name.clone(),
///             "department" => employee.department.clone(),
///             "salary" => format!("${:.0}", employee.salary),
///             _ => String::new(),
///         };
///         label(text)
///     },
///     |model, action| {
///         match action {
///             TableAction::Sort(column, direction) => {
///                 model.sort_order = SortOrder::single(column, direction);
///             }
///             TableAction::Select(id, mods) => {
///                 model.selection.select(id, mods);
///             }
///             TableAction::Activate(id) => {
///                 model.edit_employee(id);
///             }
///         }
///     },
/// )
/// ```
pub fn table<'a, State, Action, R, C, F, H>(
    data: &'a [R],
    columns: &'a [ColumnDef],
    sort_order: &'a SortOrder,
    cell_builder: F,
    handler: H,
) -> TableView<State, Action, R, C, F, H>
where
    R: TableRow,
    F: Fn(&R, &str, bool) -> C,
    H: Fn(&mut State, TableAction<R::Id>) -> Action,
{
    let _ = (data, columns, sort_order); // Used in actual rendering

    TableView {
        _phantom: PhantomData,
        cell_builder,
        handler,
        row_height: 24.0,
        header_height: 28.0,
        striped: false,
    }
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
}
