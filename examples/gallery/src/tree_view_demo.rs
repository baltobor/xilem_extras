//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Batteries-included tree view demo. Shows how `tree_view` collapses the
//! ~100-line manual row builder from `tree_demo.rs` into a handful of
//! configuration calls.
//!
//! Two trees are displayed side-by-side:
//!
//!  * **Default**: zero opt-ins beyond `selection` + `on_action`. The
//!    builder ships with default chevrons, indentation, hover bg, and a
//!    soft-blue selection background.
//!  * **Customized**: the same data with `icon_for` (folder vs. document
//!    icons), `text_color`, `selected_bg`, and a custom `TreeStyle` for a
//!    warmer hover background.

use masonry::layout::AsUnit;
use xilem::masonry::peniko::Color;
use xilem::style::Style;
use xilem::view::{flex_col, flex_row, label, portal, CrossAxisAlignment};
use xilem::{AnyWidgetView, WidgetView};
use xilem_extras::{tree_view, TreeAction, TreeStyle};
use xilem_material_icons::{icons, FONT_FAMILY, ICON_SIZE_SM};

use crate::app_model::AppModel;
use crate::mock_data::FileNode;

const HEADER_FG: Color = Color::from_rgb8(220, 218, 214);
const SUBTLE_FG: Color = Color::from_rgb8(160, 156, 150);
const HOVER_BG: Color = Color::from_rgb8(55, 53, 50);
const FOLDER_FG: Color = Color::from_rgb8(220, 180, 80);

pub fn tree_view_demo(model: &mut AppModel) -> impl WidgetView<AppModel, ()> + use<'_> {
    // Default tree: just selection + handler. Everything else picks up the
    // builder's defaults.
    let default_tree = tree_view(&model.file_tree, &model.tree_expansion)
        .selection(&model.tree_selection)
        .style(TreeStyle::new().hover_bg(HOVER_BG))
        .on_action(default_handler)
        .build();

    // Customized tree: same data, opt into per-node icons + a different
    // selection background. Demonstrates that the builder type is stable
    // regardless of which opt-ins you chain.
    let customized_tree = tree_view(&model.file_tree, &model.tree_expansion)
        .selection(&model.tree_selection)
        .style(TreeStyle::new().hover_bg(HOVER_BG).indent(18.0))
        .selected_bg(Color::from_rgba8(80, 110, 60, 220))
        .text_color(HEADER_FG)
        .text_size(13.0)
        .icon_for(node_icon)
        .label_for(|n: &FileNode| {
            // Annotate directories with a child count.
            if n.is_dir {
                format!("{} ({})", n.name, n.children.len())
            } else {
                n.name.clone()
            }
        })
        .on_action(default_handler)
        .build();

    let column = |title: &'static str, body| {
        flex_col((
            label(title)
                .text_size(13.0)
                .weight(xilem::FontWeight::BOLD)
                .color(HEADER_FG),
            portal(
                flex_col((body,))
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .gap(0.px())
                    .padding(8.0),
            ),
        ))
        .gap(4.px())
    };

    flex_col((
        flex_row((
            label("Tree View Demo")
                .text_size(15.0)
                .weight(xilem::FontWeight::BOLD)
                .color(HEADER_FG),
            label(format!(
                "selection: {}",
                model
                    .tree_selection
                    .selected()
                    .map(|s| s.as_str())
                    .unwrap_or("(none)")
            ))
            .text_size(12.0)
            .color(SUBTLE_FG),
        ))
        .gap(12.px()),
        label("`tree_view` is a builder over `tree_group_styled` that supplies\n\
               default chevron, indent, hover bg, and selection chrome — most\n\
               callers should reach for it before writing a row builder by hand.")
            .text_size(11.0)
            .color(SUBTLE_FG),
        flex_row((column("Default", default_tree), column("Customized", customized_tree)))
            .gap(24.px()),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Start)
    .gap(8.px())
    .padding(16.0)
}

fn default_handler(model: &mut AppModel, id: &String, action: TreeAction) {
    match action {
        TreeAction::Toggle => model.toggle_tree_node(id),
        TreeAction::Select => model.select_tree_node(id.clone()),
        TreeAction::DoubleClick => model.select_tree_node(id.clone()),
        _ => {}
    }
}

/// Per-node icon for the customized tree. Folders get a folder glyph that
/// flips on expansion; files get a generic document icon.
fn node_icon(node: &FileNode) -> Option<Box<AnyWidgetView<AppModel, ()>>> {
    let (glyph, color) = if node.is_dir {
        // tree_view doesn't pass `is_expanded` to icon_for (yet); we use
        // the closed icon since the chevron already encodes state.
        (icons::FOLDER, FOLDER_FG)
    } else {
        (icons::DESCRIPTION, HEADER_FG)
    };
    Some(Box::new(
        label(glyph)
            .font(FONT_FAMILY)
            .text_size(ICON_SIZE_SM)
            .color(color),
    ))
}
