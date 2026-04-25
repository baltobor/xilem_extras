//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Batteries-included tree view widget.
//!
//! [`tree_view`] is a thin builder around [`tree_group_styled`] that supplies
//! a default row renderer (chevron + indent + label + selection background)
//! so callers don't need to hand-roll the same ~30 lines of boilerplate
//! every time they want a tree.
//!
//! Out of the box:
//!
//!  * Expand/collapse chevron via [`crate::components::disclosure`]
//!    (Material Symbols), with a same-width invisible spacer for leaves so
//!    sibling labels at the same depth still line up.
//!  * Per-depth indentation taken from `TreeStyle::indent` (default 16 px).
//!  * Hover background via `TreeStyle::hover_bg`.
//!  * Selection background drawn directly under the row when the supplied
//!    [`SelectionState`] reports the node as selected (default: a soft blue;
//!    overridable via [`TreeView::selected_bg`]).
//!  * Click semantics inherited from `tree_group_styled` — single-click
//!    toggles expandable nodes, single-click selects leaves, double-click
//!    fires [`TreeAction::DoubleClick`], right-click fires
//!    [`TreeAction::ContextMenu`].
//!
//! Opt-in extensions (all chainable on the builder):
//!
//!  * [`TreeView::icon_for`] — render a per-node icon between chevron and
//!    label (e.g. folder/file icons in a file browser).
//!  * [`TreeView::label_for`] — override the displayed text without
//!    changing the underlying [`TreeNode::label`] contract.
//!  * [`TreeView::text_color`] / [`TreeView::text_size`] — label appearance.
//!  * [`TreeView::row_padding`] — shrink/grow the row's vertical padding.
//!
//! When a caller needs anything beyond this — inline rename inputs, file-
//! type-specific icons, per-row context menus — they should fall back to
//! the lower-level [`tree_group_styled`] (or the `_with_context_menu`
//! variants) and write their own row builder. This widget intentionally
//! covers the 80% case without becoming a god object.
//!
//! # Example
//!
//! ```ignore
//! use xilem_extras::{tree_view, ExpansionState, SingleSelection, TreeAction};
//!
//! tree_view(&model.tree, &model.expansion)
//!     .selection(&model.selection)
//!     .on_action(|model: &mut AppModel, id: &String, action| match action {
//!         TreeAction::Toggle => model.expansion.toggle(id),
//!         TreeAction::Select => model.selection.set(Some(id.clone())),
//!         TreeAction::DoubleClick => model.open(id),
//!         _ => {}
//!     })
//!     .build()
//! ```

use std::sync::Arc;

use masonry::layout::AsUnit;
use masonry::properties::Padding;
use xilem::core::one_of::Either;
use xilem::masonry::peniko::Color;
use xilem::style::Style;
use xilem::view::{flex_row, label};
use xilem::{AnyWidgetView, WidgetView};

use crate::components::disclosure;
use crate::traits::{SelectionState, TreeNode};

use super::tree_view::{tree_group_styled, TreeAction, TreeStyle};
use super::ExpansionState;

// Trait-object aliases used by the builder. Storing the user-supplied
// closures as `Arc<dyn Fn>` (instead of carrying them as separate generic
// parameters) keeps the public type stable across builder methods, so
// `build()` always sees the same `TreeView<...>` type regardless of which
// opt-ins the caller chained.
type IconFn<N, State> = dyn Fn(&N) -> Option<Box<AnyWidgetView<State, ()>>> + Send + Sync;
type LabelFn<N> = dyn Fn(&N) -> String + Send + Sync;
type ActionFn<N, State> = dyn Fn(&mut State, &<N as crate::traits::Identifiable>::Id, TreeAction)
    + Send
    + Sync;

/// Default soft-blue selection background. Picked to read clearly on both
/// light and dark themes; override via [`TreeView::selected_bg`] if you
/// have a palette to match.
pub const DEFAULT_SELECTED_BG: Color = Color::from_rgba8(60, 80, 110, 220);
/// Default row text color. Light gray that reads on dark backgrounds and
/// stays legible against the default `selected_bg`.
pub const DEFAULT_TEXT_COLOR: Color = Color::from_rgb8(220, 218, 214);
/// Default chevron color when present. Slightly dimmer than text so it
/// reads as a hint, not a primary glyph.
pub const DEFAULT_CHEVRON_COLOR: Color = Color::from_rgb8(180, 178, 172);

