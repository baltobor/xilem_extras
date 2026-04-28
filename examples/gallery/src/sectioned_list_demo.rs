//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Sectioned list demo - demonstrates grouped lists with section headers.

use masonry::layout::AsUnit;
use xilem::masonry::peniko::Color;
use xilem::style::Style;
use xilem::view::{CrossAxisAlignment, flex_col, flex_row, label, button};
use xilem::{FontWeight, WidgetView};

use xilem_extras::{list_view_sectioned, ListViewAction, ListViewStyle, SectionDef, SectionedRowInfo, SelectionState};
use xilem_material_icons::{icons, FONT_FAMILY, ICON_SIZE_SM};

use crate::app_model::AppModel;

const TEXT_COLOR: Color = Color::from_rgb8(220, 218, 214);
const TEXT_SECONDARY: Color = Color::from_rgb8(160, 156, 150);
const BG_HEADER: Color = Color::from_rgb8(35, 33, 30);
const BG_SELECTED: Color = Color::from_rgb8(65, 62, 58);
const BG_STRIPE: Color = Color::from_rgb8(42, 40, 38);
const ICON_STAR: Color = Color::from_rgb8(255, 200, 50);
const ICON_RECENT: Color = Color::from_rgb8(100, 150, 255);
const ICON_CONTACT: Color = Color::from_rgb8(100, 180, 100);

fn section_header(title: String) -> impl WidgetView<AppModel, ()> {
    flex_row((
        label(title)
            .text_size(12.0)
            .weight(FontWeight::BOLD)
            .color(TEXT_SECONDARY),
    ))
    .padding(8.0)
    .background_color(BG_HEADER)
}

fn contact_item(
    name: String,
    email: String,
    icon: &str,
    icon_color: Color,
    is_selected: bool,
    is_striped: bool,
) -> impl WidgetView<AppModel, ()> {
    let row_bg = if is_selected {
        BG_SELECTED
    } else if is_striped {
        BG_STRIPE
    } else {
        Color::TRANSPARENT
    };

    flex_row((
        label(icon.to_string())
            .font(FONT_FAMILY)
            .text_size(ICON_SIZE_SM)
            .color(icon_color)
            .width(24.px()),
        flex_col((
            label(name)
                .text_size(13.0)
                .color(TEXT_COLOR),
            label(email)
                .text_size(11.0)
                .color(TEXT_SECONDARY),
        ))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .gap(2.px()),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Center)
    .gap(8.px())
    .padding(8.0)
    .background_color(row_bg)
}

pub fn sectioned_list_demo(model: &mut AppModel) -> impl WidgetView<AppModel, ()> + use<'_> {
    // Create sections from contacts
    // For demo: first 3 are "Favorites", next 5 are "Recent", rest are "All"
    let favorites = if model.contacts.len() >= 3 {
        &model.contacts[0..3]
    } else {
        &model.contacts[..]
    };

    let recent_start = favorites.len().min(model.contacts.len());
    let recent_end = (recent_start + 5).min(model.contacts.len());
    let recent = &model.contacts[recent_start..recent_end];

    let all_start = recent_end;
    let all = if all_start < model.contacts.len() {
        &model.contacts[all_start..]
    } else {
        &model.contacts[0..0]
    };

    let sections = [
        SectionDef::new("Favorites", favorites),
        SectionDef::new("Recent", recent),
        SectionDef::new("All Contacts", all),
    ];

    // Update selection item order
    let contact_ids: Vec<u64> = model.contacts.iter().map(|c| c.id).collect();
    model.sectioned_list_selection.set_items(contact_ids);

    let list_view = list_view_sectioned(
        &sections,
        &model.sectioned_list_selection,
        ListViewStyle::new()
            .row_height(52.0)
            .selected_bg(BG_SELECTED)
            .striped(true)
            .stripe_bg(BG_STRIPE),
        |state: &mut AppModel, row_info| {
            match row_info {
                SectionedRowInfo::Header { title, .. } => {
                    section_header(title).boxed()
                }
                SectionedRowInfo::Item {
                    section_index,
                    global_item_index,
                    is_selected,
                    is_striped,
                    ..
                } => {
                    let contact = &state.contacts[global_item_index];
                    let (icon, icon_color) = match section_index {
                        0 => (icons::STAR, ICON_STAR),
                        1 => (icons::SCHEDULE, ICON_RECENT),
                        _ => (icons::PERSON, ICON_CONTACT),
                    };
                    contact_item(
                        contact.name.clone(),
                        contact.email.clone(),
                        icon,
                        icon_color,
                        is_selected,
                        is_striped,
                    ).boxed()
                }
            }
        },
        |state: &mut AppModel, action| {
            match action {
                ListViewAction::Select(id, mods) => state.sectioned_list_selection.select(id, mods),
                ListViewAction::Activate(_id) => {
                    // Double-click / Enter action
                }
            }
        },
    );

    flex_col((
        label("Sectioned Contacts")
            .text_size(16.0)
            .weight(FontWeight::BOLD)
            .color(TEXT_COLOR),
        label("Grouped list with section headers, keyboard navigation")
            .text_size(12.0)
            .color(TEXT_SECONDARY),

        list_view,

        // Selection info
        flex_row((
            label(format!("Selected: {} contacts", model.sectioned_list_selection.count()))
                .text_size(12.0)
                .color(TEXT_SECONDARY),
            button(label("Clear"), |model: &mut AppModel| {
                model.sectioned_list_selection.clear();
            }),
        ))
        .gap(16.px()),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Start)
    .gap(8.px())
    .padding(16.0)
}
