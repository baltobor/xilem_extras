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

use xilem_extras::{tree_group, TreeAction};
use xilem_extras::components::icon::{icons, MATERIAL_SYMBOLS_FAMILY, ICON_SIZE_SM};

use crate::app_model::AppModel;
use crate::mock_data::FileNode;

const TEXT_COLOR: Color = Color::from_rgb8(220, 218, 214);
const FOLDER_COLOR: Color = Color::from_rgb8(220, 180, 80);
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
                .text_size(13.0)
                .color(TEXT_COLOR),
        ))
        .gap(4.px())
        .padding(Padding::left(indent))
        .background_color(row_bg)
        .boxed()
    } else {
        // File row with document icon (extra indent for no chevron)
        let file_indent = indent + 20.0;

        flex_row((
            label(icons::DESCRIPTION.to_string())
                .font(MATERIAL_SYMBOLS_FAMILY)
                .text_size(ICON_SIZE_SM)
                .color(TEXT_COLOR),
            label(node.name.clone())
                .text_size(13.0)
                .color(TEXT_COLOR),
        ))
        .gap(4.px())
        .padding(Padding::left(file_indent))
        .background_color(row_bg)
        .boxed()
    }
}

pub fn tree_demo(model: &mut AppModel) -> impl WidgetView<AppModel, ()> + use<'_> {
    let tree_view = tree_group(
        &model.file_tree,
        &model.tree_expansion,
        Some(&model.tree_selection),
        |node: &FileNode, depth, is_expanded, is_selected| {
            build_tree_row(node, depth, is_expanded, is_selected)
        },
        |state: &mut AppModel, node_id: &String, action| {
            match action {
                TreeAction::Toggle => state.toggle_tree_node(node_id),
                TreeAction::Select => state.select_tree_node(node_id.clone()),
                TreeAction::DoubleClick => {
                    // Could open file here
                    state.select_tree_node(node_id.clone());
                }
            }
        },
    );

    flex_col((
        label("Tree Demo")
            .text_size(16.0)
            .weight(xilem::FontWeight::BOLD)
            .color(TEXT_COLOR),
        label("Click folders to expand/collapse, files to select")
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
