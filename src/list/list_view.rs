//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! List view for flat collections with selection support.

use masonry::layout::AsUnit;
use xilem::masonry::peniko::Color;
use xilem::style::Style;
use xilem::view::flex_col;
use xilem::{AnyWidgetView, WidgetView};

use crate::components::{row_button_with_press, RowButtonPress};
use crate::traits::{Identifiable, SelectionModifiers, SelectionState};
use xilem::masonry::core::PointerButton;

/// Actions that can occur on list items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListAction<Id> {
    /// Item selected (with modifiers for multi-selection)
    Select(Id, SelectionModifiers),
    /// Item activated (double-click or Enter)
    Activate(Id),
}

/// Style configuration for list rows.
#[derive(Debug, Clone)]
pub struct ListStyle {
    /// Background color on hover.
    pub hover_bg: Color,
    /// Gap between rows in pixels.
    pub gap: f64,
}

impl Default for ListStyle {
    fn default() -> Self {
        Self {
            hover_bg: Color::TRANSPARENT,
            gap: 0.0,
        }
    }
}

impl ListStyle {
    /// Creates a new ListStyle with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the hover background color.
    pub fn hover_bg(mut self, color: Color) -> Self {
        self.hover_bg = color;
        self
    }

    /// Sets the gap between rows.
    pub fn gap(mut self, gap: f64) -> Self {
        self.gap = gap;
        self
    }
}

/// Creates a list view for a flat collection.
///
/// # Arguments
///
/// * `items` - The collection of items to display
/// * `selection` - The selection state
/// * `row_builder` - Function that builds a view for each item: `(item, is_selected) -> View`
/// * `handler` - Function that handles list actions
///
/// # Example
///
/// ```ignore
/// list(
///     &model.contacts,
///     &model.selection,
///     |contact, is_selected| {
///         flex_row((
///             label(&contact.name),
///             label(&contact.email),
///         ))
///         .background_color(if is_selected { BG_SELECTED } else { Color::TRANSPARENT })
///     },
///     |state, action| {
///         match action {
///             ListAction::Select(id, mods) => state.selection.select(id, mods),
///             ListAction::Activate(id) => state.open_contact(&id),
///         }
///     },
/// )
/// ```
pub fn list<'a, State, I, R, Sel, F, H>(
    items: &'a [I],
    selection: &'a Sel,
    row_builder: F,
    handler: H,
) -> impl WidgetView<State, ()> + use<'a, State, I, R, Sel, F, H>
where
    State: 'static,
    I: Identifiable + 'a,
    I::Id: Clone + Send + Sync + 'static,
    R: WidgetView<State, ()> + 'static,
    F: Fn(&I, bool) -> R + Clone + 'a,
    H: Fn(&mut State, ListAction<I::Id>) + Clone + Send + Sync + 'static,
    Sel: SelectionState<I::Id> + 'a,
{
    list_styled(items, selection, ListStyle::default(), row_builder, handler)
}

/// Creates a list view with custom styling.
///
/// Same as [`list`] but accepts a [`ListStyle`] for customization.
///
/// # Example
///
/// ```ignore
/// list_styled(
///     &model.contacts,
///     &model.selection,
///     ListStyle::new().hover_bg(BG_HOVER).gap(2.0),
///     |contact, is_selected| { ... },
///     |state, action| { ... },
/// )
/// ```
pub fn list_styled<'a, State, I, R, Sel, F, H>(
    items: &'a [I],
    selection: &'a Sel,
    style: ListStyle,
    row_builder: F,
    handler: H,
) -> impl WidgetView<State, ()> + use<'a, State, I, R, Sel, F, H>
where
    State: 'static,
    I: Identifiable + 'a,
    I::Id: Clone + Send + Sync + 'static,
    R: WidgetView<State, ()> + 'static,
    F: Fn(&I, bool) -> R + Clone + 'a,
    H: Fn(&mut State, ListAction<I::Id>) + Clone + Send + Sync + 'static,
    Sel: SelectionState<I::Id> + 'a,
{
    let rows: Vec<Box<AnyWidgetView<State, ()>>> = items
        .iter()
        .map(|item| {
            let is_selected = selection.is_selected(&item.id());
            let row_view = row_builder(item, is_selected);
            let item_id = item.id();
            let handler = handler.clone();
            let hover_bg = style.hover_bg;

            let btn = row_button_with_press(row_view, move |state: &mut State, press: &RowButtonPress| {
                // Only handle primary button clicks
                match press.button {
                    None | Some(PointerButton::Primary) => {
                        let sel_mods = SelectionModifiers::from_modifiers(press.modifiers);
                        let action = if press.click_count >= 2 {
                            ListAction::Activate(item_id.clone())
                        } else {
                            ListAction::Select(item_id.clone(), sel_mods)
                        };
                        handler(state, action);
                    }
                    _ => {}
                }
            })
            .hover_bg(hover_bg);

            btn.boxed()
        })
        .collect();

    flex_col(rows).gap(style.gap.px())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_action_select() {
        let action = ListAction::Select(42u64, SelectionModifiers::NONE);
        if let ListAction::Select(id, mods) = action {
            assert_eq!(id, 42);
            assert_eq!(mods, SelectionModifiers::NONE);
        } else {
            panic!("Expected Select action");
        }
    }

    #[test]
    fn list_action_activate() {
        let action = ListAction::<u64>::Activate(42);
        if let ListAction::Activate(id) = action {
            assert_eq!(id, 42);
        } else {
            panic!("Expected Activate action");
        }
    }

    #[test]
    fn list_action_equality() {
        let a1 = ListAction::Select(1u64, SelectionModifiers::NONE);
        let a2 = ListAction::Select(1u64, SelectionModifiers::NONE);
        let a3 = ListAction::Select(2u64, SelectionModifiers::NONE);
        let a4 = ListAction::Select(1u64, SelectionModifiers::COMMAND);

        assert_eq!(a1, a2);
        assert_ne!(a1, a3);
        assert_ne!(a1, a4);
    }

    #[test]
    fn list_action_with_modifiers() {
        let action = ListAction::Select(1u64, SelectionModifiers::COMMAND);
        if let ListAction::Select(_, mods) = action {
            assert!(mods.command);
            assert!(!mods.shift);
        }
    }

    #[test]
    fn list_style_builder() {
        let style = ListStyle::new()
            .hover_bg(Color::from_rgb8(50, 50, 50))
            .gap(4.0);

        assert_eq!(style.hover_bg, Color::from_rgb8(50, 50, 50));
        assert_eq!(style.gap, 4.0);
    }

    #[test]
    fn list_style_default() {
        let style = ListStyle::default();
        assert_eq!(style.hover_bg, Color::TRANSPARENT);
        assert_eq!(style.gap, 0.0);
    }
}