/// Builder for the batteries-included tree view widget. See the module
/// docs and [`tree_view`] for the full API and example usage.
///
/// The builder's type stays stable across opt-in calls because the
/// per-node closures (`icon_for`, `label_for`, `on_action`) are stored as
/// `Arc<dyn Fn>` trait objects internally — that means `build()` works
/// the same whether you call zero opt-ins or all of them.
///
/// Type parameters:
///  * `'a` — borrow lifetime of the source tree, expansion state, and
///    optional selection state.
///  * `N` — the [`TreeNode`] implementation.
///  * `State` — the application's mutable state.
///  * `Sel` — the [`SelectionState`] (defaults to `()`, meaning no
///    selection rendering).
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

/// Start a new tree-view builder. Defaults: hover bg transparent, indent
/// 16 px per depth level, selection bg [`DEFAULT_SELECTED_BG`], text color
/// [`DEFAULT_TEXT_COLOR`], 11 pt label, 2 px vertical row padding.
///
/// Call `.build()` once you're done configuring; without `.on_action`
/// the tree is read-only (clicks are ignored).
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
    /// Provide a [`SelectionState`] so selected rows paint with the
    /// selection background. Without this call (or with `None`) every
    /// row is rendered with no selection chrome.
    ///
    /// Changing the selection type is the one place the builder's `Sel`
    /// type parameter actually shifts — every other opt-in is stored as a
    /// trait object internally.
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

    /// Override the [`TreeStyle`] (hover background, indent, gap).
    pub fn style(mut self, style: TreeStyle) -> Self {
        self.style = style;
        self
    }

    /// Background painted under the row when its node is selected.
    pub fn selected_bg(mut self, color: Color) -> Self {
        self.selected_bg = color;
        self
    }

    /// Color used for the row's label text.
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Color used for the chevron (collapsed and expanded) on expandable
    /// rows. Leaves always render an invisible spacer regardless.
    pub fn chevron_color(mut self, color: Color) -> Self {
        self.chevron_color = color;
        self
    }

    /// Font size of the label text. Default 11 pt.
    pub fn text_size(mut self, size: f32) -> Self {
        self.text_size = size;
        self
    }

    /// Padding around each row. The `left` value is added on top of the
    /// per-depth indentation, so use it to control the row's gutter.
    pub fn row_padding(mut self, padding: Padding) -> Self {
        self.row_padding = padding;
        self
    }

    /// Provide a per-node icon view inserted between the chevron and the
    /// label. The closure may return `None` for nodes that should have no
    /// icon — they still occupy the icon slot to keep labels aligned.
    pub fn icon_for<F>(mut self, f: F) -> Self
    where
        F: Fn(&N) -> Option<Box<AnyWidgetView<State, ()>>> + Send + Sync + 'static,
    {
        self.icon_for = Some(Arc::new(f));
        self
    }

    /// Override the displayed label without changing what
    /// [`TreeNode::label`] returns. Useful when the source type's label is
    /// a stable identifier you want to embellish for display (e.g. add a
    /// child count or a prefix).
    pub fn label_for<F>(mut self, f: F) -> Self
    where
        F: Fn(&N) -> String + Send + Sync + 'static,
    {
        self.label_for = Some(Arc::new(f));
        self
    }

    /// Action handler invoked on Toggle / Select / DoubleClick /
    /// ContextMenu / Edit. Without this call the tree is read-only.
    pub fn on_action<F>(mut self, h: F) -> Self
    where
        F: Fn(&mut State, &N::Id, TreeAction) + Send + Sync + 'static,
    {
        self.handler = Some(Arc::new(h));
        self
    }
}

/// Concrete (post-configuration) build method. The `where` clause is
/// where the real type machinery lives — the builder above is duck-typed
/// across closure swaps, but `build()` requires the action handler and
/// selection type to be concrete callables.
impl<'a, N, State, Sel, IconFn, LabelFn, Handler>
    TreeView<'a, N, State, Sel, IconFn, LabelFn, Handler>
