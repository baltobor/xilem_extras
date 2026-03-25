//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

use std::collections::HashSet;
use std::hash::Hash;

use crate::traits::TreeNode;

/// Tracks which nodes in a tree are expanded.
///
/// This state is kept separately from the data model, allowing the same
/// data to be rendered with different expansion states.
///
/// # Example
///
/// ```
/// use xilem_extras::ExpansionState;
///
/// let mut expansion = ExpansionState::<String>::new();
///
/// // Expand a node
/// expansion.expand("folder1".to_string());
/// assert!(expansion.is_expanded(&"folder1".to_string()));
///
/// // Toggle expansion
/// expansion.toggle(&"folder1".to_string());
/// assert!(!expansion.is_expanded(&"folder1".to_string()));
/// ```
#[derive(Debug, Clone)]
pub struct ExpansionState<Id: Clone + Eq + Hash> {
    expanded: HashSet<Id>,
}

impl<Id: Clone + Eq + Hash> Default for ExpansionState<Id> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Id: Clone + Eq + Hash> ExpansionState<Id> {
    /// Creates a new empty expansion state (all nodes collapsed).
    pub fn new() -> Self {
        Self {
            expanded: HashSet::new(),
        }
    }

    /// Creates an expansion state with the given nodes expanded.
    pub fn with_expanded(ids: impl IntoIterator<Item = Id>) -> Self {
        Self {
            expanded: ids.into_iter().collect(),
        }
    }

    /// Returns whether the given node is expanded.
    pub fn is_expanded(&self, id: &Id) -> bool {
        self.expanded.contains(id)
    }

    /// Toggles the expansion state of a node.
    pub fn toggle(&mut self, id: &Id) {
        if self.expanded.contains(id) {
            self.expanded.remove(id);
        } else {
            self.expanded.insert(id.clone());
        }
    }

    /// Expands a node.
    pub fn expand(&mut self, id: Id) {
        self.expanded.insert(id);
    }

    /// Collapses a node.
    pub fn collapse(&mut self, id: &Id) {
        self.expanded.remove(id);
    }

    /// Expands all expandable nodes in the tree.
    pub fn expand_all<N: TreeNode<Id = Id>>(&mut self, root: &N) {
        self.expand_recursive(root);
    }

    fn expand_recursive<N: TreeNode<Id = Id>>(&mut self, node: &N) {
        if node.is_expandable() {
            self.expanded.insert(node.id());
        }
        for child in node.children() {
            self.expand_recursive(child);
        }
    }

    /// Collapses all nodes.
    pub fn collapse_all(&mut self) {
        self.expanded.clear();
    }

    /// Returns the number of expanded nodes.
    pub fn count(&self) -> usize {
        self.expanded.len()
    }

    /// Returns whether any nodes are expanded.
    pub fn is_empty(&self) -> bool {
        self.expanded.is_empty()
    }

    /// Returns an iterator over expanded node IDs.
    pub fn iter(&self) -> impl Iterator<Item = &Id> {
        self.expanded.iter()
    }

    /// Expands the path from root to the given node.
    ///
    /// This expands all ancestor nodes so that the target node becomes visible.
    pub fn expand_to<N: TreeNode<Id = Id>>(&mut self, root: &N, target_id: &Id) {
        self.expand_path_recursive(root, target_id);
    }

