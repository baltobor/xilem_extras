//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Shared value types for the tree-view family.
//!
//! `TreeAction` is dispatched by both the canonical `tree_view` builder and
//! the legacy `tree_group` helpers. `TreeStyle` configures row-level layout
//! (indent per depth, chevron column width, hover bg, row gap) and is also
//! shared. Keeping these in their own module lets the legacy file
//! (`tree_view.rs`) stay a self-contained delete-target without dragging
//! these types with it.

use xilem::masonry::kurbo::Point;
use xilem::masonry::peniko::Color;

/// Actions that can occur on tree nodes.
#[derive(Debug, Clone, PartialEq)]
pub enum TreeAction {
    /// Expand or collapse (single click on expandable node)
    Toggle,
    /// Single click selection (on leaf nodes)
    Select,
    /// Double click activation (e.g., open file)
    DoubleClick,
    /// Right click context menu at the given position
    ContextMenu(Point),
    /// Start inline editing (triggered by F2 key or context menu).
    /// The handler receives the node_id and should set editing state so the
    /// row_builder can render a text_input instead of a label.
    StartEdit,
    /// Commit inline editing with the new text value.
    /// Sent when user presses Enter in the edit input.
    CommitEdit(String),
    /// Cancel inline editing without saving.
    /// Sent when user presses Escape in the edit input.
    CancelEdit,
}

/// Style configuration for tree rows.
#[derive(Debug, Clone)]
pub struct TreeStyle {
    /// Background color on hover.
    pub hover_bg: Color,
    /// Indentation per depth level in pixels.
    pub indent: f64,
    /// Width of the chevron column, in pixels. Both expandable rows
    /// (chevron + row_button) and leaf rows (transparent placeholder)
    /// occupy exactly this width, so a leaf at depth N aligns horizontally
    /// with an expandable sibling at the same depth.
    pub chevron_col_width: f64,
    /// Gap between rows in pixels.
    pub gap: f64,
}

impl Default for TreeStyle {
    fn default() -> Self {
        Self {
            hover_bg: Color::TRANSPARENT,
            indent: 20.0,
            chevron_col_width: 16.0,
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

    /// Sets the chevron column width.
    pub fn chevron_col_width(mut self, width: f64) -> Self {
        self.chevron_col_width = width;
        self
    }

    /// Sets the gap between rows.
    pub fn gap(mut self, gap: f64) -> Self {
        self.gap = gap;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_action_equality() {
        assert_eq!(TreeAction::Toggle, TreeAction::Toggle);
        assert_ne!(TreeAction::Toggle, TreeAction::Select);
        assert_ne!(TreeAction::Select, TreeAction::DoubleClick);
        assert_ne!(TreeAction::DoubleClick, TreeAction::ContextMenu(Point::ZERO));
        assert_eq!(TreeAction::StartEdit, TreeAction::StartEdit);
        assert_ne!(TreeAction::StartEdit, TreeAction::Select);
        assert_eq!(TreeAction::CommitEdit("test".to_string()), TreeAction::CommitEdit("test".to_string()));
        assert_ne!(TreeAction::CommitEdit("a".to_string()), TreeAction::CommitEdit("b".to_string()));
        assert_eq!(TreeAction::CancelEdit, TreeAction::CancelEdit);
    }

    #[test]
    fn tree_style_default() {
        let style = TreeStyle::default();
        assert_eq!(style.hover_bg, Color::TRANSPARENT);
        assert_eq!(style.indent, 20.0);
        assert_eq!(style.chevron_col_width, 16.0);
        assert_eq!(style.gap, 0.0);
    }

    #[test]
    fn tree_style_builder() {
        let style = TreeStyle::new()
            .hover_bg(Color::from_rgb8(50, 50, 50))
            .indent(28.0)
            .chevron_col_width(24.0)
            .gap(2.0);

        assert_eq!(style.indent, 28.0);
        assert_eq!(style.chevron_col_width, 24.0);
        assert_eq!(style.gap, 2.0);
    }
}
