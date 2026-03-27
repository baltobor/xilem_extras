//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

use std::cmp::Ordering;

use crate::traits::TableRow;

#[cfg(test)]
use crate::traits::CellValue;

/// Sort direction for a column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

impl SortDirection {
    /// Toggles between ascending and descending.
    pub fn toggle(self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }

    /// Returns the opposite direction.
    pub fn reversed(self) -> Self {
        self.toggle()
    }
}

/// A single sort descriptor (column + direction).
///
/// Sort descriptors can be chained for multi-column sorting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortDescriptor {
    /// The column key to sort by.
    pub column: String,
    /// The sort direction.
    pub direction: SortDirection,
}

impl SortDescriptor {
    /// Creates a new sort descriptor.
    pub fn new(column: impl Into<String>, direction: SortDirection) -> Self {
        Self {
            column: column.into(),
            direction,
        }
    }

    /// Creates an ascending sort descriptor.
    pub fn ascending(column: impl Into<String>) -> Self {
        Self::new(column, SortDirection::Ascending)
    }

    /// Creates a descending sort descriptor.
    pub fn descending(column: impl Into<String>) -> Self {
        Self::new(column, SortDirection::Descending)
    }

    /// Compares two table rows using this descriptor.
    pub fn compare<R: TableRow>(&self, a: &R, b: &R) -> Ordering {
        let a_val = a.cell(&self.column);
        let b_val = b.cell(&self.column);

        let ordering = a_val.sort_key().cmp(&b_val.sort_key());

        match self.direction {
            SortDirection::Ascending => ordering,
            SortDirection::Descending => ordering.reverse(),
        }
    }
}

/// Current sort state for a table.
///
/// Supports multi-column sorting with a list of sort descriptors.
/// The first descriptor is the primary sort, subsequent descriptors
/// are used as tiebreakers.
#[derive(Debug, Clone, Default)]
pub struct SortOrder {
    /// Ordered list of sort descriptors.
    descriptors: Vec<SortDescriptor>,
}

impl SortOrder {
    /// Creates an empty sort order (unsorted).
    pub fn new() -> Self {
        Self {
            descriptors: Vec::new(),
        }
    }

    /// Creates a sort order with a single descriptor.
    pub fn single(column: impl Into<String>, direction: SortDirection) -> Self {
        Self {
            descriptors: vec![SortDescriptor::new(column, direction)],
        }
    }

    /// Creates a sort order with multiple descriptors.
    pub fn multi(descriptors: impl IntoIterator<Item = SortDescriptor>) -> Self {
        Self {
            descriptors: descriptors.into_iter().collect(),
        }
    }

    /// Returns the primary (first) sort column, if any.
    pub fn primary_column(&self) -> Option<&str> {
        self.descriptors.first().map(|d| d.column.as_str())
    }

    /// Returns the sort direction for the primary column.
    pub fn direction(&self) -> Option<SortDirection> {
        self.descriptors.first().map(|d| d.direction)
    }

    /// Returns the sort direction for a specific column, if it's in the sort order.
    pub fn direction_for(&self, column: &str) -> Option<SortDirection> {
        self.descriptors
            .iter()
            .find(|d| d.column == column)
            .map(|d| d.direction)
    }

    /// Returns the position of a column in the sort order (0-indexed).
    pub fn position_of(&self, column: &str) -> Option<usize> {
        self.descriptors.iter().position(|d| d.column == column)
    }

    /// Returns the number of sort descriptors.
    pub fn len(&self) -> usize {
        self.descriptors.len()
    }

    /// Returns whether there are no sort descriptors.
    pub fn is_empty(&self) -> bool {
        self.descriptors.is_empty()
    }

    /// Clears all sort descriptors.
    pub fn clear(&mut self) {
        self.descriptors.clear();
    }

    /// Handles a click on a column header.
    ///
    /// - If the column is the primary sort: toggles direction
    /// - Otherwise: makes this column the primary sort (ascending)
    ///
    /// With shift modifier: adds/updates the column as a secondary sort.
    pub fn toggle_column(&mut self, column: &str, shift_held: bool) {
        if shift_held {
            // Multi-column sort mode
            if let Some(pos) = self.position_of(column) {
                // Column already in sort order - toggle its direction
                self.descriptors[pos].direction = self.descriptors[pos].direction.toggle();
            } else {
                // Add as new sort level
                self.descriptors.push(SortDescriptor::ascending(column));
            }
        } else {
            // Single-column sort mode
            if self.primary_column() == Some(column) {
                // Same column - toggle direction
                if let Some(desc) = self.descriptors.first_mut() {
                    desc.direction = desc.direction.toggle();
                }
            } else {
                // Different column - replace with ascending
                self.descriptors.clear();
                self.descriptors.push(SortDescriptor::ascending(column));
            }
        }
    }

    /// Compares two table rows using all descriptors in order.
    pub fn compare<R: TableRow>(&self, a: &R, b: &R) -> Ordering {
        for descriptor in &self.descriptors {
            let ordering = descriptor.compare(a, b);
            if ordering != Ordering::Equal {
                return ordering;
            }
        }
        Ordering::Equal
    }

