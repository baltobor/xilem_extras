//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

use std::marker::PhantomData;

use xilem::view::{flex_col, Flex};
use xilem::WidgetView;

use crate::traits::{SelectionState, TreeNode};
use super::ExpansionState;

/// Actions that can occur on tree nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeAction {
    /// Expand or collapse (chevron click)
    Toggle,
    /// Single click selection
    Select,
    /// Double click activation (e.g., open file)
    DoubleClick,
    /// Right click context menu
    ContextMenu,
}

/// A tree view that renders hierarchical data.
///
/// # Type Parameters
///
/// - `State`: Application state type
/// - `Action`: Action type returned by callbacks
/// - `N`: Tree node type
/// - `R`: Row view type returned by row_builder
/// - `F`: Row builder function type
/// - `H`: Event handler function type
pub struct TreeView<State, Action, N, R, F, H, Sel = ()> {
    root: PhantomData<N>,
    row_builder: F,
    handler: H,
    row_height: f64,
    indent: f64,
    selection: Option<PhantomData<Sel>>,
    _phantom: PhantomData<(State, Action, R)>,
}

impl<State: 'static, Action: 'static, N, R, F, H> TreeView<State, Action, N, R, F, H, ()>
where
    N: TreeNode,
    R: WidgetView<State, Action>,
    F: Fn(&N, usize, bool) -> R,
    H: Fn(&mut State, &N::Id, TreeAction) -> Action,
{
    /// Sets the row height (default: 24.0).
    pub fn row_height(mut self, height: f64) -> Self {
        self.row_height = height;
        self
    }

    /// Sets the indentation per depth level (default: 16.0).
    pub fn indent(mut self, indent: f64) -> Self {
        self.indent = indent;
        self
    }

    /// Adds selection state to the tree view.
    pub fn selection<Sel: SelectionState<N::Id>>(
        self,
        _selection: &Sel,
    ) -> TreeView<State, Action, N, R, F, H, Sel> {
        TreeView {
            root: PhantomData,
            row_builder: self.row_builder,
            handler: self.handler,
            row_height: self.row_height,
            indent: self.indent,
            selection: Some(PhantomData),
            _phantom: PhantomData,
        }
    }
}

/// Creates a tree view for hierarchical data.
///
/// # Arguments
///
/// * `root` - The root node of the tree
/// * `expansion` - Tracks which nodes are expanded
/// * `row_builder` - Function that builds a view for each node: `(node, depth, is_expanded) -> View`
/// * `handler` - Function that handles tree actions: `(state, node_id, action) -> Action`
///
/// # Example
///
/// ```ignore
/// tree(
///     &model.file_tree,
///     &model.expansion,
///     |node, depth, is_expanded| {
///         flex_row((
///             if node.is_expandable() {
///                 disclosure(is_expanded).boxed()
///             } else {
///                 sized_box(16.0, 16.0).boxed()
///             },
///             label(&node.name),
///         ))
///         .padding_left(depth as f64 * 16.0)
///     },
///     |model, node_id, action| {
///         match action {
///             TreeAction::Toggle => model.expansion.toggle(node_id),
///             TreeAction::Select => model.selected = Some(node_id.clone()),
///             TreeAction::DoubleClick => model.open(node_id),
///             TreeAction::ContextMenu => model.show_menu(node_id),
///         }
///     },
/// )
/// ```
pub fn tree<'a, State: 'static, Action: 'static, N, R, F, H>(
    root: &'a N,
    expansion: &'a ExpansionState<N::Id>,
    row_builder: F,
    handler: H,
) -> TreeView<State, Action, N, R, F, H>
where
    N: TreeNode,
    R: WidgetView<State, Action>,
    F: Fn(&N, usize, bool) -> R,
    H: Fn(&mut State, &N::Id, TreeAction) -> Action,
{
    let _ = (root, expansion); // Used in actual rendering

    TreeView {
        root: PhantomData,
        row_builder,
        handler,
        row_height: 24.0,
        indent: 16.0,
        selection: None,
        _phantom: PhantomData,
    }
}

