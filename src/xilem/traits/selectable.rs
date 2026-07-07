//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

use std::hash::Hash;

/// Keyboard/mouse modifiers affecting selection behavior.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SelectionModifiers {
    /// Cmd (macOS) or Ctrl (Windows/Linux) - toggle individual selection
    pub command: bool,
    /// Shift - extend selection range
    pub shift: bool,
    /// Alt/Option - alternative toggle modifier
    pub alt: bool,
}

impl SelectionModifiers {
    /// No modifiers pressed.
    pub const NONE: Self = Self {
        command: false,
        shift: false,
        alt: false,
    };

    /// Command/Ctrl modifier pressed.
    pub const COMMAND: Self = Self {
        command: true,
        shift: false,
        alt: false,
    };

    /// Shift modifier pressed.
    pub const SHIFT: Self = Self {
        command: false,
        shift: true,
        alt: false,
    };

    /// Alt/Option modifier pressed.
    pub const ALT: Self = Self {
        command: false,
        shift: false,
        alt: true,
    };

    /// Both Command/Ctrl and Shift pressed.
    pub const BOTH: Self = Self {
        command: true,
        shift: true,
        alt: false,
    };

    /// Create from masonry's Modifiers type.
    ///
    /// Maps Cmd (macOS) or Ctrl (Windows/Linux) to `command`.
    #[cfg(target_os = "macos")]
    pub fn from_modifiers(modifiers: xilem::masonry::core::Modifiers) -> Self {
        Self {
            command: modifiers.meta(),
            shift: modifiers.shift(),
            alt: modifiers.alt(),
        }
    }

    /// Create from masonry's Modifiers type.
    ///
    /// Maps Cmd (macOS) or Ctrl (Windows/Linux) to `command`.
    #[cfg(not(target_os = "macos"))]
    pub fn from_modifiers(modifiers: xilem::masonry::core::Modifiers) -> Self {
        Self {
            command: modifiers.ctrl(),
            shift: modifiers.shift(),
            alt: modifiers.alt(),
        }
    }
}

/// Strategy trait for managing selection state.
///
/// Implementations can provide single selection, multi-selection,
/// or custom selection behaviors.
///
/// # Type Parameters
///
/// - `Id`: The identifier type for selectable items (must be `Clone + Eq + Hash`).
///
/// # Example
///
/// ```
/// use xilem_extras::{SelectionState, SelectionModifiers, SingleSelection};
///
/// let mut selection = SingleSelection::<u64>::new();
/// selection.select(42, SelectionModifiers::NONE);
/// assert!(selection.is_selected(&42));
/// ```
pub trait SelectionState<Id: Clone + Eq + Hash> {
    /// Returns whether the given item is currently selected.
    fn is_selected(&self, id: &Id) -> bool;

    /// Updates the selection based on the given item and modifiers.
    ///
    /// Behavior depends on the implementation:
    /// - `SingleSelection`: Replaces selection, toggles on same item
    /// - `MultiSelection`: Uses modifiers for toggle/range selection
    fn select(&mut self, id: Id, modifiers: SelectionModifiers);

    /// Clears all selection.
    fn clear(&mut self);

    /// Returns the number of selected items.
    fn count(&self) -> usize;

    /// Returns whether any items are selected.
    fn is_empty(&self) -> bool {
        self.count() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modifiers_none() {
        let m = SelectionModifiers::NONE;
        assert!(!m.command);
        assert!(!m.shift);
        assert!(!m.alt);
    }

    #[test]
    fn test_modifiers_command() {
        let m = SelectionModifiers::COMMAND;
        assert!(m.command);
        assert!(!m.shift);
        assert!(!m.alt);
    }

    #[test]
    fn test_modifiers_shift() {
        let m = SelectionModifiers::SHIFT;
        assert!(!m.command);
        assert!(m.shift);
        assert!(!m.alt);
    }

    #[test]
    fn test_modifiers_alt() {
        let m = SelectionModifiers::ALT;
        assert!(!m.command);
        assert!(!m.shift);
        assert!(m.alt);
    }

    #[test]
    fn test_modifiers_both() {
        let m = SelectionModifiers::BOTH;
        assert!(m.command);
        assert!(m.shift);
        assert!(!m.alt);
    }

    #[test]
    fn test_modifiers_default() {
        let m = SelectionModifiers::default();
        assert_eq!(m, SelectionModifiers::NONE);
    }
}
