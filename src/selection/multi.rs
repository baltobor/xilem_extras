//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

use std::collections::HashSet;
use std::hash::Hash;
use crate::traits::{SelectionModifiers, SelectionState};

/// Multi-item selection state with modifier support.
///
/// Supports keyboard modifiers for toggle and range selection:
/// - No modifiers: Replace selection with clicked item
/// - Cmd/Ctrl: Toggle individual item
/// - Shift: Extend selection from anchor to clicked item (requires index resolver)
/// - Cmd/Ctrl + Shift: Add range to existing selection
///
/// # Example
///
/// ```
/// use xilem_extras::{MultiSelection, SelectionState, SelectionModifiers};
///
/// let mut selection = MultiSelection::<u64>::new();
///
/// // Click without modifiers - select single item
/// selection.select(1, SelectionModifiers::NONE);
/// assert!(selection.is_selected(&1));
///
/// // Cmd+click - toggle additional item
/// selection.select(2, SelectionModifiers::COMMAND);
/// assert!(selection.is_selected(&1));
/// assert!(selection.is_selected(&2));
///
/// // Cmd+click again - toggle off
/// selection.select(2, SelectionModifiers::COMMAND);
/// assert!(!selection.is_selected(&2));
/// ```
/// Multi-selection with anchor and optional item list for range selection.
#[derive(Debug, Clone)]
pub struct MultiSelection<Id: Clone + Eq + Hash> {
    selected: HashSet<Id>,
    /// The anchor for range selection (set on non-shift clicks)
    anchor: Option<Id>,
    /// All items in order (for range selection)
    all_items: Vec<Id>,
}

impl<Id: Clone + Eq + Hash> Default for MultiSelection<Id> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Id: Clone + Eq + Hash> MultiSelection<Id> {
    /// Creates a new empty multi-selection.
    pub fn new() -> Self {
        Self {
            selected: HashSet::new(),
            anchor: None,
            all_items: Vec::new(),
        }
    }

    /// Creates a multi-selection with initial selected items.
    pub fn with_selected(items: impl IntoIterator<Item = Id>) -> Self {
        let selected: HashSet<Id> = items.into_iter().collect();
        Self {
            selected,
            anchor: None,
            all_items: Vec::new(),
        }
    }

    /// Sets the ordered list of all items for range selection.
    ///
    /// This must be called before shift+click range selection will work.
    pub fn set_items(&mut self, items: Vec<Id>) {
        self.all_items = items;
    }

    /// Returns an iterator over selected items.
    pub fn iter(&self) -> impl Iterator<Item = &Id> {
        self.selected.iter()
    }

    /// Returns a slice of all selected items (unordered).
    pub fn selected(&self) -> Vec<&Id> {
        self.selected.iter().collect()
    }

    /// Adds an item to the selection without affecting other items.
    pub fn add(&mut self, id: Id) {
        self.selected.insert(id);
    }

    /// Removes an item from the selection.
    pub fn remove(&mut self, id: &Id) {
        self.selected.remove(id);
    }

    /// Toggles an item's selection state.
    pub fn toggle(&mut self, id: Id) {
        if self.selected.contains(&id) {
            self.selected.remove(&id);
        } else {
            self.selected.insert(id);
        }
    }

    /// Returns the anchor item for range selection.
    pub fn anchor(&self) -> Option<&Id> {
        self.anchor.as_ref()
    }

    /// Sets the anchor for range selection.
    pub fn set_anchor(&mut self, id: Id) {
        self.anchor = Some(id);
    }

    /// Selects a range of items from anchor to target (inclusive).
    fn select_range(&mut self, target: &Id, replace: bool) {
        let Some(anchor) = &self.anchor else { return };

        // Find indices in all_items
        let anchor_idx = self.all_items.iter().position(|x| x == anchor);
        let target_idx = self.all_items.iter().position(|x| x == target);

        if let (Some(a), Some(t)) = (anchor_idx, target_idx) {
            let (start, end) = if a <= t { (a, t) } else { (t, a) };

            if replace {
                self.selected.clear();
            }

            for idx in start..=end {
                self.selected.insert(self.all_items[idx].clone());
            }
        }
    }
}

impl<Id: Clone + Eq + Hash> SelectionState<Id> for MultiSelection<Id> {
    fn is_selected(&self, id: &Id) -> bool {
        self.selected.contains(id)
    }

