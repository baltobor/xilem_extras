//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

use super::Identifiable;

/// A value that can be displayed in a table cell.
///
/// This enum covers common data types with display-ready formatting.
#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    /// String text
    Text(String),
    /// Integer number
    Integer(i64),
    /// Floating point number with precision
    Float(f64, usize),
    /// Boolean displayed as checkmark or empty
    Boolean(bool),
    /// Empty/null cell
    Empty,
}

impl CellValue {
    /// Returns the string representation of this value.
    pub fn as_str(&self) -> String {
        match self {
            CellValue::Text(s) => s.clone(),
            CellValue::Integer(n) => n.to_string(),
            CellValue::Float(n, precision) => format!("{:.prec$}", n, prec = *precision),
            CellValue::Boolean(b) => if *b { "Yes" } else { "No" }.to_string(),
            CellValue::Empty => String::new(),
        }
    }

    /// Returns a comparable value for sorting.
    ///
    /// Returns a tuple of (type_order, comparable_value) where type_order
    /// ensures consistent ordering across types.
    pub fn sort_key(&self) -> (u8, i64, String) {
        match self {
            CellValue::Empty => (0, 0, String::new()),
            CellValue::Boolean(b) => (1, if *b { 1 } else { 0 }, String::new()),
            CellValue::Integer(n) => (2, *n, String::new()),
            CellValue::Float(n, _) => (2, (*n * 1_000_000.0) as i64, String::new()),
            CellValue::Text(s) => (3, 0, s.to_lowercase()),
        }
    }
}

impl From<String> for CellValue {
    fn from(s: String) -> Self {
        CellValue::Text(s)
    }
}

impl From<&str> for CellValue {
    fn from(s: &str) -> Self {
        CellValue::Text(s.to_string())
    }
}

impl From<i64> for CellValue {
    fn from(n: i64) -> Self {
        CellValue::Integer(n)
    }
}

impl From<i32> for CellValue {
    fn from(n: i32) -> Self {
        CellValue::Integer(n as i64)
    }
}

impl From<f64> for CellValue {
    fn from(n: f64) -> Self {
        CellValue::Float(n, 2)
    }
}

impl From<bool> for CellValue {
    fn from(b: bool) -> Self {
        CellValue::Boolean(b)
    }
}

impl<T> From<Option<T>> for CellValue
where
    T: Into<CellValue>,
{
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => v.into(),
            None => CellValue::Empty,
        }
    }
}

/// A row in a table with column-keyed cells.
///
/// # Example
///
/// ```
/// use xilem_extras::{Identifiable, TableRow, CellValue};
///
/// struct Employee {
///     id: u64,
///     name: String,
///     department: String,
///     salary: f64,
/// }
///
/// impl Identifiable for Employee {
///     type Id = u64;
///     fn id(&self) -> Self::Id {
///         self.id
///     }
/// }
///
/// impl TableRow for Employee {
///     fn cell(&self, column_key: &str) -> CellValue {
///         match column_key {
///             "name" => CellValue::Text(self.name.clone()),
///             "department" => CellValue::Text(self.department.clone()),
///             "salary" => CellValue::Float(self.salary, 2),
///             _ => CellValue::Empty,
///         }
///     }
/// }
/// ```
pub trait TableRow: Identifiable {
    /// Returns the cell value for the given column key.
    fn cell(&self, column_key: &str) -> CellValue;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_value_text() {
        let v = CellValue::Text("hello".to_string());
        assert_eq!(v.as_str(), "hello");
    }

    #[test]
    fn test_cell_value_integer() {
        let v = CellValue::Integer(42);
        assert_eq!(v.as_str(), "42");
    }

    #[test]
    fn test_cell_value_float() {
        let v = CellValue::Float(3.14159, 2);
        assert_eq!(v.as_str(), "3.14");
    }

    #[test]
    fn test_cell_value_float_zero_precision() {
        let v = CellValue::Float(3.9, 0);
        assert_eq!(v.as_str(), "4");
    }

    #[test]
    fn test_cell_value_boolean() {
        assert_eq!(CellValue::Boolean(true).as_str(), "Yes");
        assert_eq!(CellValue::Boolean(false).as_str(), "No");
    }

    #[test]
    fn test_cell_value_empty() {
        assert_eq!(CellValue::Empty.as_str(), "");
    }

    #[test]
    fn test_from_string() {
        let v: CellValue = "test".into();
        assert_eq!(v, CellValue::Text("test".to_string()));
    }

    #[test]
    fn test_from_i32() {
        let v: CellValue = 42i32.into();
        assert_eq!(v, CellValue::Integer(42));
    }

    #[test]
    fn test_from_option_some() {
        let v: CellValue = Some("test").into();
        assert_eq!(v, CellValue::Text("test".to_string()));
    }

    #[test]
    fn test_from_option_none() {
        let v: CellValue = Option::<String>::None.into();
        assert_eq!(v, CellValue::Empty);
    }

    #[test]
    fn test_sort_key_ordering() {
        let empty = CellValue::Empty;
        let bool_false = CellValue::Boolean(false);
        let bool_true = CellValue::Boolean(true);
        let int = CellValue::Integer(10);
        let text = CellValue::Text("abc".into());

        assert!(empty.sort_key() < bool_false.sort_key());
        assert!(bool_false.sort_key() < bool_true.sort_key());
        assert!(bool_true.sort_key() < int.sort_key());
        assert!(int.sort_key() < text.sort_key());
    }

    #[test]
    fn test_sort_key_integers() {
        let a = CellValue::Integer(10);
        let b = CellValue::Integer(20);
        assert!(a.sort_key() < b.sort_key());
    }

    #[test]
    fn test_sort_key_text_case_insensitive() {
        let a = CellValue::Text("ABC".into());
        let b = CellValue::Text("abc".into());
        assert_eq!(a.sort_key().2, b.sort_key().2);
    }
}
