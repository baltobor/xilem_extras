//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Tree views for xilem.
//!
//! There are two public surfaces:
//!
//! 1. [`tree_view`] / [`TreeView`] — the canonical tree view. Built from the
//!    [`disclosure_row`](crate::tree::DEFAULT_INDENT_PER_DEPTH) primitive plus
//!    [`keyboard_focus`](self) for keyboard navigation. Use this.
//! 2. [`tree_group`] / `tree_group_styled` / `tree_forest` etc. — older,
//!    keyboard-less helpers kept as **legacy reference**. Do not extend.

mod disclosure_row;
mod expansion_state;
mod flatten;
mod keyboard_focus;
mod scroll_focus;
mod tree_view;
mod tree_view_builder;
mod types;

pub use expansion_state::ExpansionState;

// Shared value types used by both the canonical builder and the legacy
// `tree_group` family. Lives in its own module so the legacy file stays a
// self-contained delete-target.
pub use types::{TreeAction, TreeStyle};

// Legacy: tree_group family (no keyboard navigation).
pub use tree_view::{
    flatten_forest, flatten_tree, tree, tree_forest, tree_forest_styled,
    tree_forest_with_context_menu, tree_group, tree_group_styled, tree_group_with_context_menu,
    tree_group_with_context_menu_editable,
};

// Canonical tree view. `tree_view` is the single-root constructor;
// `tree_forest_view` is the multi-root flavour. Both share `TreeView`
// as their builder type so every opt-in (selection, icon_for, …)
// works identically across the two.
pub use tree_view_builder::{
    DEFAULT_CHEVRON_COLOR, DEFAULT_SELECTED_BG, DEFAULT_TEXT_COLOR, HighlightFill, TreeView,
    tree_forest_view, tree_view,
};

// Primitives — exposed for power users building custom tree-like views.
pub use disclosure_row::disclosure_row;
pub use flatten::{FlattenedNode, flatten_forest_with_parents, flatten_tree_with_parents};
pub use keyboard_focus::{KeyAction, KeyHandler, KeyboardFocus, keyboard_focus};
pub use scroll_focus::{DEFAULT_ROW_HEIGHT_HINT, ScrollFocus, scroll_focus};
