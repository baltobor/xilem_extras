//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Column width state management for resizable table columns.

use std::collections::HashMap;

/// Manages column widths for a resizable table.
///
/// Stores the current width of each column keyed by column name.
/// Provides methods for getting, setting, and resizing column widths
/// while respecting minimum width constraints.
#[derive(Debug, Clone, Default)]
pub struct ColumnWidths {
    widths: HashMap<String, f64>,
}

impl ColumnWidths {
    /// Creates a new empty ColumnWidths.
    pub fn new() -> Self {
        Self {
            widths: HashMap::new(),
        }
    }

    /// Creates ColumnWidths from a slice of (key, width) pairs.
    ///
    /// # Example
    ///
    /// ```
    /// use xilem_extras::ColumnWidths;
    ///
    /// let widths = ColumnWidths::from_columns(&[
    ///     ("name", 120.0),
    ///     ("route", 120.0),
    ///     ("distance", 80.0),
    /// ]);
    /// ```
    pub fn from_columns(columns: &[(&str, f64)]) -> Self {
        let widths = columns
            .iter()
            .map(|(key, width)| (key.to_string(), *width))
            .collect();
        Self { widths }
    }

    /// Gets the width for a column, returning a default if not set.
    ///
    /// Returns the stored width if present, otherwise returns 100.0 as a default.
    pub fn get(&self, key: &str) -> f64 {
        self.widths.get(key).copied().unwrap_or(100.0)
    }

    /// Gets the width for a column with a custom default.
    pub fn get_or(&self, key: &str, default: f64) -> f64 {
        self.widths.get(key).copied().unwrap_or(default)
    }

    /// Sets the width for a column.
    pub fn set(&mut self, key: &str, width: f64) {
        self.widths.insert(key.to_string(), width);
    }

    /// Resizes a column by a delta amount, respecting the minimum width.
    ///
    /// The resulting width will be clamped to at least `min_width`.
    ///
    /// # Arguments
    ///
    /// * `key` - The column key to resize
    /// * `delta` - The amount to change the width (positive = wider, negative = narrower)
    /// * `min_width` - The minimum allowed width
    ///
    /// # Returns
    ///
    /// The new width after resizing.
    pub fn resize(&mut self, key: &str, delta: f64, min_width: f64) -> f64 {
        let current = self.get(key);
        let new_width = (current + delta).max(min_width);
        self.set(key, new_width);
        new_width
    }

    /// Resizes a column to a specific width, respecting the minimum width.
    ///
    /// # Arguments
    ///
    /// * `key` - The column key to resize
    /// * `width` - The desired width
    /// * `min_width` - The minimum allowed width
    ///
    /// # Returns
    ///
    /// The new width after resizing (may be clamped to min_width).
    pub fn resize_to(&mut self, key: &str, width: f64, min_width: f64) -> f64 {
        let new_width = width.max(min_width);
        self.set(key, new_width);
        new_width
    }

    /// Returns an iterator over all column keys.
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.widths.keys()
    }

    /// Returns the number of columns with stored widths.
    pub fn len(&self) -> usize {
        self.widths.len()
    }

    /// Returns true if no column widths are stored.
    pub fn is_empty(&self) -> bool {
        self.widths.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let widths = ColumnWidths::new();
        assert!(widths.is_empty());
        assert_eq!(widths.len(), 0);
    }

    #[test]
    fn from_columns_stores_widths() {
        let widths = ColumnWidths::from_columns(&[
            ("name", 120.0),
            ("age", 80.0),
        ]);
        assert_eq!(widths.get("name"), 120.0);
        assert_eq!(widths.get("age"), 80.0);
        assert_eq!(widths.len(), 2);
    }

    #[test]
    fn get_returns_default_for_missing() {
        let widths = ColumnWidths::new();
        assert_eq!(widths.get("missing"), 100.0);
        assert_eq!(widths.get_or("missing", 50.0), 50.0);
    }

    #[test]
    fn set_stores_width() {
        let mut widths = ColumnWidths::new();
        widths.set("test", 150.0);
        assert_eq!(widths.get("test"), 150.0);
    }

    #[test]
    fn resize_applies_delta() {
        let mut widths = ColumnWidths::from_columns(&[("col", 100.0)]);

        let new_width = widths.resize("col", 20.0, 50.0);
        assert_eq!(new_width, 120.0);
        assert_eq!(widths.get("col"), 120.0);

        let new_width = widths.resize("col", -30.0, 50.0);
        assert_eq!(new_width, 90.0);
    }

    #[test]
    fn resize_respects_min_width() {
        let mut widths = ColumnWidths::from_columns(&[("col", 100.0)]);

        let new_width = widths.resize("col", -80.0, 50.0);
        assert_eq!(new_width, 50.0);
        assert_eq!(widths.get("col"), 50.0);
    }

    #[test]
    fn resize_to_sets_width() {
        let mut widths = ColumnWidths::new();

        let new_width = widths.resize_to("col", 150.0, 50.0);
        assert_eq!(new_width, 150.0);

        let new_width = widths.resize_to("col", 30.0, 50.0);
        assert_eq!(new_width, 50.0);
    }
}
