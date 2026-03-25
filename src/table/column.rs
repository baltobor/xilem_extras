//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

/// Column alignment options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Alignment {
    #[default]
    Start,
    Center,
    End,
}

/// Column width specification.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColumnWidth {
    /// Fixed width in pixels.
    Fixed(f64),
    /// Flexible width with relative weight.
    Flex(f64),
    /// Auto-size to content.
    Auto,
}

impl Default for ColumnWidth {
    fn default() -> Self {
        ColumnWidth::Flex(1.0)
    }
}

/// Definition of a table column.
#[derive(Debug, Clone)]
pub struct ColumnDef {
    /// Unique key for this column (used for cell lookup).
    pub key: String,
    /// Display title in the header.
    pub title: String,
    /// Column width specification.
    pub width: ColumnWidth,
    /// Whether this column is sortable.
    pub sortable: bool,
    /// Content alignment.
    pub alignment: Alignment,
    /// Minimum width (for resizable columns).
    pub min_width: Option<f64>,
    /// Maximum width (for resizable columns).
    pub max_width: Option<f64>,
}

impl ColumnDef {
    /// Creates a new column definition.
    pub fn new(key: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            width: ColumnWidth::default(),
            sortable: true,
            alignment: Alignment::default(),
            min_width: None,
            max_width: None,
        }
    }
}

/// Builder for creating column definitions.
pub struct ColumnBuilder {
    def: ColumnDef,
}

impl ColumnBuilder {
    /// Creates a new column builder.
    pub fn new(key: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            def: ColumnDef::new(key, title),
        }
    }

    /// Sets a fixed width in pixels.
    pub fn fixed(mut self, width: f64) -> Self {
        self.def.width = ColumnWidth::Fixed(width);
        self
    }

    /// Sets a flexible width with relative weight.
    pub fn flex(mut self, weight: f64) -> Self {
        self.def.width = ColumnWidth::Flex(weight);
        self
    }

    /// Sets the column to auto-size to content.
    pub fn auto(mut self) -> Self {
        self.def.width = ColumnWidth::Auto;
        self
    }

    /// Sets whether this column is sortable.
    pub fn sortable(mut self, sortable: bool) -> Self {
        self.def.sortable = sortable;
        self
    }

    /// Sets the content alignment.
    pub fn align(mut self, alignment: Alignment) -> Self {
        self.def.alignment = alignment;
        self
    }

    /// Sets the minimum width.
    pub fn min_width(mut self, width: f64) -> Self {
        self.def.min_width = Some(width);
        self
    }

    /// Sets the maximum width.
    pub fn max_width(mut self, width: f64) -> Self {
        self.def.max_width = Some(width);
        self
    }

    /// Builds the column definition.
    pub fn build(self) -> ColumnDef {
        self.def
    }
}

/// Creates a column builder.
///
/// # Example
///
/// ```
/// use xilem_extras::{column, Alignment};
///
/// let name_col = column("name", "Name")
///     .flex(2.0)
///     .sortable(true)
///     .build();
///
/// let age_col = column("age", "Age")
///     .fixed(80.0)
///     .align(Alignment::End)
///     .build();
/// ```
pub fn column(key: impl Into<String>, title: impl Into<String>) -> ColumnBuilder {
    ColumnBuilder::new(key, title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_default_values() {
        let col = column("test", "Test").build();
        assert_eq!(col.key, "test");
        assert_eq!(col.title, "Test");
        assert_eq!(col.width, ColumnWidth::Flex(1.0));
        assert!(col.sortable);
        assert_eq!(col.alignment, Alignment::Start);
        assert_eq!(col.min_width, None);
        assert_eq!(col.max_width, None);
    }

    #[test]
    fn column_fixed_width() {
        let col = column("test", "Test").fixed(100.0).build();
        assert_eq!(col.width, ColumnWidth::Fixed(100.0));
    }

    #[test]
    fn column_flex_width() {
        let col = column("test", "Test").flex(2.0).build();
        assert_eq!(col.width, ColumnWidth::Flex(2.0));
    }

    #[test]
    fn column_auto_width() {
        let col = column("test", "Test").auto().build();
        assert_eq!(col.width, ColumnWidth::Auto);
    }

    #[test]
    fn column_not_sortable() {
        let col = column("test", "Test").sortable(false).build();
        assert!(!col.sortable);
    }

    #[test]
    fn column_alignment() {
        let col = column("test", "Test").align(Alignment::Center).build();
        assert_eq!(col.alignment, Alignment::Center);
    }

    #[test]
    fn column_width_constraints() {
        let col = column("test", "Test")
            .min_width(50.0)
            .max_width(200.0)
            .build();
        assert_eq!(col.min_width, Some(50.0));
        assert_eq!(col.max_width, Some(200.0));
    }

    #[test]
    fn column_chaining() {
        let col = column("name", "Full Name")
            .flex(2.0)
            .align(Alignment::Start)
            .min_width(100.0)
            .max_width(300.0)
            .sortable(true)
            .build();

        assert_eq!(col.key, "name");
        assert_eq!(col.title, "Full Name");
        assert_eq!(col.width, ColumnWidth::Flex(2.0));
        assert_eq!(col.alignment, Alignment::Start);
        assert_eq!(col.min_width, Some(100.0));
        assert_eq!(col.max_width, Some(300.0));
        assert!(col.sortable);
    }

    #[test]
    fn alignment_default() {
        let align: Alignment = Default::default();
        assert_eq!(align, Alignment::Start);
    }
}