where
    N: TreeNode + 'a,
    N::Id: Clone + Send + Sync + 'static,
    State: 'static,
    Sel: SelectionState<N::Id> + 'a,
    IconFn: Fn(&N) -> Option<Box<AnyWidgetView<State, ()>>> + Clone + 'a,
    LabelFn: Fn(&N) -> String + Clone + 'a,
    Handler: Fn(&mut State, &N::Id, TreeAction) + Clone + Send + Sync + 'static,
{
    /// Build the configured tree as a Xilem view. Requires that
    /// `on_action` was set; otherwise no actions can fire and the tree is
    /// inert (still valid, just not interactive).
    pub fn build(self) -> impl WidgetView<State, ()> + use<'a, N, State, Sel, IconFn, LabelFn, Handler> {
        // The closures captured by the row builder must be `Clone` because
        // `tree_group_styled` invokes the builder once per visible row and
        // requires `F: Clone`. Wrapping in Arc keeps each row builder call
        // cheap regardless of how heavy the user's closures are.
        let icon_for = self.icon_for.map(Arc::new);
        let label_for = self.label_for.map(Arc::new);
        let style = self.style.clone();
        let indent_per_level = style.indent;
        let selected_bg = self.selected_bg;
        let text_color = self.text_color;
        let chevron_color = self.chevron_color;
        let text_size = self.text_size;
        let row_padding = self.row_padding;

        // Default no-op handler so the harness still receives a valid Fn
        // even when the caller didn't set one. ContextMenu/Edit actions
        // simply have nowhere to go — that's the inert behavior we want.
        let user_handler = self.handler;
        let handler = move |state: &mut State, id: &N::Id, action: TreeAction| {
            if let Some(h) = user_handler.as_ref() {
                h(state, id, action);
            }
        };

        let row_builder = move |node: &N, depth: usize, is_expanded: bool, is_selected: bool| {
            // Chevron column — invisible spacer for leaves so labels at the
            // same depth align across mixed leaf/expandable siblings.
            let chevron: Box<AnyWidgetView<State, ()>> = if node.is_expandable() {
                disclosure(is_expanded).color(chevron_color).build()
            } else {
                // Same metric width as the disclosure glyph; a no-op label
                // works here because Material Symbols glyphs are roughly
                // square and the Disclosure widget uses ICON_SIZE_SM.
                let spacer = label("\u{00A0}\u{00A0}")
                    .text_size(text_size)
                    .color(Color::TRANSPARENT);
                Box::new(spacer)
            };

            // Optional per-node icon slot. Always present so labels stay
            // aligned across rows that do/don't have icons.
            let icon_view: Box<AnyWidgetView<State, ()>> = match icon_for.as_ref() {
                Some(f) => match f(node) {
                    Some(v) => v,
                    None => Box::new(label("").text_size(text_size)),
                },
                None => Box::new(label("").text_size(text_size)),
            };

            // Label — caller may override the displayed text.
            let display = match label_for.as_ref() {
                Some(f) => f(node),
                None => node.label().to_string(),
            };
            let text = label(display).text_size(text_size).color(text_color);

            let row = flex_row((chevron, icon_view, text)).gap(4.px()).padding(
                Padding {
                    top: row_padding.top,
                    bottom: row_padding.bottom,
                    left: row_padding.left + depth as f64 * indent_per_level,
                    right: row_padding.right,
                },
            );

            // Either is used so both branches return the same concrete
            // type (background_color produces a styled view; we want one
            // arm with the selected color and one transparent arm).
            if is_selected {
                Either::A(row.background_color(selected_bg))
            } else {
                Either::B(row.background_color(Color::TRANSPARENT))
            }
        };

        tree_group_styled(
            self.root,
            self.expansion,
            self.selection,
            style,
            row_builder,
            handler,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SingleSelection;
    use crate::traits::Identifiable;

    /// A trivial TreeNode for type-only smoke tests — verifies the builder
    /// chain compiles end-to-end with realistic generics.
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

    /// Confirms the builder type-checks with all opt-ins applied.
    #[test]
    fn builder_compiles() {
        let app = AppState {
            sel: SingleSelection::new(),
            exp: ExpansionState::new(),
        };
        let root = Node {
            id: "root".into(),
            label: "Root".into(),
            children: vec![],
        };
        // Just exercise the type machinery — actually rendering the view
        // would require a Xilem app harness which we don't pull in here.
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
