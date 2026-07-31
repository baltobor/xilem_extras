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
/// by providing a unique, stable key for each item.
///
/// SwiftUI has the same concept under the name `Identifiable`. We call it
/// `Keyed` instead: `Key` is the term Rust/web diffing code already uses for
/// "what a reconciliation algorithm keys list items on" (see e.g. React's
/// `key` prop, or keyed diffing in general), so it reads as native Rust
/// vocabulary rather than a straight port of the Swift name.
///
/// # Example
///
/// ```
/// use xilem_extras::Keyed;
///
/// struct User {
///     id: u64,
///     name: String,
/// }
///
/// impl Keyed for User {
///     type Key = u64;
///     fn key(&self) -> Self::Key {
///         self.id
///     }
/// }
/// ```
pub trait Keyed {
    /// The type of the unique key.
    ///
    /// Must be `Clone + Eq + Hash` for efficient storage in hash-based collections.
    type Key: Clone + Eq + Hash + Send + 'static;

    /// Returns the unique key for this item.
    fn key(&self) -> Self::Key;
}

impl Keyed for String {
    type Key = String;

    fn key(&self) -> Self::Key {
        self.clone()
    }
}

impl<'a> Keyed for &'a str {
    type Key = String;

    fn key(&self) -> Self::Key {
        (*self).to_string()
    }
}

impl Keyed for u64 {
    type Key = u64;

    fn key(&self) -> Self::Key {
        *self
    }
}

impl Keyed for u32 {
    type Key = u32;

    fn key(&self) -> Self::Key {
        *self
    }
}

impl Keyed for usize {
    type Key = usize;

    fn key(&self) -> Self::Key {
        *self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_keyed() {
        let s = String::from("test");
        assert_eq!(s.key(), "test");
    }

    #[test]
    fn str_keyed() {
        let s = "test";
        assert_eq!(s.key(), "test".to_string());
    }

    #[test]
    fn u64_keyed() {
        let n: u64 = 42;
        assert_eq!(n.key(), 42);
    }

    #[test]
    fn custom_struct_keyed() {
        struct Item {
            id: u64,
        }

        impl Keyed for Item {
            type Key = u64;
            fn key(&self) -> Self::Key {
                self.id
            }
        }

        let item = Item { id: 123 };
        assert_eq!(item.key(), 123);
    }
}
