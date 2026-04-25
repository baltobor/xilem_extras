//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

mod expansion_state;
mod tree_view;
mod tree_view_builder;

pub use expansion_state::ExpansionState;
pub use tree_view::{
    tree, tree_group, tree_group_styled, tree_group_with_context_menu,
    tree_group_with_context_menu_editable,
    tree_forest, tree_forest_styled, tree_forest_with_context_menu,
    TreeAction, TreeStyle, flatten_tree, flatten_forest,
};
pub use tree_view_builder::{
    tree_view, TreeView, DEFAULT_SELECTED_BG, DEFAULT_TEXT_COLOR, DEFAULT_CHEVRON_COLOR,
};
