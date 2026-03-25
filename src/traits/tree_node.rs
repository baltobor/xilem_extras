//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

use super::Identifiable;

/// A node in a hierarchical tree structure.
///
/// This trait extends `Identifiable` to provide tree-specific functionality:
/// - Access to children nodes
/// - Expandability state
/// - Display label
///
/// # Example
///
/// ```
/// use xilem_extras::{Identifiable, TreeNode};
///
/// struct FileNode {
///     path: String,
///     name: String,
///     is_dir: bool,
///     children: Vec<FileNode>,
/// }
///
/// impl Identifiable for FileNode {
///     type Id = String;
///     fn id(&self) -> Self::Id {
///         self.path.clone()
///     }
/// }
///
/// impl TreeNode for FileNode {
///     fn children(&self) -> &[Self] {
///         &self.children
///     }
///
///     fn is_expandable(&self) -> bool {
///         self.is_dir
///     }
///
///     fn label(&self) -> &str {
///         &self.name
///     }
/// }
/// ```
pub trait TreeNode: Identifiable + Sized {
    /// Returns the direct children of this node.
    fn children(&self) -> &[Self];

    /// Returns whether this node can be expanded.
    ///
    /// Default implementation returns `true` if the node has children.
    /// Override for nodes that should show expand affordance even when empty
    /// (e.g., lazy-loaded folders).
    fn is_expandable(&self) -> bool {
        !self.children().is_empty()
    }

    /// Returns the display label for this node.
    fn label(&self) -> &str;

    /// Returns an iterator that yields all nodes in the tree in pre-order.
    fn iter_preorder(&self) -> PreorderIterator<'_, Self> {
        PreorderIterator {
            stack: vec![self],
        }
    }

    /// Returns the depth of this node (root = 0).
    ///
    /// This requires traversing from root, so use sparingly.
    fn depth_in(&self, root: &Self) -> Option<usize> {
        fn find_depth<T: TreeNode>(node: &T, target_id: &T::Id, current_depth: usize) -> Option<usize> {
            if node.id() == *target_id {
                return Some(current_depth);
            }
            for child in node.children() {
                if let Some(depth) = find_depth(child, target_id, current_depth + 1) {
                    return Some(depth);
                }
            }
            None
        }
        find_depth(root, &self.id(), 0)
    }
}

/// Pre-order iterator over tree nodes.
pub struct PreorderIterator<'a, T: TreeNode> {
    stack: Vec<&'a T>,
}

impl<'a, T: TreeNode> Iterator for PreorderIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        // Push children in reverse order so leftmost child is processed first
        for child in node.children().iter().rev() {
            self.stack.push(child);
        }
        Some(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_is_expandable_with_children() {
        let tree = create_test_tree();
        assert!(tree.is_expandable());
    }

    #[test]
    fn test_is_expandable_without_children() {
        let leaf = TestNode {
            id: "leaf".into(),
            name: "Leaf".into(),
            children: vec![],
        };
        assert!(!leaf.is_expandable());
    }

    #[test]
    fn test_preorder_traversal() {
        let tree = create_test_tree();
        let ids: Vec<_> = tree.iter_preorder().map(|n| n.id()).collect();
        assert_eq!(ids, vec!["root", "a", "a1", "a2", "b"]);
    }

    #[test]
    fn test_preorder_single_node() {
        let leaf = TestNode {
            id: "leaf".into(),
            name: "Leaf".into(),
            children: vec![],
        };
        let ids: Vec<_> = leaf.iter_preorder().map(|n| n.id()).collect();
        assert_eq!(ids, vec!["leaf"]);
    }

    #[test]
    fn test_depth_in_root() {
        let tree = create_test_tree();
        assert_eq!(tree.depth_in(&tree), Some(0));
    }

    #[test]
    fn test_depth_in_child() {
        let tree = create_test_tree();
        let a1 = &tree.children[0].children[0];
        assert_eq!(a1.depth_in(&tree), Some(2));
    }

    #[test]
    fn test_depth_in_not_found() {
        let tree = create_test_tree();
        let orphan = TestNode {
            id: "orphan".into(),
            name: "Orphan".into(),
            children: vec![],
        };
        assert_eq!(orphan.depth_in(&tree), None);
    }
}
