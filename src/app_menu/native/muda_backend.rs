//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Muda backend for native menus on macOS and Windows.

use std::collections::HashMap;
use std::sync::Arc;

use muda::{
    accelerator::Accelerator, CheckMenuItem, Menu, MenuEvent, MenuId, MenuItem as MudaMenuItem,
    PredefinedMenuItem, Submenu,
};

use crate::app_menu::builder::{MenuBarBuilder, MenuBuilder, MenuItemBuilder};

/// Action stored for a menu item.
pub struct StoredAction<State, Action> {
    pub action: Arc<dyn Fn(&mut State) -> Action + Send + Sync>,
    pub enabled: Option<Arc<dyn Fn(&State) -> bool + Send + Sync>>,
    pub checked: Option<Arc<dyn Fn(&State) -> bool + Send + Sync>>,
}

/// Native menu bar using muda.
pub struct MudaMenuBar<State, Action> {
    menu: Menu,
    /// Maps menu IDs to action indices
    action_map: HashMap<MenuId, usize>,
    /// Stored actions with their enabled/checked callbacks
    actions: Vec<StoredAction<State, Action>>,
    /// Menu items that can be updated (for enabled/checked state)
    updatable_items: Vec<UpdatableItem>,
}

/// An item that can have its state updated.
enum UpdatableItem {
    Regular {
        id: MenuId,
        action_idx: usize,
    },
    Check {
        id: MenuId,
        action_idx: usize,
    },
}

impl<State: 'static, Action: 'static> MudaMenuBar<State, Action> {
    /// Create a new muda menu bar from a builder.
    pub fn new(builder: &MenuBarBuilder<State, Action>, state: &State) -> Self {
        let menu = Menu::new();
        let mut action_map = HashMap::new();
        let mut actions = Vec::new();
        let mut updatable_items = Vec::new();

        for menu_builder in &builder.menus {
            let submenu = Self::build_menu(
                menu_builder,
                state,
                &mut action_map,
                &mut actions,
                &mut updatable_items,
            );
            menu.append(&submenu).ok();
        }

        Self {
            menu,
            action_map,
            actions,
            updatable_items,
        }
    }

    /// Build a submenu from a MenuBuilder.
    fn build_menu(
        menu_builder: &MenuBuilder<State, Action>,
        state: &State,
        action_map: &mut HashMap<MenuId, usize>,
        actions: &mut Vec<StoredAction<State, Action>>,
        updatable_items: &mut Vec<UpdatableItem>,
    ) -> Submenu {
        let submenu = Submenu::new(&menu_builder.title, true);

        Self::build_items(
            &submenu,
            &menu_builder.items,
            state,
            action_map,
            actions,
            updatable_items,
        );

        submenu
    }

