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
/// Implemented by `MenuItem` (clickable items) and `MenuSeparator` (visual dividers).
/// Each entry can build its widget representation and optionally provide an action callback.
pub trait MenuEntry<State, Action>: Send + Sync + 'static {
    /// Returns the label for this entry, or `None` for separators.
    fn label(&self) -> Option<&str>;

    /// Returns `true` if this entry is actionable (not a separator).
    fn is_actionable(&self) -> bool;

    /// Returns `true` if this entry triggers inline editing mode.
    /// Default is `false`.
    fn is_editable(&self) -> bool {
        false
    }

    /// Builds the widget representation of this menu entry.
    fn build_widget(&self) -> NewWidget<dyn Widget>;

    /// Executes the action for this entry, if it has one.
    /// Returns the action result or `None` if not actionable.
    fn execute(&self, state: &mut State) -> Option<Action>;
}