    /// Sorts a slice of items in place using the current sort order.
    ///
    /// This method uses Rust's stable sort for predictable behavior.
    pub fn sort_slice<R: TableRow>(&self, items: &mut [R]) {
        if !self.is_empty() {
            items.sort_by(|a, b| self.compare(a, b));
        }
    }

    /// Returns a sorted copy of the items.
    ///
    /// Uses Rust's iterator and sorting facilities.
    pub fn sorted<R: TableRow + Clone>(&self, items: &[R]) -> Vec<R> {
        let mut result: Vec<R> = items.to_vec();
        self.sort_slice(&mut result);
        result
    }

    /// Returns indices that would sort the items.
    ///
    /// Useful for virtual lists where you don't want to clone data.
    pub fn sort_indices<R: TableRow>(&self, items: &[R]) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..items.len()).collect();
        if !self.is_empty() {
            indices.sort_by(|&a, &b| self.compare(&items[a], &items[b]));
        }
        indices
    }

    /// Creates an iterator adapter that yields items in sorted order.
    ///
    /// This is the idiomatic Rust approach using iterators.
    pub fn iter_sorted<'a, R: TableRow + Clone>(
        &'a self,
        items: &'a [R],
    ) -> impl Iterator<Item = &'a R> {
        let indices = self.sort_indices(items);
        indices.into_iter().map(move |i| &items[i])
    }

    /// Filters items by a predicate, then sorts.
    ///
    /// Combines filter and sort in a single pass.
    pub fn filter_sorted<'a, R: TableRow + Clone, P: Fn(&R) -> bool>(
        &self,
        items: &'a [R],
        predicate: P,
    ) -> Vec<&'a R> {
        let mut result: Vec<&R> = items.iter().filter(|item| predicate(*item)).collect();
        if !self.is_empty() {
            result.sort_by(|a, b| self.compare(*a, *b));
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Identifiable;

    #[derive(Debug, Clone)]
    struct TestRow {
        id: u64,
        name: String,
        age: i64,
        score: f64,
    }

    impl Identifiable for TestRow {
        type Id = u64;
        fn id(&self) -> Self::Id {
            self.id
        }
    }

    impl TableRow for TestRow {
        fn cell(&self, column: &str) -> CellValue {
            match column {
                "name" => CellValue::Text(self.name.clone()),
                "age" => CellValue::Integer(self.age),
                "score" => CellValue::Float(self.score, 2),
                _ => CellValue::Empty,
            }
        }
    }

    fn test_data() -> Vec<TestRow> {
        vec![
            TestRow { id: 1, name: "Alice".into(), age: 30, score: 85.5 },
            TestRow { id: 2, name: "Bob".into(), age: 25, score: 92.0 },
            TestRow { id: 3, name: "Charlie".into(), age: 35, score: 78.3 },
            TestRow { id: 4, name: "Alice".into(), age: 28, score: 88.0 },
        ]
    }

    #[test]
    fn sort_direction_toggle() {
        assert_eq!(SortDirection::Ascending.toggle(), SortDirection::Descending);
        assert_eq!(SortDirection::Descending.toggle(), SortDirection::Ascending);
    }

    #[test]
    fn sort_descriptor_ascending() {
        let desc = SortDescriptor::ascending("name");
        assert_eq!(desc.column, "name");
        assert_eq!(desc.direction, SortDirection::Ascending);
    }

    #[test]
    fn sort_descriptor_descending() {
        let desc = SortDescriptor::descending("age");
        assert_eq!(desc.column, "age");
        assert_eq!(desc.direction, SortDirection::Descending);
    }

    #[test]
    fn sort_order_empty() {
        let order = SortOrder::new();
        assert!(order.is_empty());
        assert_eq!(order.len(), 0);
        assert_eq!(order.primary_column(), None);
    }

    #[test]
    fn sort_order_single() {
        let order = SortOrder::single("name", SortDirection::Ascending);
        assert_eq!(order.len(), 1);
        assert_eq!(order.primary_column(), Some("name"));
        assert_eq!(order.direction(), Some(SortDirection::Ascending));
    }

    #[test]
    fn sort_order_toggle_same_column() {
        let mut order = SortOrder::single("name", SortDirection::Ascending);
        order.toggle_column("name", false);
        assert_eq!(order.direction(), Some(SortDirection::Descending));
    }

    #[test]
    fn sort_order_toggle_different_column() {
        let mut order = SortOrder::single("name", SortDirection::Ascending);
        order.toggle_column("age", false);
        assert_eq!(order.primary_column(), Some("age"));
        assert_eq!(order.direction(), Some(SortDirection::Ascending));
    }

    #[test]
    fn sort_order_multi_column_with_shift() {
        let mut order = SortOrder::single("name", SortDirection::Ascending);
        order.toggle_column("age", true);
        assert_eq!(order.len(), 2);
        assert_eq!(order.primary_column(), Some("name"));
        assert_eq!(order.direction_for("age"), Some(SortDirection::Ascending));
    }

    #[test]
    fn sort_order_toggle_existing_multi_column() {
        let mut order = SortOrder::multi([
            SortDescriptor::ascending("name"),
            SortDescriptor::ascending("age"),
        ]);
        order.toggle_column("age", true);
        assert_eq!(order.direction_for("age"), Some(SortDirection::Descending));
    }

    #[test]
    fn sort_by_name_ascending() {
        let data = test_data();
        let order = SortOrder::single("name", SortDirection::Ascending);
        let sorted = order.sorted(&data);

        let names: Vec<_> = sorted.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, vec!["Alice", "Alice", "Bob", "Charlie"]);
    }

    #[test]
    fn sort_by_name_descending() {
        let data = test_data();
        let order = SortOrder::single("name", SortDirection::Descending);
        let sorted = order.sorted(&data);

        let names: Vec<_> = sorted.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, vec!["Charlie", "Bob", "Alice", "Alice"]);
    }

    #[test]
    fn sort_by_age_ascending() {
        let data = test_data();
        let order = SortOrder::single("age", SortDirection::Ascending);
        let sorted = order.sorted(&data);

        let ages: Vec<_> = sorted.iter().map(|r| r.age).collect();
        assert_eq!(ages, vec![25, 28, 30, 35]);
    }

    #[test]
    fn sort_multi_column() {
        let data = test_data();
        let order = SortOrder::multi([
            SortDescriptor::ascending("name"),
            SortDescriptor::ascending("age"),
        ]);
        let sorted = order.sorted(&data);

        // Alice(28), Alice(30), Bob(25), Charlie(35)
        let results: Vec<_> = sorted.iter().map(|r| (&r.name, r.age)).collect();
        assert_eq!(results[0], (&"Alice".to_string(), 28));
        assert_eq!(results[1], (&"Alice".to_string(), 30));
        assert_eq!(results[2], (&"Bob".to_string(), 25));
        assert_eq!(results[3], (&"Charlie".to_string(), 35));
    }

    #[test]
    fn sort_indices() {
        let data = test_data();
        let order = SortOrder::single("age", SortDirection::Ascending);
        let indices = order.sort_indices(&data);

        // Bob(25) is at index 1, Alice(28) at 3, Alice(30) at 0, Charlie(35) at 2
        assert_eq!(indices, vec![1, 3, 0, 2]);
    }

    #[test]
    fn iter_sorted() {
        let data = test_data();
        let order = SortOrder::single("age", SortDirection::Ascending);
        let ages: Vec<_> = order.iter_sorted(&data).map(|r| r.age).collect();
        assert_eq!(ages, vec![25, 28, 30, 35]);
    }

    #[test]
    fn filter_sorted() {
        let data = test_data();
        let order = SortOrder::single("age", SortDirection::Ascending);
        let filtered = order.filter_sorted(&data, |r| r.age >= 28);

        let ages: Vec<_> = filtered.iter().map(|r| r.age).collect();
        assert_eq!(ages, vec![28, 30, 35]);
    }

    #[test]
    fn sort_empty_slice() {
        let data: Vec<TestRow> = vec![];
        let order = SortOrder::single("name", SortDirection::Ascending);
        let sorted = order.sorted(&data);
        assert!(sorted.is_empty());
    }

    #[test]
    fn sort_unsorted() {
        let data = test_data();
        let order = SortOrder::new();
        let sorted = order.sorted(&data);

        // Order should be preserved
        let ids: Vec<_> = sorted.iter().map(|r| r.id).collect();
        assert_eq!(ids, vec![1, 2, 3, 4]);
    }

    #[test]
    fn position_of_column() {
        let order = SortOrder::multi([
            SortDescriptor::ascending("name"),
            SortDescriptor::ascending("age"),
            SortDescriptor::descending("score"),
        ]);

        assert_eq!(order.position_of("name"), Some(0));
        assert_eq!(order.position_of("age"), Some(1));
        assert_eq!(order.position_of("score"), Some(2));
        assert_eq!(order.position_of("unknown"), None);
    }

    #[test]
    fn clear_sort_order() {
        let mut order = SortOrder::single("name", SortDirection::Ascending);
        order.clear();
        assert!(order.is_empty());
    }

    #[test]
    fn sort_by_float() {
        let data = test_data();
        let order = SortOrder::single("score", SortDirection::Descending);
        let sorted = order.sorted(&data);

        let scores: Vec<_> = sorted.iter().map(|r| r.score).collect();
        assert_eq!(scores, vec![92.0, 88.0, 85.5, 78.3]);
    }

    #[test]
    fn sort_slice_in_place() {
        let mut data = test_data();
        let order = SortOrder::single("age", SortDirection::Ascending);
        order.sort_slice(&mut data);

        let ages: Vec<_> = data.iter().map(|r| r.age).collect();
        assert_eq!(ages, vec![25, 28, 30, 35]);
    }
}
