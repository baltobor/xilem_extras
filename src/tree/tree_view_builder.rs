//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Batteries-included tree view with keyboard navigation.
//!
//! Composes three small primitives — `disclosure_row` (chevron + content),
//! `keyboard_focus` (key event capture), and xilem's `flex_col` / `portal` —
//! into a SwiftUI-`List(children:)`-style tree view.
//!
//! ```ignore
//! tree_view(&model.tree, &model.expansion)
//!     .selection(&model.selection)
//!     .icon_for(|node| Some(label(icons::FOLDER).build()))
//!     .on_action(|state, id, action| match action {
//!         TreeAction::Toggle => state.expansion.toggle(id),
//!         TreeAction::Select => state.selection.set(Some(id.clone())),
//!         _ => {}
//!     })
//!     .build()
//! ```
//!
//! The chevron is the only thing that toggles expansion — clicks on the row
//! body are routed to `TreeAction::Select`, double clicks to
//! `TreeAction::DoubleClick`, right clicks to `TreeAction::ContextMenu`.
//!
//! Keyboard navigation, always enabled:
//!  * Up / Down — move selection
//!  * Left — collapse if expanded, else move to parent
//!  * Right — expand if collapsible
//!  * Space — toggle expand/collapse on selected
//!  * Enter — `TreeAction::DoubleClick` on selected
//!  * Home / End — jump to first / last
//!  * PageUp / PageDown — page navigation (10 rows per page)
//!  * F2 — `TreeAction::StartEdit` on selected

use std::sync::Arc;

use masonry::layout::AsUnit;
use masonry::properties::Padding;
use xilem::core::MessageResult;
use xilem::masonry::peniko::Color;
use xilem::style::Style;
use xilem::view::{flex_col, flex_row, label, CrossAxisAlignment};
use xilem::{AnyWidgetView, WidgetView};

use crate::components::{row_button_with_press, RowButtonPress};
use crate::traits::{Identifiable, SelectionState, TreeNode};
use xilem::masonry::core::PointerButton;

use super::disclosure_row::disclosure_row;
use super::flatten::{flatten_tree_with_parents, FlattenedNode};
use super::keyboard_focus::{keyboard_focus, KeyAction};
use super::tree_view::{TreeAction, TreeStyle};
use super::ExpansionState;

// Trait-object aliases used by the builder. Storing user-supplied closures
// as `Arc<dyn Fn>` keeps the public type stable across the chain of opt-in
// methods, so `build()` always sees the same `TreeView<...>` shape.
type IconFn<N, State> = dyn Fn(&N) -> Option<Box<AnyWidgetView<State, ()>>> + Send + Sync;
type LabelFn<N> = dyn Fn(&N) -> String + Send + Sync;
type ActionFn<N, State> =
    dyn Fn(&mut State, &<N as Identifiable>::Id, TreeAction) + Send + Sync;

/// Default selection background — a soft blue that reads on dark + light themes.
pub const DEFAULT_SELECTED_BG: Color = Color::from_rgba8(60, 80, 110, 220);
/// Default row text color — light gray, legible against `DEFAULT_SELECTED_BG`.
pub const DEFAULT_TEXT_COLOR: Color = Color::from_rgb8(220, 218, 214);
/// Default chevron color — slightly dimmer than text.
pub const DEFAULT_CHEVRON_COLOR: Color = Color::from_rgb8(180, 178, 172);
/// How many rows PageUp / PageDown moves at a time.
const PAGE_SIZE: usize = 10;

#[must_use = "TreeView builders do nothing until you call .build()"]
pub struct TreeView<'a, N, State, Sel = ()>
where
    N: TreeNode + 'a,
{
    root: &'a N,
    expansion: &'a ExpansionState<N::Id>,
    selection: Option<&'a Sel>,
    style: TreeStyle,
    selected_bg: Color,
    text_color: Color,
    chevron_color: Color,
    text_size: f32,
    row_padding: Padding,
    icon_for: Option<Arc<IconFn<N, State>>>,
    label_for: Option<Arc<LabelFn<N>>>,
    handler: Option<Arc<ActionFn<N, State>>>,
    _phantom: std::marker::PhantomData<fn(&mut State)>,
}

