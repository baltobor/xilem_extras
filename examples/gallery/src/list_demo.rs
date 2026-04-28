//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! List widget demo - demonstrates the virtualized list view with keyboard navigation.

use masonry::layout::AsUnit;
use xilem::masonry::peniko::Color;
use xilem::style::Style;
use xilem::view::{CrossAxisAlignment, flex_col, flex_row, label, button};
use xilem::WidgetView;

use xilem_extras::{list_view_styled, ListViewAction, ListViewStyle, SelectionState};
use xilem_material_icons::{icons, FONT_FAMILY, ICON_SIZE_SM};

use crate::app_model::AppModel;

const TEXT_COLOR: Color = Color::from_rgb8(220, 218, 214);
const TEXT_SECONDARY: Color = Color::from_rgb8(160, 156, 150);
const BG_HOVER: Color = Color::from_rgb8(55, 53, 50);
const BG_SELECTED: Color = Color::from_rgb8(65, 62, 58);
const BG_STRIPE: Color = Color::from_rgb8(42, 40, 38);
const ICON_COLOR: Color = Color::from_rgb8(100, 180, 100);

fn contact_row(
    name: String,
    email: String,
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
        label(icons::PERSON.to_string())
            .font(FONT_FAMILY)
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
}

pub fn list_demo(model: &mut AppModel) -> impl WidgetView<AppModel, ()> + use<'_> {
    // Update selection item order for shift+click range selection
    let contact_ids: Vec<u64> = model.contacts.iter().map(|c| c.id).collect();
    model.list_selection.set_items(contact_ids);

    // Use the new virtualized list view with keyboard navigation
    let list_view = list_view_styled(
        &model.contacts,
        &model.list_selection,
        ListViewStyle::new()
            .row_height(52.0)  // Account for two-line rows
            .hover_bg(BG_HOVER)
            .selected_bg(BG_SELECTED)
            .striped(true)
            .stripe_bg(BG_STRIPE),
        |state: &mut AppModel, idx, is_selected, is_striped| {
            let contact = &state.contacts[idx];
            contact_row(contact.name.clone(), contact.email.clone(), is_selected, is_striped)
        },
        |state: &mut AppModel, action| {
            match action {
                ListViewAction::Select(id, mods) => state.list_selection.select(id, mods),
                ListViewAction::Activate(_id) => {
                    // Double-click / Enter action (e.g., open contact details)
                }
            }
        },
    );

    flex_col((
        label("Contacts (Virtualized List)")
            .text_size(16.0)
            .weight(xilem::FontWeight::BOLD)
            .color(TEXT_COLOR),
        label("Arrow keys to navigate, Shift+click for range, Cmd+click to toggle")
            .text_size(12.0)
            .color(TEXT_SECONDARY),

        // Virtualized list (handles its own scrolling)
        list_view,

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