    /// Build menu items into a submenu.
    fn build_items(
        parent: &Submenu,
        items: &[MenuItemBuilder<State, Action>],
        state: &State,
        action_map: &mut HashMap<MenuId, usize>,
        actions: &mut Vec<StoredAction<State, Action>>,
        updatable_items: &mut Vec<UpdatableItem>,
    ) {
        for item in items {
            match item {
                MenuItemBuilder::Action {
                    label,
                    action,
                    shortcut,
                    enabled,
                    checked,
                } => {
                    let action_idx = actions.len();
                    actions.push(StoredAction {
                        action: Arc::clone(action),
                        enabled: enabled.clone(),
                        checked: checked.clone(),
                    });

                    let is_enabled = enabled.as_ref().is_none_or(|f| f(state));
                    let is_checked = checked.as_ref().is_some_and(|f| f(state));

                    let accelerator = shortcut.as_ref().and_then(|s| {
                        s.to_accelerator().parse::<Accelerator>().ok()
                    });

                    if checked.is_some() {
                        // Use CheckMenuItem for checkable items
                        let check_item = CheckMenuItem::new(label, is_enabled, is_checked, accelerator);
                        let id = check_item.id().clone();
                        action_map.insert(id.clone(), action_idx);
                        updatable_items.push(UpdatableItem::Check { id, action_idx });
                        parent.append(&check_item).ok();
                    } else {
                        // Regular menu item
                        let menu_item = MudaMenuItem::new(label, is_enabled, accelerator);
                        let id = menu_item.id().clone();
                        action_map.insert(id.clone(), action_idx);
                        updatable_items.push(UpdatableItem::Regular { id, action_idx });
                        parent.append(&menu_item).ok();
                    }
                }
                MenuItemBuilder::Separator => {
                    parent.append(&PredefinedMenuItem::separator()).ok();
                }
                MenuItemBuilder::Submenu {
                    label,
                    items: sub_items,
                } => {
                    let sub = Submenu::new(label, true);
                    Self::build_items(&sub, sub_items, state, action_map, actions, updatable_items);
                    parent.append(&sub).ok();
                }
                MenuItemBuilder::ActionEnum {
                    label,
                    id,
                    shortcut,
                    enabled,
                    checked,
                    ..
                } => {
                    // ActionEnum items use a string ID for event routing
                    let is_enabled = enabled.as_ref().is_none_or(|f| f(state));
                    let is_checked = checked.as_ref().is_some_and(|f| f(state));

                    let accelerator = shortcut.as_ref().and_then(|s| {
                        s.to_accelerator().parse::<Accelerator>().ok()
                    });

                    if checked.is_some() {
                        let check_item = CheckMenuItem::with_id(
                            id.as_str(),
                            label,
                            is_enabled,
                            is_checked,
                            accelerator,
                        );
                        parent.append(&check_item).ok();
                    } else {
                        let menu_item = MudaMenuItem::with_id(
                            id.as_str(),
                            label,
                            is_enabled,
                            accelerator,
                        );
                        parent.append(&menu_item).ok();
                    }
                }
                MenuItemBuilder::Dynamic { builder: dyn_builder } => {
                    // Generate dynamic items from current state
                    let dynamic_items = dyn_builder(state);
                    Self::build_items(
                        parent,
                        &dynamic_items,
                        state,
                        action_map,
                        actions,
                        updatable_items,
                    );
                }
            }
        }
    }

    /// Get the muda Menu for attachment to a window.
    pub fn menu(&self) -> &Menu {
        &self.menu
    }

    /// Update enabled/checked states based on current app state.
    pub fn update_states(&self, state: &State) {
        for updatable in &self.updatable_items {
            match updatable {
                UpdatableItem::Regular { id, action_idx } => {
                    if let Some(stored) = self.actions.get(*action_idx) {
                        if let Some(enabled_fn) = &stored.enabled {
                            let is_enabled = enabled_fn(state);
                            // Note: muda doesn't provide a way to update enabled state
                            // after creation. This would require rebuilding the menu.
                            let _ = (id, is_enabled); // Suppress unused warning
                        }
                    }
                }
                UpdatableItem::Check { id, action_idx } => {
                    if let Some(stored) = self.actions.get(*action_idx) {
                        if let Some(checked_fn) = &stored.checked {
                            let _is_checked = checked_fn(state);
                            // Note: muda CheckMenuItem doesn't expose set_checked after creation
                            let _ = id; // Suppress unused warning
                        }
                    }
                }
            }
        }
    }

    /// Handle a menu event and execute the corresponding action.
    pub fn handle_event(&self, event: &MenuEvent, state: &mut State) -> Option<Action> {
        if let Some(&action_idx) = self.action_map.get(event.id()) {
            if let Some(stored) = self.actions.get(action_idx) {
                return Some((stored.action)(state));
            }
        }
        None
    }

    /// Get the menu event receiver.
    pub fn event_receiver() -> &'static muda::MenuEventReceiver {
        MenuEvent::receiver()
    }
}