    fn expand_path_recursive<N: TreeNode<Id = Id>>(&mut self, node: &N, target_id: &Id) -> bool {
        if &node.id() == target_id {
            return true;
        }

        for child in node.children() {
            if self.expand_path_recursive(child, target_id) {
                self.expanded.insert(node.id());
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Identifiable;

    #[derive(Debug, Clone)]
    struct TestNode {
        id: String,
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
            &self.id
        }
    }

    fn create_test_tree() -> TestNode {
        TestNode {
            id: "root".into(),
            children: vec![
                TestNode {
                    id: "a".into(),
                    children: vec![
                        TestNode {
                            id: "a1".into(),
                            children: vec![],
                        },
                        TestNode {
                            id: "a2".into(),
                            children: vec![],
                        },
                    ],
                },
                TestNode {
                    id: "b".into(),
                    children: vec![
                        TestNode {
                            id: "b1".into(),
                            children: vec![],
                        },
                    ],
                },
            ],
        }
    }

    #[test]
    fn new_is_empty() {
        let state = ExpansionState::<String>::new();
        assert!(state.is_empty());
        assert_eq!(state.count(), 0);
    }

    #[test]
    fn with_expanded_creates_state() {
        let state = ExpansionState::with_expanded(["a".to_string(), "b".to_string()]);
        assert!(state.is_expanded(&"a".to_string()));
        assert!(state.is_expanded(&"b".to_string()));
        assert!(!state.is_expanded(&"c".to_string()));
    }

    #[test]
    fn expand_adds_node() {
        let mut state = ExpansionState::<String>::new();
        state.expand("test".to_string());
        assert!(state.is_expanded(&"test".to_string()));
    }

    #[test]
    fn collapse_removes_node() {
        let mut state = ExpansionState::with_expanded(["test".to_string()]);
        state.collapse(&"test".to_string());
        assert!(!state.is_expanded(&"test".to_string()));
    }

    #[test]
    fn toggle_expands_collapsed() {
        let mut state = ExpansionState::<String>::new();
        state.toggle(&"test".to_string());
        assert!(state.is_expanded(&"test".to_string()));
    }

    #[test]
    fn toggle_collapses_expanded() {
        let mut state = ExpansionState::with_expanded(["test".to_string()]);
        state.toggle(&"test".to_string());
        assert!(!state.is_expanded(&"test".to_string()));
    }

    #[test]
    fn expand_all_expands_tree() {
        let tree = create_test_tree();
        let mut state = ExpansionState::new();
        state.expand_all(&tree);

        // All nodes with children should be expanded
        assert!(state.is_expanded(&"root".to_string()));
        assert!(state.is_expanded(&"a".to_string()));
        assert!(state.is_expanded(&"b".to_string()));

        // Leaf nodes are not expandable, so not in expanded set
        assert!(!state.is_expanded(&"a1".to_string()));
        assert!(!state.is_expanded(&"a2".to_string()));
        assert!(!state.is_expanded(&"b1".to_string()));
    }

    #[test]
    fn collapse_all_clears() {
        let tree = create_test_tree();
        let mut state = ExpansionState::new();
        state.expand_all(&tree);
        state.collapse_all();
        assert!(state.is_empty());
    }

    #[test]
    fn expand_to_expands_path() {
        let tree = create_test_tree();
        let mut state = ExpansionState::new();
        state.expand_to(&tree, &"a1".to_string());

        // Path from root to a1 should be expanded
        assert!(state.is_expanded(&"root".to_string()));
        assert!(state.is_expanded(&"a".to_string()));

        // Other branches should not be expanded
        assert!(!state.is_expanded(&"b".to_string()));
    }

    #[test]
    fn expand_to_nonexistent_does_nothing() {
        let tree = create_test_tree();
        let mut state = ExpansionState::new();
        state.expand_to(&tree, &"nonexistent".to_string());
        assert!(state.is_empty());
    }

    #[test]
    fn expand_to_root_does_nothing() {
        let tree = create_test_tree();
        let mut state = ExpansionState::new();
        state.expand_to(&tree, &"root".to_string());
        // Root has no ancestors to expand
        assert!(state.is_empty());
    }

    #[test]
    fn iter_returns_all_expanded() {
        let state = ExpansionState::with_expanded(["a".to_string(), "b".to_string()]);
        let expanded: HashSet<_> = state.iter().cloned().collect();
        assert_eq!(expanded, HashSet::from(["a".to_string(), "b".to_string()]));
    }

    #[test]
    fn count_returns_correct_value() {
        let state = ExpansionState::with_expanded(["a".to_string(), "b".to_string(), "c".to_string()]);
        assert_eq!(state.count(), 3);
    }

    #[test]
    fn default_is_empty() {
        let state: ExpansionState<String> = Default::default();
        assert!(state.is_empty());
    }

    #[test]
    fn collapse_nonexistent_is_noop() {
        let mut state = ExpansionState::<String>::new();
        state.collapse(&"nonexistent".to_string());
        assert!(state.is_empty());
    }

    #[test]
    fn expand_idempotent() {
        let mut state = ExpansionState::<String>::new();
        state.expand("test".to_string());
        state.expand("test".to_string());
        assert_eq!(state.count(), 1);
    }
}
