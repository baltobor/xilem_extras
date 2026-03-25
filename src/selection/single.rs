//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

use std::hash::Hash;
use crate::traits::{SelectionModifiers, SelectionState};

/// Single-item selection state.
///
/// Only one item can be selected at a time. Clicking the same item
/// again toggles it off (deselects).
///
/// # Example
///
/// ```
/// use xilem_extras::{SingleSelection, SelectionState, SelectionModifiers};
///
/// let mut selection = SingleSelection::<u64>::new();
///
/// // Select item 1
/// selection.select(1, SelectionModifiers::NONE);
/// assert!(selection.is_selected(&1));
/// assert_eq!(selection.selected(), Some(&1));
///
/// // Select item 2 - replaces item 1
/// selection.select(2, SelectionModifiers::NONE);
/// assert!(!selection.is_selected(&1));
/// assert!(selection.is_selected(&2));
///
/// // Click item 2 again - toggles off
/// selection.select(2, SelectionModifiers::NONE);
/// assert!(!selection.is_selected(&2));
/// assert_eq!(selection.selected(), None);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SingleSelection<Id> {
    selected: Option<Id>,
}

impl<Id> Default for SingleSelection<Id> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Id> SingleSelection<Id> {
    /// Creates a new empty single selection.
    pub fn new() -> Self {
        Self { selected: None }
    }

    /// Creates a single selection with an initial selection.
    pub fn with_selected(id: Id) -> Self {
        Self { selected: Some(id) }
    }

    /// Returns the currently selected item, if any.
    pub fn selected(&self) -> Option<&Id> {
        self.selected.as_ref()
    }

    /// Sets the selection directly without toggle behavior.
    pub fn set(&mut self, id: Option<Id>) {
        self.selected = id;
    }
}

impl<Id: Clone + Eq + Hash> SelectionState<Id> for SingleSelection<Id> {
    fn is_selected(&self, id: &Id) -> bool {
        self.selected.as_ref() == Some(id)
    }

    fn select(&mut self, id: Id, _modifiers: SelectionModifiers) {
        // Toggle off if clicking the same item
        if self.selected.as_ref() == Some(&id) {
            self.selected = None;
        } else {
            self.selected = Some(id);
        }
    }

    fn clear(&mut self) {
        self.selected = None;
    }

    fn count(&self) -> usize {
        if self.selected.is_some() { 1 } else { 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let selection = SingleSelection::<u64>::new();
        assert!(selection.is_empty());
        assert_eq!(selection.selected(), None);
    }

    #[test]
    fn with_selected_creates_selection() {
        let selection = SingleSelection::with_selected(42u64);
        assert!(selection.is_selected(&42));
        assert_eq!(selection.selected(), Some(&42));
    }

    #[test]
    fn select_adds_item() {
        let mut selection = SingleSelection::<u64>::new();
        selection.select(1, SelectionModifiers::NONE);
        assert!(selection.is_selected(&1));
        assert_eq!(selection.count(), 1);
    }

    #[test]
    fn select_replaces_previous() {
        let mut selection = SingleSelection::<u64>::new();
        selection.select(1, SelectionModifiers::NONE);
        selection.select(2, SelectionModifiers::NONE);
        assert!(!selection.is_selected(&1));
        assert!(selection.is_selected(&2));
        assert_eq!(selection.count(), 1);
    }

    #[test]
    fn select_same_toggles_off() {
        let mut selection = SingleSelection::<u64>::new();
        selection.select(1, SelectionModifiers::NONE);
        selection.select(1, SelectionModifiers::NONE);
        assert!(!selection.is_selected(&1));
        assert!(selection.is_empty());
    }

    #[test]
    fn clear_removes_selection() {
        let mut selection = SingleSelection::with_selected(42u64);
        selection.clear();
        assert!(selection.is_empty());
    }

    #[test]
    fn modifiers_are_ignored() {
        let mut selection = SingleSelection::<u64>::new();
        selection.select(1, SelectionModifiers::COMMAND);
        selection.select(2, SelectionModifiers::SHIFT);
        // Still behaves as single selection
        assert!(!selection.is_selected(&1));
        assert!(selection.is_selected(&2));
    }

    #[test]
    fn set_bypasses_toggle() {
        let mut selection = SingleSelection::<u64>::new();
        selection.set(Some(1));
        assert!(selection.is_selected(&1));
        selection.set(Some(1)); // Same item, no toggle
        assert!(selection.is_selected(&1));
    }

    #[test]
    fn set_none_clears() {
        let mut selection = SingleSelection::with_selected(42u64);
        selection.set(None);
        assert!(selection.is_empty());
    }

    #[test]
    fn default_is_empty() {
        let selection: SingleSelection<u64> = Default::default();
        assert!(selection.is_empty());
    }

    #[test]
    fn is_empty_when_count_zero() {
        let selection = SingleSelection::<u64>::new();
        assert!(selection.is_empty());
        assert_eq!(selection.count(), 0);
    }

    #[test]
    fn not_empty_when_selected() {
        let selection = SingleSelection::with_selected(1u64);
        assert!(!selection.is_empty());
        assert_eq!(selection.count(), 1);
    }
}
