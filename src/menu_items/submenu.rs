//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Submenu entry for nested menu items.

use std::marker::PhantomData;

use xilem::masonry::core::{NewWidget, Widget};

use super::entry::MenuEntry;
use super::sequence::MenuItems;
use crate::menu_button::PulldownSubmenuItem;

/// A submenu entry that contains nested menu items.
///
/// Created via [`submenu`]. When hovered, displays a secondary dropdown
/// with the nested items.
///
/// # Example
///
/// ```ignore
/// use xilem_extras::menu_items::{menu_item, submenu, separator};
///
/// context_menu(child, (
///     menu_item("Cut", |s: &mut State| s.cut()),
///     submenu("Format", (
///         menu_item("Bold", |s| s.bold()),
///         menu_item("Italic", |s| s.italic()),
///     )),
/// ))
/// ```
pub struct Submenu<State, Action, Items> {
    label: String,
    items: Items,
    _phantom: PhantomData<fn(&mut State) -> Action>,
}

impl<State, Action, Items> Clone for Submenu<State, Action, Items>
where
    Items: Clone,
{
    fn clone(&self) -> Self {
        Self {
            label: self.label.clone(),
            items: self.items.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<State, Action, Items> Submenu<State, Action, Items> {
    /// Creates a new submenu with the given label and nested items.
    pub fn new(label: impl Into<String>, items: Items) -> Self {
        Self {
            label: label.into(),
            items,
            _phantom: PhantomData,
        }
    }

    /// Returns the nested items.
    pub fn items(&self) -> &Items {
        &self.items
    }
}

impl<State, Action, Items> MenuEntry<State, Action> for Submenu<State, Action, Items>
where
    State: 'static,
    Action: 'static,
    Items: MenuItems<State, Action> + Clone + Send + Sync + 'static,
{
    fn label(&self) -> Option<&str> {
        Some(&self.label)
    }

    fn is_actionable(&self) -> bool {
        false // Submenus are not directly actionable
    }

    fn is_submenu(&self) -> bool {
        true
    }

    fn submenu_items(&self) -> Option<Vec<super::BoxedMenuEntry<State, Action>>> {
        Some(self.items.clone().collect_entries())
    }

    fn build_widget(&self) -> NewWidget<dyn Widget> {
        NewWidget::new(PulldownSubmenuItem::new(self.label.clone())).erased()
    }

    fn execute(&self, _state: &mut State) -> Option<Action> {
        None // Submenus don't execute actions directly
    }

    fn clone_boxed(&self) -> super::BoxedMenuEntry<State, Action> {
        Box::new(self.clone())
    }
}

/// Creates a submenu with nested menu items.
///
/// # Example
///
/// ```ignore
/// submenu("Format", (
///     menu_item("Bold", |s| s.bold()),
///     menu_item("Italic", |s| s.italic()),
///     separator(),
///     menu_item("Underline", |s| s.underline()),
/// ))
/// ```
pub fn submenu<State, Action, Items>(
    label: impl Into<String>,
    items: Items,
) -> Submenu<State, Action, Items>
where
    Items: MenuItems<State, Action>,
{
    Submenu::new(label, items)
}
