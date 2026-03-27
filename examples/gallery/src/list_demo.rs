//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! List widget demo - demonstrates the list view with multi-selection.

use masonry::layout::AsUnit;
use xilem::masonry::vello::peniko::Color;
use xilem::style::Style;
use xilem::view::{CrossAxisAlignment, flex_col, flex_row, label, button, portal};
use xilem::{AnyWidgetView, WidgetView};

use xilem_extras::{list_styled, ListAction, ListStyle, SelectionState};
use xilem_extras::components::icon::{MATERIAL_SYMBOLS_FAMILY, ICON_SIZE_SM};
use xilem_material_icons::icons;

use crate::app_model::AppModel;
use crate::mock_data::Contact;

const TEXT_COLOR: Color = Color::from_rgb8(220, 218, 214);
const TEXT_SECONDARY: Color = Color::from_rgb8(160, 156, 150);
const BG_HOVER: Color = Color::from_rgb8(55, 53, 50);
const BG_SELECTED: Color = Color::from_rgb8(65, 62, 58);
const ICON_COLOR: Color = Color::from_rgb8(100, 180, 100);

fn contact_row(contact: &Contact, is_selected: bool) -> Box<AnyWidgetView<AppModel, ()>> {
    let row_bg = if is_selected { BG_SELECTED } else { Color::TRANSPARENT };
    let name = contact.name.clone();
    let email = contact.email.clone();

    flex_row((
        label(icons::PERSON.to_string())
            .font(MATERIAL_SYMBOLS_FAMILY)
            .text_size(ICON_SIZE_SM)
            .color(ICON_COLOR)
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
    .boxed()
}

pub fn list_demo(model: &mut AppModel) -> impl WidgetView<AppModel, ()> + use<'_> {
    // Update selection item order for shift+click range selection
    let contact_ids: Vec<u64> = model.contacts.iter().map(|c| c.id).collect();
    model.list_selection.set_items(contact_ids);

    // Use the framework's list view
    let list_view = list_styled(
        &model.contacts,
        &model.list_selection,
        ListStyle::new().hover_bg(BG_HOVER),
        |contact, is_selected| contact_row(contact, is_selected),
        |state: &mut AppModel, action| {
            match action {
                ListAction::Select(id, mods) => state.list_selection.select(id, mods),
                ListAction::Activate(_id) => {
                    // Double-click action (e.g., open contact details)
                }
            }
        },
    );

    flex_col((
        label("Contacts")
            .text_size(16.0)
            .weight(xilem::FontWeight::BOLD)
            .color(TEXT_COLOR),
        label("Click to select, Cmd+click to add to selection")
            .text_size(12.0)
            .color(TEXT_SECONDARY),

        // Scrollable list
        portal(list_view),

        // Selection info
        flex_row((
            label(format!("Selected: {} contacts", model.list_selection.count()))
                .text_size(12.0)
                .color(TEXT_SECONDARY),
            button(label("Clear"), |model: &mut AppModel| {
                model.list_selection.clear();
            }),
        ))
        .gap(16.px()),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Start)
    .gap(8.px())
    .padding(16.0)
}
