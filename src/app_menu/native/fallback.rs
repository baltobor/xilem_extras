//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Fallback menu bar for Linux using xilem_extras menu_button widgets.
//!
//! This backend renders the application menu bar as a horizontal row of
//! pulldown menu buttons, reusing the existing menu_button infrastructure.

use std::marker::PhantomData;
use std::sync::Arc;

use crate::app_menu::builder::{MenuBarBuilder, MenuBuilder, MenuItemBuilder};

/// Fallback menu bar using xilem_extras widgets.
///
/// On Linux (where muda doesn't work), this creates a horizontal bar
/// of menu_button widgets that look like a traditional menu bar.
pub struct FallbackMenuBar<State, Action> {
    _phantom: PhantomData<fn(&mut State) -> Action>,
}

impl<State: 'static, Action: 'static> FallbackMenuBar<State, Action> {
    /// Create a new fallback menu bar.
    pub fn new(_builder: &MenuBarBuilder<State, Action>, _state: &State) -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

/// Stored action for fallback menu items.
pub struct FallbackAction<State, Action> {
    pub action: Arc<dyn Fn(&mut State) -> Action + Send + Sync>,
    pub enabled: Option<Arc<dyn Fn(&State) -> bool + Send + Sync>>,
    pub checked: Option<Arc<dyn Fn(&State) -> bool + Send + Sync>>,
}

/// Convert MenuItemBuilder to fallback menu items for the existing menu_button API.
pub fn convert_items_to_fallback<State, Action>(
    items: &[MenuItemBuilder<State, Action>],
    state: &State,
) -> Vec<FallbackMenuItemData<State, Action>>
where
    State: 'static,
    Action: 'static,
{
    let mut result = Vec::new();

    for item in items {
        match item {
            MenuItemBuilder::Action {
                label,
                action,
                shortcut,
                enabled,
                checked,
            } => {
                let shortcut_text = shortcut.as_ref().map(|s| s.display());
                let is_enabled = enabled.as_ref().is_none_or(|f| f(state));
                let is_checked = checked.as_ref().is_some_and(|f| f(state));

                result.push(FallbackMenuItemData::Action {
                    label: label.clone(),
                    id: None,
                    shortcut_text,
                    enabled: is_enabled,
                    checked: is_checked,
                    action: Some(Arc::clone(action)),
                });
            }
            MenuItemBuilder::ActionEnum {
                label,
                id,
                shortcut,
                enabled,
                checked,
                ..
            } => {
                let shortcut_text = shortcut.as_ref().map(|s| s.display());
                let is_enabled = enabled.as_ref().is_none_or(|f| f(state));
                let is_checked = checked.as_ref().is_some_and(|f| f(state));

                result.push(FallbackMenuItemData::Action {
                    label: label.clone(),
                    id: Some(id.clone()),
                    shortcut_text,
                    enabled: is_enabled,
                    checked: is_checked,
                    action: None, // ActionEnum uses ID-based dispatch
                });
            }
            MenuItemBuilder::Separator => {
                result.push(FallbackMenuItemData::Separator);
            }
            MenuItemBuilder::Submenu { label, items: sub_items } => {
                let children = convert_items_to_fallback(sub_items, state);
                result.push(FallbackMenuItemData::Submenu {
                    label: label.clone(),
                    children,
                });
            }
            MenuItemBuilder::Dynamic { builder } => {
                let dynamic_items = builder(state);
                let converted = convert_items_to_fallback(&dynamic_items, state);
                result.extend(converted);
            }
        }
    }

    result
}

/// Menu item data for the fallback renderer.
pub enum FallbackMenuItemData<State, Action> {
    Action {
        label: String,
        /// ID for ActionEnum items (used for event routing).
        id: Option<String>,
        shortcut_text: Option<String>,
        enabled: bool,
        checked: bool,
        /// Closure action (None for ActionEnum items which use ID-based dispatch).
        action: Option<Arc<dyn Fn(&mut State) -> Action + Send + Sync>>,
    },
    Separator,
    Submenu {
        label: String,
        children: Vec<FallbackMenuItemData<State, Action>>,
    },
}

/// Convert a MenuBuilder to fallback data.
pub fn menu_to_fallback_data<State, Action>(
    menu: &MenuBuilder<State, Action>,
    state: &State,
) -> FallbackMenuData<State, Action>
where
    State: 'static,
    Action: 'static,
{
    FallbackMenuData {
        title: menu.title.clone(),
        items: convert_items_to_fallback(&menu.items, state),
    }
}

/// Top-level menu data for fallback rendering.
pub struct FallbackMenuData<State, Action> {
    pub title: String,
    pub items: Vec<FallbackMenuItemData<State, Action>>,
}
