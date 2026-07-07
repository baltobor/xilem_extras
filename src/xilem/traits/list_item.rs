//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

use super::Identifiable;

/// A flat list item with a stable identity.
///
/// This is a simpler trait than `TreeNode` for non-hierarchical lists.
/// For hierarchical lists, use `TreeNode` instead.
///
/// # Example
///
/// ```
/// use xilem_extras::{Identifiable, ListItem};
///
/// struct Contact {
///     id: u64,
///     name: String,
///     email: String,
/// }
///
/// impl Identifiable for Contact {
///     type Id = u64;
///     fn id(&self) -> Self::Id {
///         self.id
///     }
/// }
///
/// impl ListItem for Contact {
///     fn label(&self) -> &str {
///         &self.name
///     }
/// }
/// ```
pub trait ListItem: Identifiable {
    /// Returns the primary display text for this item.
    fn label(&self) -> &str;

    /// Returns optional secondary text (subtitle).
    fn subtitle(&self) -> Option<&str> {
        None
    }

    /// Returns whether this item is enabled.
    fn is_enabled(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SimpleItem {
        id: u32,
        name: String,
    }

    impl Identifiable for SimpleItem {
        type Id = u32;
        fn id(&self) -> Self::Id {
            self.id
        }
    }

    impl ListItem for SimpleItem {
        fn label(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_default_subtitle() {
        let item = SimpleItem {
            id: 1,
            name: "Test".into(),
        };
        assert_eq!(item.subtitle(), None);
    }

    #[test]
    fn test_default_enabled() {
        let item = SimpleItem {
            id: 1,
            name: "Test".into(),
        };
        assert!(item.is_enabled());
    }

    #[test]
    fn test_label() {
        let item = SimpleItem {
            id: 1,
            name: "Test Item".into(),
        };
        assert_eq!(item.label(), "Test Item");
    }
}
