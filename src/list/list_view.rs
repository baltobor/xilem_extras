//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

use std::marker::PhantomData;

use crate::traits::{Identifiable, ListItem, SelectionModifiers, SelectionState};
use crate::tree::ExpansionState;

/// Actions that can occur on list items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListAction<Id> {
    /// Item selected (with modifiers for multi-selection)
    Select(Id, SelectionModifiers),
    /// Item activated (double-click or Enter)
    Activate(Id),
}

/// A list view for flat collections with selection support.
///
/// # Type Parameters
///
/// - `State`: Application state type
/// - `Action`: Action type returned by callbacks
/// - `I`: List item type
/// - `R`: Row view type
/// - `Sel`: Selection state type
/// - `F`: Row builder function type
/// - `H`: Handler function type
pub struct ListView<State, Action, I, R, Sel, F, H> {
    _phantom: PhantomData<(State, Action, I, R, Sel, F, H)>,
    row_builder: F,
    handler: H,
    row_height: f64,
}

impl<State, Action, I, R, Sel, F, H> ListView<State, Action, I, R, Sel, F, H> {
    /// Sets the row height.
    pub fn row_height(mut self, height: f64) -> Self {
        self.row_height = height;
        self
    }
}

/// Creates a list view for a flat collection.
///
/// # Arguments
///
/// * `items` - The collection of items to display
/// * `selection` - The selection state
/// * `row_builder` - Function that builds a view for each item: `(item, is_selected) -> View`
/// * `handler` - Function that handles list actions
///
/// # Example
///
/// ```ignore
/// list(
///     &model.contacts,
///     &model.selection,
///     |contact, is_selected| {
///         flex_row((
///             label(&contact.name),
///             label(&contact.email).color(text_secondary),
///         ))
///         .background_color(if is_selected { bg_selected } else { bg_normal })
///     },
///     |model, action| {
///         match action {
///             ListAction::Select(id, mods) => model.selection.select(id, mods),
///             ListAction::Activate(id) => model.open_contact(id),
///         }
///     },
/// )
/// ```
pub fn list<'a, State, Action, I, R, Sel, F, H>(
    items: &'a [I],
    selection: &'a Sel,
    row_builder: F,
    handler: H,
) -> ListView<State, Action, I, R, Sel, F, H>
where
    I: ListItem,
    Sel: SelectionState<I::Id>,
    F: Fn(&I, bool) -> R,
    H: Fn(&mut State, ListAction<I::Id>) -> Action,
{
    let _ = (items, selection); // Used in actual rendering

    ListView {
        _phantom: PhantomData,
        row_builder,
        handler,
        row_height: 24.0,
    }
}

/// A nested list view for hierarchical collections.
pub struct NestedListView<State, Action, I, R, C, F, H> {
    _phantom: PhantomData<(State, Action, I, R, C, F, H)>,
    row_builder: F,
    children_fn: C,
    handler: H,
    row_height: f64,
    indent: f64,
}

impl<State, Action, I, R, C, F, H> NestedListView<State, Action, I, R, C, F, H> {
    /// Sets the row height.
    pub fn row_height(mut self, height: f64) -> Self {
        self.row_height = height;
        self
    }

    /// Sets the indentation per level.
    pub fn indent(mut self, indent: f64) -> Self {
        self.indent = indent;
        self
    }
}

/// Creates a nested list view with children.
///
/// # Arguments
///
/// * `items` - The top-level items
/// * `children` - Function that returns children for each item
/// * `expansion` - Tracks which items are expanded
/// * `row_builder` - Function that builds a view for each item
/// * `handler` - Function that handles list actions
pub fn list_with_children<'a, State, Action, I, R, C, F, H>(
    items: &'a [I],
    children: C,
    expansion: &'a ExpansionState<I::Id>,
    row_builder: F,
    handler: H,
) -> NestedListView<State, Action, I, R, C, F, H>
where
    I: Identifiable,
    C: Fn(&I) -> &[I],
    F: Fn(&I, usize, bool) -> R,
    H: Fn(&mut State, ListAction<I::Id>) -> Action,
{
    let _ = (items, expansion); // Used in actual rendering

    NestedListView {
        _phantom: PhantomData,
        row_builder,
        children_fn: children,
        handler,
        row_height: 24.0,
        indent: 16.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selection::SingleSelection;

    #[derive(Debug, Clone)]
    struct TestItem {
        id: u64,
        name: String,
    }

    impl Identifiable for TestItem {
        type Id = u64;
        fn id(&self) -> Self::Id {
            self.id
        }
    }

    impl ListItem for TestItem {
        fn label(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn list_action_select() {
        let action = ListAction::Select(42u64, SelectionModifiers::NONE);
        if let ListAction::Select(id, mods) = action {
            assert_eq!(id, 42);
            assert_eq!(mods, SelectionModifiers::NONE);
        } else {
            panic!("Expected Select action");
        }
    }

    #[test]
    fn list_action_activate() {
        let action = ListAction::<u64>::Activate(42);
        if let ListAction::Activate(id) = action {
            assert_eq!(id, 42);
        } else {
            panic!("Expected Activate action");
        }
    }

    #[test]
    fn list_action_equality() {
        let a1 = ListAction::Select(1u64, SelectionModifiers::NONE);
        let a2 = ListAction::Select(1u64, SelectionModifiers::NONE);
        let a3 = ListAction::Select(2u64, SelectionModifiers::NONE);
        let a4 = ListAction::Select(1u64, SelectionModifiers::COMMAND);

        assert_eq!(a1, a2);
        assert_ne!(a1, a3);
        assert_ne!(a1, a4);
    }

    #[test]
    fn list_action_with_modifiers() {
        let action = ListAction::Select(1u64, SelectionModifiers::COMMAND);
        if let ListAction::Select(_, mods) = action {
            assert!(mods.command);
            assert!(!mods.shift);
        }
    }
}