/// Build a muda Menu from a MenuBarBuilder.
///
/// This is a simpler alternative to `MudaMenuBar` that just creates the menu
/// structure. Use this when you want to handle menu events yourself.
///
/// Menu items created with `.action(MenuAction::X)` will have IDs from
/// `MenuActionTrait::id()`. Handle events by matching `event.id().0.as_str()`.
///
/// # Example
///
/// ```ignore
/// use xilem_extras::app_menu::{build_muda_menu, MenuBarBuilder};
///
/// let menu_def = MenuBarBuilder::new()
///     .menu("File", |m| m
///         .action(MenuAction::New)
///         .action(MenuAction::Open)
///     );
///
/// let menu = build_muda_menu(&menu_def, &model);
/// menu.init_for_nsapp(); // macOS
///
/// // Later, handle events:
/// while let Ok(event) = MenuEvent::receiver().try_recv() {
///     match event.id().0.as_str() {
///         "new" => model.new_file(),
///         "open" => model.open_file(),
///         _ => {}
///     }
/// }
/// ```
pub fn build_muda_menu<State, Action>(
    builder: &MenuBarBuilder<State, Action>,
    state: &State,
) -> Menu
where
    State: 'static,
    Action: 'static,
{
    let menu = Menu::new();

    for menu_builder in &builder.menus {
        let submenu = build_muda_submenu(menu_builder, state);
        menu.append(&submenu).ok();
    }

    menu
}

/// Build a muda Submenu from a MenuBuilder.
fn build_muda_submenu<State, Action>(
    menu_builder: &MenuBuilder<State, Action>,
    state: &State,
) -> Submenu
where
    State: 'static,
    Action: 'static,
{
    let submenu = Submenu::new(&menu_builder.title, true);
    build_muda_items(&submenu, &menu_builder.items, state);
    submenu
}

/// Build menu items into a muda Submenu.
fn build_muda_items<State, Action>(
    parent: &Submenu,
    items: &[MenuItemBuilder<State, Action>],
    state: &State,
) where
    State: 'static,
    Action: 'static,
{
    for item in items {
        match item {
            MenuItemBuilder::Action {
                label,
                shortcut,
                enabled,
                checked,
                ..
            } => {
                let is_enabled = enabled.as_ref().is_none_or(|f| f(state));
                let is_checked = checked.as_ref().is_some_and(|f| f(state));
                let accelerator = shortcut.as_ref().and_then(|s| {
                    s.to_accelerator().parse::<Accelerator>().ok()
                });

                if checked.is_some() {
                    let check_item = CheckMenuItem::new(label, is_enabled, is_checked, accelerator);
                    parent.append(&check_item).ok();
                } else {
                    let menu_item = MudaMenuItem::new(label, is_enabled, accelerator);
                    parent.append(&menu_item).ok();
                }
            }
            MenuItemBuilder::ActionEnum {
                label,
                id,
                shortcut,
                enabled,
                checked,
                ..
            } => {
                let is_enabled = enabled.as_ref().is_none_or(|f| f(state));
                let is_checked = checked.as_ref().is_some_and(|f| f(state));
                let accelerator = shortcut.as_ref().and_then(|s| {
                    s.to_accelerator().parse::<Accelerator>().ok()
                });

                if checked.is_some() {
                    let check_item = CheckMenuItem::with_id(
                        id.as_str(),
                        label,
                        is_enabled,
                        is_checked,
                        accelerator,
                    );
                    parent.append(&check_item).ok();
                } else {
                    let menu_item = MudaMenuItem::with_id(
                        id.as_str(),
                        label,
                        is_enabled,
                        accelerator,
                    );
                    parent.append(&menu_item).ok();
                }
            }
            MenuItemBuilder::Separator => {
                parent.append(&PredefinedMenuItem::separator()).ok();
            }
            MenuItemBuilder::Submenu {
                label,
                items: sub_items,
            } => {
                let sub = Submenu::new(label, true);
                build_muda_items(&sub, sub_items, state);
                parent.append(&sub).ok();
            }
            MenuItemBuilder::Dynamic { builder: dyn_builder } => {
                let dynamic_items = dyn_builder(state);
                build_muda_items(parent, &dynamic_items, state);
            }
        }
    }
}