/// Start a new tree-view builder.
pub fn tree_view<'a, N, State>(
    root: &'a N,
    expansion: &'a ExpansionState<N::Id>,
) -> TreeView<'a, N, State>
where
    N: TreeNode + 'a,
    State: 'static,
{
    TreeView {
        root,
        expansion,
        selection: None,
        style: TreeStyle::new(),
        selected_bg: DEFAULT_SELECTED_BG,
        text_color: DEFAULT_TEXT_COLOR,
        chevron_color: DEFAULT_CHEVRON_COLOR,
        text_size: 11.0,
        row_padding: Padding {
            top: 2.0,
            bottom: 2.0,
            left: 4.0,
            right: 4.0,
        },
        icon_for: None,
        label_for: None,
        handler: None,
        _phantom: std::marker::PhantomData,
    }
}

impl<'a, N, State, Sel> TreeView<'a, N, State, Sel>
where
    N: TreeNode + 'a,
    State: 'static,
{
    pub fn selection<NewSel>(self, selection: &'a NewSel) -> TreeView<'a, N, State, NewSel>
    where
        NewSel: SelectionState<N::Id>,
    {
        TreeView {
            root: self.root,
            expansion: self.expansion,
            selection: Some(selection),
            style: self.style,
            selected_bg: self.selected_bg,
            text_color: self.text_color,
            chevron_color: self.chevron_color,
            text_size: self.text_size,
            row_padding: self.row_padding,
            icon_for: self.icon_for,
            label_for: self.label_for,
            handler: self.handler,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn style(mut self, style: TreeStyle) -> Self {
        self.style = style;
        self
    }

    pub fn selected_bg(mut self, color: Color) -> Self {
        self.selected_bg = color;
        self
    }

    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    pub fn chevron_color(mut self, color: Color) -> Self {
        self.chevron_color = color;
        self
    }

    pub fn text_size(mut self, size: f32) -> Self {
        self.text_size = size;
        self
    }

    pub fn indent(mut self, indent: f64) -> Self {
        self.style = self.style.indent(indent);
        self
    }

    pub fn hover_bg(mut self, color: Color) -> Self {
        self.style = self.style.hover_bg(color);
        self
    }

    /// Kept for API compatibility — flex-based row sizing is content-based.
    pub fn row_height(self, _height: f64) -> Self {
        self
    }

    pub fn row_padding(mut self, padding: Padding) -> Self {
        self.row_padding = padding;
        self
    }

    pub fn icon_for<F>(mut self, f: F) -> Self
    where
        F: Fn(&N) -> Option<Box<AnyWidgetView<State, ()>>> + Send + Sync + 'static,
    {
        self.icon_for = Some(Arc::new(f));
        self
    }

    pub fn label_for<F>(mut self, f: F) -> Self
    where
        F: Fn(&N) -> String + Send + Sync + 'static,
    {
        self.label_for = Some(Arc::new(f));
        self
    }

    pub fn on_action<F>(mut self, h: F) -> Self
    where
        F: Fn(&mut State, &N::Id, TreeAction) + Send + Sync + 'static,
    {
        self.handler = Some(Arc::new(h));
        self
    }
}

impl<'a, N, State, Sel> TreeView<'a, N, State, Sel>
where
    N: TreeNode + 'a,
    N::Id: Clone + Send + Sync + 'static,
    State: 'static,
    Sel: SelectionState<N::Id> + 'a,
{
    /// Build the configured tree as a Xilem view with full keyboard navigation.
    pub fn build(self) -> impl WidgetView<State, ()> + use<'a, N, State, Sel> {
        // Flatten the tree once. The result is captured both by the row-builder
        // pass (for ordered iteration) and by the keyboard handler closure
        // (for navigation lookups).
        let mut flat_nodes: Vec<FlattenedNode<N::Id>> = Vec::new();
        flatten_tree_with_parents(self.root, self.expansion, 0, None, &mut flat_nodes);

        // Compute selected index from current SelectionState.
        let selected_index = self
            .selection
            .and_then(|sel| flat_nodes.iter().position(|n| sel.is_selected(&n.id)));

        // Build all visible rows recursively. Each chevron is a button that
        // dispatches TreeAction::Toggle; each row body is a row_button_with_press
        // that dispatches Select / DoubleClick / ContextMenu.
        let rows = self.build_rows(&flat_nodes, selected_index);

        let content = flex_col(rows)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .gap(0.px());

        // Capture state needed by the keyboard handler.
        let user_handler = self.handler.clone();
        let flat_for_keys = flat_nodes;

        keyboard_focus(content, move |state: &mut State, action: KeyAction| {
            handle_key(state, action, &flat_for_keys, selected_index, user_handler.as_deref());
            MessageResult::Action(())
        })
    }

    fn build_rows(
        &self,
        flat_nodes: &[FlattenedNode<N::Id>],
        selected_index: Option<usize>,
    ) -> Vec<Box<AnyWidgetView<State, ()>>> {
        let mut rows: Vec<Box<AnyWidgetView<State, ()>>> = Vec::with_capacity(flat_nodes.len());
        self.build_rows_recursive(self.root, &mut rows, flat_nodes, selected_index);
        rows
    }

    fn build_rows_recursive(
        &self,
        node: &N,
        rows: &mut Vec<Box<AnyWidgetView<State, ()>>>,
        flat_nodes: &[FlattenedNode<N::Id>],
        selected_index: Option<usize>,
    ) {
        let row_index = rows.len();
        let Some(flat) = flat_nodes.get(row_index) else { return; };
        let node_id = node.id();

        // The recursion order MUST match the flatten order; this assertion
        // catches the case where a `TreeNode` impl changes children between
        // flatten and row-build calls.
        debug_assert!(flat.id == node_id, "tree iteration order diverged from flatten order");

        let depth = flat.depth;
        let is_expanded = flat.is_expanded;
        let is_expandable = flat.is_expandable;
        let is_selected = selected_index == Some(row_index);

        rows.push(self.build_single_row(node, depth, is_expanded, is_expandable, is_selected));

        if is_expanded {
            for child in node.children() {
                self.build_rows_recursive(child, rows, flat_nodes, selected_index);
            }
        }
    }

    fn build_single_row(
        &self,
        node: &N,
        depth: usize,
        is_expanded: bool,
        is_expandable: bool,
        is_selected: bool,
    ) -> Box<AnyWidgetView<State, ()>> {
        let text_color = self.text_color;
        let text_size = self.text_size;
        let chevron_color = self.chevron_color;
        let selected_bg = self.selected_bg;
        let hover_bg = self.style.hover_bg;

        // Icon + label content (the part right of the chevron).
        let icon_view: Box<AnyWidgetView<State, ()>> = match self.icon_for.as_ref() {
            Some(f) => f(node).unwrap_or_else(|| Box::new(label("").text_size(text_size))),
            None => Box::new(label("").text_size(text_size)),
        };

        let display = match self.label_for.as_ref() {
            Some(f) => f(node),
            None => node.label().to_string(),
        };

        let text_color_for_row = if is_selected {
            // Slightly brighter text on the selection background — keeps
            // readability without needing a separate selected_text_color knob.
            Color::from_rgb8(245, 244, 242)
        } else {
            text_color
        };
        let text = label(display).text_size(text_size).color(text_color_for_row);

        // Wrap icon+text in a row_button so clicks dispatch Select / DoubleClick /
        // ContextMenu. The chevron stays OUTSIDE this button so its clicks can
        // be handled separately by `disclosure_row`.
        let body = flex_row((icon_view, text)).gap(4.px());

        let node_id = node.id();
        let user_handler_for_press = self.handler.clone();
        let body_clickable = row_button_with_press(body, move |state: &mut State, press: &RowButtonPress| {
            if let Some(h) = user_handler_for_press.as_ref() {
                match press.button {
                    Some(PointerButton::Secondary) => {
                        h(state, &node_id, TreeAction::ContextMenu(press.position));
                    }
                    None | Some(PointerButton::Primary) => {
                        if press.click_count >= 2 {
                            h(state, &node_id, TreeAction::DoubleClick);
                        } else {
                            h(state, &node_id, TreeAction::Select);
                        }
                    }
                    _ => {}
                }
            }
        });

        // Selection background: applied to the ROW (chevron + body) so the
        // highlight covers the whole line, not just the body.
        let body_styled: Box<AnyWidgetView<State, ()>> = if is_selected {
            Box::new(body_clickable.background_color(selected_bg))
        } else if hover_bg != Color::TRANSPARENT {
            // Hover bg comes from style.hover_bg — applied unconditionally;
            // xilem's row_button paints hover internally.
            Box::new(body_clickable)
        } else {
            Box::new(body_clickable)
        };

        // Chevron handler dispatches TreeAction::Toggle.
        let node_id_for_toggle = node.id();
        let user_handler_for_toggle = self.handler.clone();
        let on_toggle = move |state: &mut State| {
            if let Some(h) = user_handler_for_toggle.as_ref() {
                h(state, &node_id_for_toggle, TreeAction::Toggle);
            }
        };

        // Indent + chevron (toggle button) + clickable body.
        disclosure_row(
            depth,
            is_expanded,
            is_expandable,
            chevron_color,
            body_styled,
            on_toggle,
        )
    }
}

/// Handle one keyboard navigation event by invoking the user's
/// `on_action` callback the same way a mouse click would.
///
/// Resolved against the latest flattened node list captured during `build()`.
/// Returning to `build()` after this fires a fresh flatten on the next rebuild.
fn handle_key<State, Id>(
    state: &mut State,
    action: KeyAction,
    flat_nodes: &[FlattenedNode<Id>],
    selected_index: Option<usize>,
    handler: Option<&(dyn Fn(&mut State, &Id, TreeAction) + Send + Sync)>,
) where
    Id: Clone,
{
    let Some(h) = handler else { return; };
    if flat_nodes.is_empty() {
        return;
    }

    let max_idx = flat_nodes.len() - 1;
    let cur = selected_index.unwrap_or(0);

    let select_at = |state: &mut State, idx: usize| {
        if let Some(node) = flat_nodes.get(idx) {
            h(state, &node.id, TreeAction::Select);
        }
    };

    match action {
        KeyAction::Up => {
            let next = cur.saturating_sub(1);
            select_at(state, next);
        }
        KeyAction::Down => {
            let next = (cur + 1).min(max_idx);
            select_at(state, next);
        }
        KeyAction::Home => {
            select_at(state, 0);
        }
        KeyAction::End => {
            select_at(state, max_idx);
        }
        KeyAction::PageUp => {
            let next = cur.saturating_sub(PAGE_SIZE);
            select_at(state, next);
        }
        KeyAction::PageDown => {
            let next = (cur + PAGE_SIZE).min(max_idx);
            select_at(state, next);
        }
        KeyAction::Left => {
            // If the focused node is expanded, collapse it. Otherwise jump
            // to its parent (matches macOS Finder / VS Code default).
            if let Some(node) = flat_nodes.get(cur) {
                if node.is_expanded && node.is_expandable {
                    h(state, &node.id, TreeAction::Toggle);
                } else if let Some(parent) = node.parent_index {
                    select_at(state, parent);
                }
            }
        }
        KeyAction::Right => {
            // If the focused node is expandable and collapsed, expand it.
            // If already expanded, move to the first child.
            if let Some(node) = flat_nodes.get(cur) {
                if node.is_expandable {
                    if !node.is_expanded {
                        h(state, &node.id, TreeAction::Toggle);
                    } else if cur + 1 <= max_idx {
                        select_at(state, cur + 1);
                    }
                }
            }
        }
        KeyAction::Toggle => {
            if let Some(node) = flat_nodes.get(cur) {
                if node.is_expandable {
                    h(state, &node.id, TreeAction::Toggle);
                }
            }
        }
        KeyAction::Activate => {
            if let Some(node) = flat_nodes.get(cur) {
                h(state, &node.id, TreeAction::DoubleClick);
            }
        }
        KeyAction::Edit => {
            if let Some(node) = flat_nodes.get(cur) {
                h(state, &node.id, TreeAction::StartEdit);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SingleSelection;

    #[derive(Clone, Debug)]
    struct Node {
        id: String,
        label: String,
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
            &self.label
        }
    }

    struct AppState {
        sel: SingleSelection<String>,
        exp: ExpansionState<String>,
    }

    #[test]
    fn builder_compiles() {
        let app = AppState {
            sel: SingleSelection::new(),
            exp: ExpansionState::new(),
        };
        let root = Node { id: "r".into(), label: "Root".into(), children: vec![] };
        let _ = tree_view::<Node, AppState>(&root, &app.exp)
            .selection(&app.sel)
            .selected_bg(Color::from_rgb8(50, 70, 100))
            .text_color(Color::WHITE)
            .text_size(12.0)
            .label_for(|n: &Node| format!("[{}] {}", n.children.len(), n.label))
            .on_action(|_state: &mut AppState, _id: &String, _action| {})
            .build();
    }
}
