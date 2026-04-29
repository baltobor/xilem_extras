//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Tree view demo using the canonical `tree_view` API.
//!
//! Demonstrates: chevron-only toggle, keyboard navigation (arrow keys, space,
//! enter, F2), per-node icons via the Material Icons font, single selection.
//! Wrapped in a `portal` for scrolling.

use masonry::layout::AsUnit;
use xilem::masonry::peniko::Color;
use xilem::style::Style;
use xilem::view::{flex_col, label, CrossAxisAlignment};
use xilem::{AnyWidgetView, WidgetView};
use xilem_extras::{
    menu_item, separator, tree_view, BoxedMenuEntry, Identifiable, MenuItems, TreeAction, TreeStyle,
};
use xilem_material_icons::{icons, FONT_FAMILY, ICON_SIZE_SM};

use crate::app_model::AppModel;
use crate::mock_data::FileNode;

const HEADER_FG: Color = Color::from_rgb8(220, 218, 214);
const SUBTLE_FG: Color = Color::from_rgb8(160, 156, 150);
const HOVER_BG: Color = Color::from_rgb8(55, 53, 50);
const FOLDER_FG: Color = Color::from_rgb8(220, 180, 80);

pub fn tree_view_demo(model: &mut AppModel) -> impl WidgetView<AppModel, ()> + use<'_> {
    let tree = tree_view(&model.file_tree, &model.tree_expansion)
        .selection(&model.tree_selection)
        .style(TreeStyle::new().hover_bg(HOVER_BG).indent(18.0))
        .selected_bg(Color::from_rgba8(80, 110, 60, 220))
        .text_color(HEADER_FG)
        .text_size(13.0)
        .icon_for(|node: &FileNode| -> Option<Box<AnyWidgetView<AppModel, ()>>> {
            let (icon, color) = if node.is_dir {
                (icons::FOLDER, FOLDER_FG)
            } else {
                (icons::DESCRIPTION, HEADER_FG)
            };
            Some(Box::new(
                label(icon)
                    .font(FONT_FAMILY)
                    .text_size(ICON_SIZE_SM)
                    .color(color),
            ))
        })
        .label_for(|n: &FileNode| {
            if n.is_dir {
                format!("{} ({})", n.name, n.children.len())
            } else {
                n.name.clone()
            }
        })
        .context_menu_for(|node: &FileNode| -> Vec<BoxedMenuEntry<AppModel, ()>> {
            // Each closure captures `id` by move so the menu item knows which
            // node it is operating on without needing extra plumbing.
            let id_open = node.id();
            let id_rename = node.id();
            let id_delete = node.id();
            (
                menu_item("Open", move |m: &mut AppModel| {
                    m.tree_activated = Some(id_open.clone());
                }),
                menu_item("Rename", move |m: &mut AppModel| {
                    m.start_editing_tree_node(&id_rename);
                }),
                separator(),
                menu_item("Delete", move |m: &mut AppModel| {
                    m.delete_tree_node(&id_delete);
                }),
            )
                .collect_entries()
        })
        .editing(
            model.tree_editing.as_ref(),
            &model.tree_editing_text,
            |state: &mut AppModel, new: String| state.tree_editing_text = new,
        )
        .on_action(default_handler)
        .build();

    flex_col((
        label("Tree View — Keyboard Navigation")
            .text_size(15.0)
            .weight(xilem::FontWeight::BOLD)
            .color(HEADER_FG),
        label(format!(
            "selection: {}    activated: {}",
            model
                .tree_selection
                .selected()
                .map(|s| s.as_str())
                .unwrap_or("(none)"),
            model
                .tree_activated
                .as_deref()
                .unwrap_or("(none)"),
        ))
        .text_size(12.0)
        .color(SUBTLE_FG),
        label("Click chevron to toggle. Click row to select. Double-click or Enter to activate. Up/Down/Left/Right/Space/Enter for keyboard nav.")
            .text_size(11.0)
            .color(SUBTLE_FG),
        // Scrolling lives inside `tree_view().build()` now (it wraps the
        // content in `scroll_focus`, which is a portal that auto-scrolls
        // to the selected row).
        flex_col((tree.boxed(),))
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .gap(0.px())
            .padding(8.0),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Start)
    .gap(8.px())
    .padding(16.0)
}

fn default_handler(model: &mut AppModel, id: &String, action: TreeAction) {
    match action {
        TreeAction::Toggle => model.toggle_tree_node(id),
        TreeAction::Select => model.select_tree_node(id.clone()),
        TreeAction::DoubleClick => {
            // Activate = Enter or double-click. Record it separately so the
            // demo can show that the gesture was actually processed.
            model.select_tree_node(id.clone());
            model.tree_activated = Some(id.clone());
        }
        TreeAction::StartEdit => model.start_editing_tree_node(id),
        TreeAction::CommitEdit(text) => {
            // Only commit if we actually have an active edit on this id;
            // otherwise it's a stray Enter from somewhere else.
            if model.tree_editing.as_deref() == Some(id.as_str()) {
                model.rename_tree_node(id, text);
            }
        }
        TreeAction::CancelEdit => model.cancel_editing_tree_node(),
        _ => {}
    }
}
