//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

use masonry::layout::AsUnit;
use xilem::masonry::vello::peniko::Color;
use xilem::style::Style;
use xilem::view::flex_col;
use xilem::{WidgetView, AnyWidgetView};

use crate::traits::{SelectionState, TreeNode};
use crate::row_button_with_clicks;
use super::ExpansionState;

/// Actions that can occur on tree nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeAction {
    /// Expand or collapse (single click on expandable node)
    Toggle,
    /// Single click selection (on leaf nodes)
    Select,
    /// Double click activation (e.g., open file)
    DoubleClick,
    /// Right click context menu
    ContextMenu,
}

/// Style configuration for tree rows.
#[derive(Debug, Clone)]
pub struct TreeStyle {
    /// Background color on hover.
    pub hover_bg: Color,
    /// Indentation per depth level in pixels.
    pub indent: f64,
    /// Gap between rows in pixels.
    pub gap: f64,
}

impl Default for TreeStyle {
    fn default() -> Self {
        Self {
            hover_bg: Color::TRANSPARENT,
            indent: 16.0,
            gap: 0.0,
        }
    }
}

impl TreeStyle {
    /// Creates a new TreeStyle with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the hover background color.
    pub fn hover_bg(mut self, color: Color) -> Self {
        self.hover_bg = color;
        self
    }

    /// Sets the indentation per depth level.
    pub fn indent(mut self, indent: f64) -> Self {
        self.indent = indent;
        self
    }

    /// Sets the gap between rows.
    pub fn gap(mut self, gap: f64) -> Self {
        self.gap = gap;
        self
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

/// Creates a tree group for hierarchical data.
///
/// The library handles recursion automatically. You provide a row builder and handler.
///
/// # Arguments
///
/// * `root` - The root node of the tree
/// * `expansion` - Tracks which nodes are expanded
/// * `selection` - Optional selection state (use `None::<&SingleSelection<Id>>` if not needed)
/// * `row_builder` - Function that builds a view for each node: `(node, depth, is_expanded, is_selected) -> View`
/// * `handler` - Function that handles tree actions and mutates state: `(state, node_id, action) -> ()`
///
/// # Example
///
/// ```ignore
/// tree_group(
///     &model.file_tree,
///     &model.expansion,
///     Some(&model.selection),
///     |node, depth, is_expanded, is_selected| {
///         flex_row((
///             if node.is_expandable() {
///                 disclosure(is_expanded).boxed()
///             } else {
///                 sized_box(()).width(16.0).boxed()
///             },
///             label(node.label()),
///         ))
///         .padding_left(depth as f64 * 16.0)
///     },
///     |state, node_id, action| {
///         match action {
///             TreeAction::Toggle => state.expansion.toggle(node_id),
///             TreeAction::Select => state.selected = Some(node_id.clone()),
///             TreeAction::DoubleClick => state.open(node_id),
///             TreeAction::ContextMenu => state.show_context_menu(node_id),
///         }
///     },
/// )
/// ```
pub fn tree_group<'a, State, N, R, F, H, Sel>(
    root: &'a N,
    expansion: &'a ExpansionState<N::Id>,
    selection: Option<&'a Sel>,
    row_builder: F,
    handler: H,
) -> impl WidgetView<State, ()> + use<'a, State, N, R, F, H, Sel>
where
    State: 'static,
    N: TreeNode + 'a,
    N::Id: Clone + Send + Sync + 'static,
    R: WidgetView<State, ()> + 'static,
    F: Fn(&N, usize, bool, bool) -> R + Clone + 'a,
    H: Fn(&mut State, &N::Id, TreeAction) + Clone + Send + Sync + 'static,
    Sel: SelectionState<N::Id> + 'a,
{
    tree_group_styled(root, expansion, selection, TreeStyle::default(), row_builder, handler)
}

/// Creates a tree group with custom styling.
///
/// Same as [`tree_group`] but accepts a [`TreeStyle`] for customization.
///
/// # Example
///
/// ```ignore
/// tree_group_styled(
///     &model.file_tree,
///     &model.expansion,
///     Some(&model.selection),
///     TreeStyle::new().hover_bg(Color::from_rgb8(55, 53, 50)),
///     |node, depth, is_expanded, is_selected| { ... },
///     |state, node_id, action| { ... },
/// )
/// ```
pub fn tree_group_styled<'a, State, N, R, F, H, Sel>(
    root: &'a N,
    expansion: &'a ExpansionState<N::Id>,
    selection: Option<&'a Sel>,
    style: TreeStyle,
    row_builder: F,
    handler: H,
) -> impl WidgetView<State, ()> + use<'a, State, N, R, F, H, Sel>
where
    State: 'static,
    N: TreeNode + 'a,
    N::Id: Clone + Send + Sync + 'static,
    R: WidgetView<State, ()> + 'static,
    F: Fn(&N, usize, bool, bool) -> R + Clone + 'a,
    H: Fn(&mut State, &N::Id, TreeAction) + Clone + Send + Sync + 'static,
    Sel: SelectionState<N::Id> + 'a,
{
    let mut flat_nodes: Vec<(&N, usize, bool)> = Vec::new();
    flatten_tree(root, expansion, 0, &mut flat_nodes);

    let rows: Vec<Box<AnyWidgetView<State, ()>>> = flat_nodes
        .into_iter()
        .map(|(node, depth, is_expanded)| {
            let is_selected = selection
                .map(|sel| sel.is_selected(&node.id()))
                .unwrap_or(false);

            let row_view = row_builder(node, depth, is_expanded, is_selected);
            let node_id = node.id();
            let is_expandable = node.is_expandable();
            let handler = handler.clone();
            let hover_bg = style.hover_bg;

            let btn = row_button_with_clicks(row_view, move |state: &mut State, click_count: u8| {
                let action = if click_count >= 2 {
                    TreeAction::DoubleClick
                } else if is_expandable {
                    TreeAction::Toggle
                } else {
                    TreeAction::Select
                };
                handler(state, &node_id, action);
            })
            .hover_bg(hover_bg);

            btn.boxed()
        })
        .collect();

    if style.gap > 0.0 {
        flex_col(rows).gap(style.gap.px())
    } else {
        flex_col(rows).gap(0.px())
    }
}

