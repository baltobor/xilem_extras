//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Tree flattening — the single canonical traversal used by `tree_view`.
//!
//! Walks a [`TreeNode`] tree and produces a flat list of [`FlattenedNode`]s
//! containing only the visible (expanded-into) nodes, with parent indices
//! recorded so keyboard Left-arrow navigation can jump to the parent in O(1).

use crate::traits::TreeNode;

use super::ExpansionState;

/// A flattened node in the tree with parent tracking for navigation.
#[derive(Debug, Clone)]
pub struct FlattenedNode<Id> {
    pub id: Id,
    pub depth: usize,
    pub is_expanded: bool,
    pub is_expandable: bool,
    /// Index of the parent in the flattened list (None for roots).
    pub parent_index: Option<usize>,
}

/// Flatten a tree into a list of visible nodes with parent tracking.
///
/// Recurses into a node's children only when the node is expanded.
pub fn flatten_tree_with_parents<N: TreeNode>(
    node: &N,
    expansion: &ExpansionState<N::Id>,
    depth: usize,
    parent_index: Option<usize>,
    result: &mut Vec<FlattenedNode<N::Id>>,
) where
    N::Id: Clone,
{
    let current_index = result.len();
    let is_expanded = expansion.is_expanded(&node.id());
    let is_expandable = node.is_expandable();

    result.push(FlattenedNode {
        id: node.id(),
        depth,
        is_expanded,
        is_expandable,
        parent_index,
    });

    if is_expanded {
        for child in node.children() {
            flatten_tree_with_parents(child, expansion, depth + 1, Some(current_index), result);
        }
    }
}

/// Flatten a forest (multiple roots) into a single list with parent tracking.
pub fn flatten_forest_with_parents<N: TreeNode>(
    roots: &[N],
    expansion: &ExpansionState<N::Id>,
    result: &mut Vec<FlattenedNode<N::Id>>,
) where
    N::Id: Clone,
{
    for root in roots {
        flatten_tree_with_parents(root, expansion, 0, None, result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Identifiable;

    #[derive(Clone, Debug)]
    struct Node {
        id: String,
        children: Vec<Node>,
    }

    impl Identifiable for Node {
        type Id = String;
        fn id(&self) -> Self::Id {
            self.id.clone()
        }
    }

    impl TreeNode for Node {
        fn children(&self) -> &[Self] {
            &self.children
        }
        fn label(&self) -> &str {
            &self.id
        }
    }

    fn n(id: &str, children: Vec<Node>) -> Node {
        Node { id: id.into(), children }
    }

    #[test]
    fn flattens_collapsed_root_to_single_entry() {
        let tree = n("root", vec![n("a", vec![]), n("b", vec![])]);
        let expansion = ExpansionState::<String>::new();
        let mut out = Vec::new();
        flatten_tree_with_parents(&tree, &expansion, 0, None, &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].id, "root");
        assert_eq!(out[0].parent_index, None);
    }

    #[test]
    fn flattens_expanded_root_with_children() {
        let tree = n("root", vec![n("a", vec![]), n("b", vec![])]);
        let expansion = ExpansionState::with_expanded(["root".into()]);
        let mut out = Vec::new();
        flatten_tree_with_parents(&tree, &expansion, 0, None, &mut out);
        assert_eq!(out.len(), 3);
        assert_eq!(out[1].parent_index, Some(0));
        assert_eq!(out[2].parent_index, Some(0));
    }

    #[test]
    fn parent_index_threads_through_depth() {
        let tree = n("root", vec![n("a", vec![n("a1", vec![])])]);
        let expansion = ExpansionState::with_expanded(["root".into(), "a".into()]);
        let mut out = Vec::new();
        flatten_tree_with_parents(&tree, &expansion, 0, None, &mut out);
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].parent_index, None);
        assert_eq!(out[1].parent_index, Some(0));
        assert_eq!(out[2].parent_index, Some(1));
    }
}
