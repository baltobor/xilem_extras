//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Tree widget demo.
//! HINT: This is just a sketch. Ti will be moved into the lib later.

use masonry::layout::AsUnit;
use xilem::masonry::properties::Padding;
use xilem::masonry::vello::peniko::Color;
use xilem::style::Style;
use xilem::view::{CrossAxisAlignment, flex_col, flex_row, label, portal};
use xilem::WidgetView;
use xilem::AnyWidgetView;

use xilem_extras::{ExpansionState, row_button};
use xilem_extras::components::icon::{icons, MATERIAL_SYMBOLS_FAMILY, ICON_SIZE_SM};

use crate::app_model::AppModel;
use crate::mock_data::FileNode;

const TEXT_COLOR: Color = Color::from_rgb8(220, 218, 214);
const FOLDER_COLOR: Color = Color::from_rgb8(220, 180, 80);
const BG_HOVER: Color = Color::from_rgb8(55, 53, 50);
const BG_SELECTED: Color = Color::from_rgb8(65, 62, 58);

fn render_tree_node(
    node: &FileNode,
    depth: usize,
    expansion: &ExpansionState<String>,
    selected: Option<&String>,
) -> Box<AnyWidgetView<AppModel>> {
    let indent = (depth * 16) as f64;
    let is_expanded = expansion.is_expanded(&node.path);
    let is_selected = selected == Some(&node.path);
    let row_bg = if is_selected { BG_SELECTED } else { Color::TRANSPARENT };

    let path = node.path.clone();

    if node.is_dir {
        // Directory: clickable to expand/collapse
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

        let row = flex_row((
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
        .padding(Padding::left(indent));

        let path_for_click = path.clone();
        let btn = row_button(row, move |model: &mut AppModel| {
            model.toggle_tree_node(&path_for_click);
        })
        .hover_bg(BG_HOVER)
        .background_color(row_bg);

        if is_expanded {
            let children: Vec<Box<AnyWidgetView<AppModel>>> = node
                .children
                .iter()
                .map(|child| render_tree_node(child, depth + 1, expansion, selected))
                .collect();

            flex_col((btn, children))
                .gap(0.px())
                .boxed()
        } else {
            btn.boxed()
        }
    } else {
        // File: clickable to select
        let file_indent = indent + 20.0; // Extra indent for no chevron

        let row = flex_row((
            label(icons::DESCRIPTION.to_string())
                .font(MATERIAL_SYMBOLS_FAMILY)
                .text_size(ICON_SIZE_SM)
                .color(TEXT_COLOR),
            label(node.name.clone())
                .text_size(13.0)
                .color(TEXT_COLOR),
        ))
        .gap(4.px())
        .padding(Padding::left(file_indent));

        row_button(row, move |model: &mut AppModel| {
            model.select_tree_node(path.clone());
        })
        .hover_bg(BG_HOVER)
        .background_color(row_bg)
        .boxed()
    }
}

pub fn tree_demo(model: &mut AppModel) -> impl WidgetView<AppModel> + use<> {
    let selected = model.tree_selection.selected();

    let tree_items: Vec<Box<AnyWidgetView<AppModel>>> = model
        .file_tree
        .children
        .iter()
        .map(|node| render_tree_node(node, 0, &model.tree_expansion, selected))
        .collect();

    flex_col((
        label("Tree Demo")
            .text_size(16.0)
            .weight(xilem::FontWeight::BOLD)
            .color(TEXT_COLOR),
        label("Click folders to expand/collapse, files to select")
            .text_size(12.0)
            .color(Color::from_rgb8(160, 156, 150)),
        portal(
            flex_col(tree_items)
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .gap(0.px())
                .padding(8.0),
        ),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Start)
    .gap(8.px())
    .padding(16.0)
}