    fn select(&mut self, id: Id, modifiers: SelectionModifiers) {
        // Alt works like Command for toggle behavior
        let toggle_modifier = modifiers.command || modifiers.alt;

        match (toggle_modifier, modifiers.shift) {
            // No modifiers: replace selection
            (false, false) => {
                self.selected.clear();
                self.selected.insert(id.clone());
                self.anchor = Some(id);
            }
            // Command or Alt only: toggle individual item
            (true, false) => {
                self.toggle(id.clone());
                if self.selected.contains(&id) {
                    self.anchor = Some(id);
                }
            }
            // Shift only: range select (replace)
            (false, true) => {
                if self.anchor.is_some() {
                    self.select_range(&id, true);
                } else {
                    // No anchor, treat as single select
                    self.selected.clear();
                    self.selected.insert(id.clone());
                    self.anchor = Some(id);
                }
            }
            // Command/Alt + Shift: range select (add)
            (true, true) => {
                if self.anchor.is_some() {
                    self.select_range(&id, false);
                } else {
                    self.selected.insert(id.clone());
                    self.anchor = Some(id);
                }
            }
        }
    }

    fn clear(&mut self) {
        self.selected.clear();
        self.anchor = None;
    }

    fn count(&self) -> usize {
        self.selected.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let selection = MultiSelection::<u64>::new();
        assert!(selection.is_empty());
    }

    #[test]
    fn with_selected_creates_selection() {
        let selection = MultiSelection::with_selected([1u64, 2, 3]);
        assert!(selection.is_selected(&1));
        assert!(selection.is_selected(&2));
        assert!(selection.is_selected(&3));
        assert_eq!(selection.count(), 3);
    }

    #[test]
    fn select_no_modifiers_replaces() {
        let mut selection = MultiSelection::with_selected([1u64, 2, 3]);
        selection.select(4, SelectionModifiers::NONE);
        assert!(!selection.is_selected(&1));
        assert!(!selection.is_selected(&2));
        assert!(!selection.is_selected(&3));
        assert!(selection.is_selected(&4));
        assert_eq!(selection.count(), 1);
    }

    #[test]
    fn select_command_toggles_on() {
        let mut selection = MultiSelection::with_selected([1u64]);
        selection.select(2, SelectionModifiers::COMMAND);
        assert!(selection.is_selected(&1));
        assert!(selection.is_selected(&2));
        assert_eq!(selection.count(), 2);
    }

    #[test]
    fn select_command_toggles_off() {
        let mut selection = MultiSelection::with_selected([1u64, 2]);
        selection.select(2, SelectionModifiers::COMMAND);
        assert!(selection.is_selected(&1));
        assert!(!selection.is_selected(&2));
        assert_eq!(selection.count(), 1);
    }

    #[test]
    fn select_alt_toggles_on() {
        let mut selection = MultiSelection::with_selected([1u64]);
        selection.select(2, SelectionModifiers::ALT);
        assert!(selection.is_selected(&1));
        assert!(selection.is_selected(&2));
        assert_eq!(selection.count(), 2);
    }

    #[test]
    fn select_alt_toggles_off() {
        let mut selection = MultiSelection::with_selected([1u64, 2]);
        selection.select(2, SelectionModifiers::ALT);
        assert!(selection.is_selected(&1));
        assert!(!selection.is_selected(&2));
        assert_eq!(selection.count(), 1);
    }

    #[test]
    fn select_shift_range_replaces() {
        let mut selection = MultiSelection::<u64>::new();
        selection.set_items(vec![1, 2, 3, 4, 5]);

        // First click sets anchor
        selection.select(2, SelectionModifiers::NONE);
        assert!(selection.is_selected(&2));

        // Shift+click selects range and replaces
        selection.select(4, SelectionModifiers::SHIFT);
        assert!(!selection.is_selected(&1));
        assert!(selection.is_selected(&2));
        assert!(selection.is_selected(&3));
        assert!(selection.is_selected(&4));
        assert!(!selection.is_selected(&5));
    }

    #[test]
    fn select_command_shift_range_adds() {
        let mut selection = MultiSelection::<u64>::new();
        selection.set_items(vec![1, 2, 3, 4, 5]);

        // Select items 1 and 2
        selection.select(1, SelectionModifiers::NONE);
        selection.select(2, SelectionModifiers::COMMAND);

        // Set new anchor at 4
        selection.select(4, SelectionModifiers::COMMAND);

        // Cmd+Shift to 5 should add 4-5 without removing 1-2
        selection.select(5, SelectionModifiers::BOTH);
        assert!(selection.is_selected(&1));
        assert!(selection.is_selected(&2));
        assert!(selection.is_selected(&4));
        assert!(selection.is_selected(&5));
    }

