//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Core trait for menu entries.

use xilem::masonry::core::{NewWidget, Widget};

/// A boxed menu entry for type-erased storage.
pub type BoxedMenuEntry<State, Action> = Box<dyn MenuEntry<State, Action>>;

/// Trait for items that can appear in a menu.
///
/// Implemented by `MenuItem` (clickable items), `MenuSeparator` (visual dividers),
/// and `Submenu` (nested menus).
/// Each entry can build its widget representation and optionally provide an action callback.
pub trait MenuEntry<State, Action>: Send + Sync + 'static {
    /// Returns the label for this entry, or `None` for separators.
    fn label(&self) -> Option<&str>;

    /// Returns `true` if this entry is actionable (not a separator or submenu).
    fn is_actionable(&self) -> bool;

    /// Returns `true` if this entry triggers inline editing mode.
    /// Default is `false`.
    fn is_editable(&self) -> bool {
        false
    }

    /// Returns `true` if this entry is a submenu.
    /// Default is `false`.
    fn is_submenu(&self) -> bool {
        false
    }

    /// Returns the checked state for this entry.
    /// `None` means no checkmark, `Some(true)` means checked, `Some(false)` means unchecked.
    /// Default is `None`.
    fn checked(&self) -> Option<bool> {
        None
    }

    /// Returns the nested menu items if this is a submenu.
    /// Default is `None`.
    fn submenu_items(&self) -> Option<Vec<BoxedMenuEntry<State, Action>>> {
        None
    }

    /// Builds the widget representation of this menu entry.
    fn build_widget(&self) -> NewWidget<dyn Widget>;

    /// Executes the action for this entry, if it has one.
    /// Returns the action result or `None` if not actionable.
    fn execute(&self, state: &mut State) -> Option<Action>;
}