/// Collects visible tree nodes into a flat list for rendering.
pub fn flatten_tree<'a, N: TreeNode>(
    node: &'a N,
    expansion: &ExpansionState<N::Id>,
    depth: usize,
    result: &mut Vec<(&'a N, usize, bool)>,
) {
    let is_expanded = expansion.is_expanded(&node.id());
    result.push((node, depth, is_expanded));

    if is_expanded {
        for child in node.children() {
            flatten_tree(child, expansion, depth + 1, result);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Identifiable;

    #[derive(Debug, Clone)]
    struct TestNode {
        id: String,
        name: String,
        children: Vec<TestNode>,
    }

    impl Identifiable for TestNode {
        type Id = String;
        fn id(&self) -> Self::Id {
            self.id.clone()
        }
    }

    impl TreeNode for TestNode {
        fn children(&self) -> &[Self] {
            &self.children
        }

        fn label(&self) -> &str {
            &self.name
        }
    }

    fn create_test_tree() -> TestNode {
        TestNode {
            id: "root".into(),
            name: "Root".into(),
            children: vec![
                TestNode {
                    id: "a".into(),
                    name: "A".into(),
                    children: vec![
                        TestNode {
                            id: "a1".into(),
                            name: "A1".into(),
                            children: vec![],
                        },
                        TestNode {
                            id: "a2".into(),
                            name: "A2".into(),
                            children: vec![],
                        },
                    ],
                },
                TestNode {
                    id: "b".into(),
                    name: "B".into(),
                    children: vec![],
                },
            ],
        }
    }

    #[test]
    fn flatten_collapsed_tree() {
        let tree = create_test_tree();
        let expansion = ExpansionState::new();
        let mut result = Vec::new();

        flatten_tree(&tree, &expansion, 0, &mut result);

        // Only root should be visible
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0.id(), "root");
        assert_eq!(result[0].1, 0); // depth
        assert!(!result[0].2); // not expanded
    }

    #[test]
    fn flatten_expanded_root() {
        let tree = create_test_tree();
        let mut expansion = ExpansionState::new();
        expansion.expand("root".to_string());
        let mut result = Vec::new();

        flatten_tree(&tree, &expansion, 0, &mut result);

        // Root and its children should be visible
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].0.id(), "root");
        assert_eq!(result[0].1, 0);
        assert!(result[0].2); // expanded
        assert_eq!(result[1].0.id(), "a");
        assert_eq!(result[1].1, 1);
        assert_eq!(result[2].0.id(), "b");
        assert_eq!(result[2].1, 1);
    }

    #[test]
    fn flatten_fully_expanded() {
        let tree = create_test_tree();
        let mut expansion = ExpansionState::new();
        expansion.expand_all(&tree);
        let mut result = Vec::new();

        flatten_tree(&tree, &expansion, 0, &mut result);

        // All nodes should be visible
        assert_eq!(result.len(), 5);
        let ids: Vec<_> = result.iter().map(|(n, _, _)| n.id()).collect();
        assert_eq!(ids, vec!["root", "a", "a1", "a2", "b"]);
    }

    #[test]
    fn flatten_preserves_depth() {
        let tree = create_test_tree();
        let mut expansion = ExpansionState::new();
        expansion.expand_all(&tree);
        let mut result = Vec::new();

        flatten_tree(&tree, &expansion, 0, &mut result);

        let depths: Vec<_> = result.iter().map(|(_, d, _)| *d).collect();
        assert_eq!(depths, vec![0, 1, 2, 2, 1]);
    }

    #[test]
    fn flatten_partial_expansion() {
        let tree = create_test_tree();
        let mut expansion = ExpansionState::new();
        expansion.expand("root".to_string());
        expansion.expand("a".to_string());
        // b is not expanded (and has no children anyway)
        let mut result = Vec::new();

        flatten_tree(&tree, &expansion, 0, &mut result);

        let ids: Vec<_> = result.iter().map(|(n, _, _)| n.id()).collect();
        assert_eq!(ids, vec!["root", "a", "a1", "a2", "b"]);
    }

    #[test]
    fn tree_action_equality() {
        assert_eq!(TreeAction::Toggle, TreeAction::Toggle);
        assert_ne!(TreeAction::Toggle, TreeAction::Select);
        assert_ne!(TreeAction::Select, TreeAction::DoubleClick);
        assert_ne!(TreeAction::DoubleClick, TreeAction::ContextMenu);
    }
}
