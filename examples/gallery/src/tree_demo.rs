//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Tree widget demo using tree_group.

use masonry::layout::AsUnit;
use xilem::masonry::properties::Padding;
use xilem::masonry::vello::peniko::Color;
use xilem::style::Style;
use xilem::view::{CrossAxisAlignment, flex_col, flex_row, label, portal};
use xilem::{WidgetView, AnyWidgetView};

use xilem_extras::{
    tree_group_with_context_menu, TreeAction, TreeStyle, svg_icon, rust_gear, ferris,
    menu_item, separator,
};
use xilem_extras::components::icon::{icons, MATERIAL_SYMBOLS_FAMILY, ICON_SIZE_SM};

use crate::app_model::AppModel;
use crate::mock_data::FileNode;

const TEXT_COLOR: Color = Color::from_rgb8(220, 218, 214);
const FOLDER_COLOR: Color = Color::from_rgb8(220, 180, 80);
const RUST_COLOR: Color = Color::from_rgb8(247, 76, 0);
const BG_SELECTED: Color = Color::from_rgb8(65, 62, 58);

fn build_tree_row(
    node: &FileNode,
    depth: usize,
    is_expanded: bool,
    is_selected: bool,
) -> Box<AnyWidgetView<AppModel, ()>> {
    let indent = (depth * 16) as f64;
    let row_bg = if is_selected { BG_SELECTED } else { Color::TRANSPARENT };

    if node.is_dir {
        // Directory row with chevron and folder icon
        let chevron = if is_expanded {
            icons::EXPAND_MORE
        } else {
            icons::CHEVRON_RIGHT
        };

        let folder_icon = if is_expanded {
            icons::FOLDER_OPEN
        } else {
            icons::FOLDER
        };

        flex_row((
            label(chevron.to_string())
                .font(MATERIAL_SYMBOLS_FAMILY)
                .text_size(ICON_SIZE_SM)
                .color(TEXT_COLOR),
            label(folder_icon.to_string())
                .font(MATERIAL_SYMBOLS_FAMILY)
                .text_size(ICON_SIZE_SM)
                .color(FOLDER_COLOR),
            label(node.name.clone())
                .text_size(15.0)
                .color(TEXT_COLOR),
        ))
        .gap(4.px())
        .padding(Padding::left(indent))
        .background_color(row_bg)
        .boxed()
    } else {
        // File row (extra indent for no chevron)
        let file_indent = indent + 20.0;

        if node.name.ends_with(".rs") {
            // Rust file with gear icon (13px to match text size)
            flex_row((
                svg_icon(rust_gear().size(13.0).color(RUST_COLOR)),
                label(node.name.clone())
                    .text_size(15.0)
                    .color(TEXT_COLOR),
            ))
            .gap(4.px())
            .padding(Padding::left(file_indent))
            .background_color(row_bg)
            .boxed()
        } else if node.name == "Cargo.toml" {
            // Cargo.toml with Ferris crab (13px to match text size)
            flex_row((
                svg_icon(ferris().size(13.0).color(RUST_COLOR)),
                label(node.name.clone())
                    .text_size(15.0)
                    .color(TEXT_COLOR),
            ))
            .gap(4.px())
            .padding(Padding::left(file_indent))
            .background_color(row_bg)
            .boxed()
        } else {
            // Other files with document icon
            flex_row((
                label(icons::DESCRIPTION.to_string())
                    .font(MATERIAL_SYMBOLS_FAMILY)
                    .text_size(ICON_SIZE_SM)
                    .color(TEXT_COLOR),
                label(node.name.clone())
                    .text_size(15.0)
                    .color(TEXT_COLOR),
            ))
            .gap(4.px())
            .padding(Padding::left(file_indent))
            .background_color(row_bg)
            .boxed()
        }
    }
}

const BG_HOVER: Color = Color::from_rgb8(55, 53, 50);

pub fn tree_demo(model: &mut AppModel) -> impl WidgetView<AppModel, ()> + use<'_> {
    // Using the new type-safe context menu API
    // Each menu item carries its own action - no index matching needed!
    let tree_view = tree_group_with_context_menu(
        &model.file_tree,
        &model.tree_expansion,
        Some(&model.tree_selection),
        TreeStyle::new().hover_bg(BG_HOVER),
        |node_id: &String| {
            // Clone the node_id for use in closures
            let id = node_id.clone();
            let id2 = node_id.clone();
            let id3 = node_id.clone();
            let id4 = node_id.clone();
            (
                menu_item("Open", move |state: &mut AppModel| {
                    state.menu_last_action = format!("Open: {}", id);
                }),
                menu_item("Delete", move |state: &mut AppModel| {
                    state.menu_last_action = format!("Delete: {}", id2);
                }),
                separator(),
                menu_item("Rename", move |state: &mut AppModel| {
                    state.menu_last_action = format!("Rename: {}", id3);
                }),
                menu_item("Properties", move |state: &mut AppModel| {
                    state.menu_last_action = format!("Properties: {}", id4);
                }),
            )
        },
        |node: &FileNode, depth, is_expanded, is_selected| {
            build_tree_row(node, depth, is_expanded, is_selected)
        },
        |state: &mut AppModel, node_id: &String, action| {
            match action {
                TreeAction::Toggle => state.toggle_tree_node(node_id),
                TreeAction::Select => state.select_tree_node(node_id.clone()),
                TreeAction::DoubleClick => {
                    state.select_tree_node(node_id.clone());
                }
                TreeAction::ContextMenu(_) => {
                    // Not used with tree_group_with_context_menu
                }
            }
        },
    );

    flex_col((
        label("Tree Demo")
            .text_size(15.0)
            .weight(xilem::FontWeight::BOLD)
            .color(TEXT_COLOR),
        label("Right-click items for context menu")
            .text_size(12.0)
            .color(Color::from_rgb8(160, 156, 150)),
        label(format!("Last action: {}", model.menu_last_action))
            .text_size(12.0)
            .color(Color::from_rgb8(160, 156, 150)),
        portal(
            flex_col((tree_view,))
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .gap(0.px())
                .padding(8.0),
        ),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Start)
    .gap(8.px())
    .padding(16.0)
}
