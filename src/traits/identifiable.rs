//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

use std::hash::Hash;

/// Provides a stable identity for items in collections.
///
/// This trait enables efficient diffing and reconciliation of collection items
/// by providing a unique, stable identifier for each item.
///
/// # Example
///
/// ```
/// use xilem_extras::Identifiable;
///
/// struct User {
///     id: u64,
///     name: String,
/// }
///
/// impl Identifiable for User {
///     type Id = u64;
///     fn id(&self) -> Self::Id {
///         self.id
///     }
/// }
/// ```
pub trait Identifiable {
    /// The type of the unique identifier.
    ///
    /// Must be `Clone + Eq + Hash` for efficient storage in hash-based collections.
    type Id: Clone + Eq + Hash + Send + 'static;

    /// Returns the unique identifier for this item.
    fn id(&self) -> Self::Id;
}

impl Identifiable for String {
    type Id = String;

    fn id(&self) -> Self::Id {
        self.clone()
    }
}

impl<'a> Identifiable for &'a str {
    type Id = String;

    fn id(&self) -> Self::Id {
        (*self).to_string()
    }
}

impl Identifiable for u64 {
    type Id = u64;

    fn id(&self) -> Self::Id {
        *self
    }
}

impl Identifiable for u32 {
    type Id = u32;

    fn id(&self) -> Self::Id {
        *self
    }
}

impl Identifiable for usize {
    type Id = usize;

    fn id(&self) -> Self::Id {
        *self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_identifiable() {
        let s = String::from("test");
        assert_eq!(s.id(), "test");
    }

    #[test]
    fn str_identifiable() {
        let s = "test";
        assert_eq!(s.id(), "test".to_string());
    }

    #[test]
    fn u64_identifiable() {
        let n: u64 = 42;
        assert_eq!(n.id(), 42);
    }

    #[test]
    fn custom_struct_identifiable() {
        struct Item {
            id: u64,
        }

        impl Identifiable for Item {
            type Id = u64;
            fn id(&self) -> Self::Id {
                self.id
            }
        }

        let item = Item { id: 123 };
        assert_eq!(item.id(), 123);
    }
}