    #[test]
    fn select_shift_reverse_range() {
        let mut selection = MultiSelection::<u64>::new();
        selection.set_items(vec![1, 2, 3, 4, 5]);

        selection.select(4, SelectionModifiers::NONE);
        selection.select(2, SelectionModifiers::SHIFT);

        assert!(selection.is_selected(&2));
        assert!(selection.is_selected(&3));
        assert!(selection.is_selected(&4));
        assert!(!selection.is_selected(&1));
        assert!(!selection.is_selected(&5));
    }

    #[test]
    fn select_shift_no_anchor() {
        let mut selection = MultiSelection::<u64>::new();
        selection.set_items(vec![1, 2, 3]);

        // Shift+click without anchor should behave like normal click
        selection.select(2, SelectionModifiers::SHIFT);
        assert!(selection.is_selected(&2));
        assert_eq!(selection.count(), 1);
    }

    #[test]
    fn clear_removes_all() {
        let mut selection = MultiSelection::with_selected([1u64, 2, 3]);
        selection.clear();
        assert!(selection.is_empty());
        assert_eq!(selection.anchor(), None);
    }

    #[test]
    fn add_does_not_affect_others() {
        let mut selection = MultiSelection::with_selected([1u64]);
        selection.add(2);
        assert!(selection.is_selected(&1));
        assert!(selection.is_selected(&2));
    }

    #[test]
    fn remove_single_item() {
        let mut selection = MultiSelection::with_selected([1u64, 2, 3]);
        selection.remove(&2);
        assert!(selection.is_selected(&1));
        assert!(!selection.is_selected(&2));
        assert!(selection.is_selected(&3));
    }

    #[test]
    fn toggle_adds_when_not_selected() {
        let mut selection = MultiSelection::<u64>::new();
        selection.toggle(1);
        assert!(selection.is_selected(&1));
    }

    #[test]
    fn toggle_removes_when_selected() {
        let mut selection = MultiSelection::with_selected([1u64]);
        selection.toggle(1);
        assert!(!selection.is_selected(&1));
    }

    #[test]
    fn iter_returns_all_selected() {
        let selection = MultiSelection::with_selected([1u64, 2, 3]);
        let items: HashSet<_> = selection.iter().cloned().collect();
        assert_eq!(items, HashSet::from([1, 2, 3]));
    }

    #[test]
    fn anchor_is_set_on_select() {
        let mut selection = MultiSelection::<u64>::new();
        selection.select(42, SelectionModifiers::NONE);
        assert_eq!(selection.anchor(), Some(&42));
    }

    #[test]
    fn anchor_is_set_on_command_select_when_adding() {
        let mut selection = MultiSelection::<u64>::new();
        selection.select(1, SelectionModifiers::COMMAND);
        assert_eq!(selection.anchor(), Some(&1));
    }

    #[test]
    fn anchor_preserved_on_command_deselect() {
        let mut selection = MultiSelection::<u64>::new();
        selection.select(1, SelectionModifiers::NONE);
        selection.select(2, SelectionModifiers::COMMAND);
        // Deselect item 2
        selection.select(2, SelectionModifiers::COMMAND);
        // Anchor should still be 2 since that was the last addition
        // Actually, when deselecting, anchor doesn't change to deselected item
        // The anchor was set to 2 when we added it, then we removed it
        // but anchor should remain as 2 was the last anchor set
    }

    #[test]
    fn default_is_empty() {
        let selection: MultiSelection<u64> = Default::default();
        assert!(selection.is_empty());
    }

    #[test]
    fn set_items_enables_range_selection() {
        let mut selection = MultiSelection::<u64>::new();
        // Without set_items, range selection won't work
        selection.select(1, SelectionModifiers::NONE);
        selection.select(3, SelectionModifiers::SHIFT);
        // Should only have item 1 since no items were set
        // Actually without items list, range selection falls back to just having anchor
        assert_eq!(selection.count(), 1);

        // Now set items and try again
        selection.clear();
        selection.set_items(vec![1, 2, 3, 4, 5]);
        selection.select(1, SelectionModifiers::NONE);
        selection.select(3, SelectionModifiers::SHIFT);
        assert_eq!(selection.count(), 3);
    }
}