/// Alias for tree_group.
///
/// See [`tree_group`] for full documentation.
pub fn tree<'a, State, N, R, F, H, Sel>(
    root: &'a N,
    expansion: &'a ExpansionState<N::Id>,
    selection: Option<&'a Sel>,
    row_builder: F,
    handler: H,
) -> impl WidgetView<State, ()> + use<'a, State, N, R, F, H, Sel>
where
    State: 'static,
    N: TreeNode + 'a,
    N::Id: Clone + Send + Sync + 'static,
    R: WidgetView<State, ()> + 'static,
    F: Fn(&N, usize, bool, bool) -> R + Clone + 'a,
    H: Fn(&mut State, &N::Id, TreeAction) + Clone + Send + Sync + 'static,
    Sel: SelectionState<N::Id> + 'a,
{
    tree_group(root, expansion, selection, row_builder, handler)
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

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0.id(), "root");
        assert_eq!(result[0].1, 0);
        assert!(!result[0].2);
    }

    #[test]
    fn flatten_expanded_root() {
        let tree = create_test_tree();
        let mut expansion = ExpansionState::new();
        expansion.expand("root".to_string());
        let mut result = Vec::new();

        flatten_tree(&tree, &expansion, 0, &mut result);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].0.id(), "root");
        assert!(result[0].2);
        assert_eq!(result[1].0.id(), "a");
        assert_eq!(result[2].0.id(), "b");
    }

    #[test]
    fn flatten_fully_expanded() {
        let tree = create_test_tree();
        let mut expansion = ExpansionState::new();
        expansion.expand_all(&tree);
        let mut result = Vec::new();

        flatten_tree(&tree, &expansion, 0, &mut result);

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
    fn tree_action_equality() {
        assert_eq!(TreeAction::Toggle, TreeAction::Toggle);
        assert_ne!(TreeAction::Toggle, TreeAction::Select);
        assert_ne!(TreeAction::Select, TreeAction::DoubleClick);
        assert_ne!(TreeAction::DoubleClick, TreeAction::ContextMenu);
    }

    #[test]
    fn tree_style_default() {
        let style = TreeStyle::default();
        assert_eq!(style.hover_bg, Color::TRANSPARENT);
        assert_eq!(style.indent, 16.0);
        assert_eq!(style.gap, 0.0);
    }

    #[test]
    fn tree_style_builder() {
        let style = TreeStyle::new()
            .hover_bg(Color::from_rgb8(50, 50, 50))
            .indent(20.0)
            .gap(2.0);

        assert_eq!(style.indent, 20.0);
        assert_eq!(style.gap, 2.0);
    }
}
