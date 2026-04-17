//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Builder types for constructing menu hierarchies.
//!
//! Provides a fluent API for declarative menu construction.

use std::sync::Arc;

use super::shortcut::Shortcut;
use super::action_trait::MenuActionTrait;

/// Builder for the application menu bar.
pub struct MenuBarBuilder<State, Action> {
    pub(crate) menus: Vec<MenuBuilder<State, Action>>,
}

impl<State, Action> Default for MenuBarBuilder<State, Action> {
    fn default() -> Self {
        Self::new()
    }
}

impl<State, Action> MenuBarBuilder<State, Action> {
    /// Create a new empty menu bar builder.
    pub fn new() -> Self {
        Self { menus: Vec::new() }
    }

    /// Add a top-level menu (e.g., "File", "Edit").
    pub fn menu<F>(mut self, title: &str, builder: F) -> Self
    where
        F: FnOnce(MenuBuilder<State, Action>) -> MenuBuilder<State, Action>,
    {
        let menu = builder(MenuBuilder::new(title));
        self.menus.push(menu);
        self
    }
}

/// Builder for a single menu (e.g., "File" menu).
pub struct MenuBuilder<State, Action> {
    pub(crate) title: String,
    pub(crate) items: Vec<MenuItemBuilder<State, Action>>,
}

impl<State, Action> MenuBuilder<State, Action> {
    /// Create a new menu with the given title.
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            items: Vec::new(),
        }
    }

    /// Add an action item to the menu.
    pub fn item<L, F>(self, label: L, action: F) -> MenuItemChain<Self, State, Action>
    where
        L: Into<String>,
        F: Fn(&mut State) + Send + Sync + 'static,
        Action: Default + 'static,
    {
        let item = MenuItemBuilder::Action {
            label: label.into(),
            action: Arc::new(move |state| {
                action(state);
                Action::default()
            }),
            shortcut: None,
            enabled: None,
            checked: None,
        };
        MenuItemChain::new(self, item)
    }

    /// Add an action item that returns an Action.
    pub fn item_with_action<L, F>(self, label: L, action: F) -> MenuItemChain<Self, State, Action>
    where
        L: Into<String>,
        F: Fn(&mut State) -> Action + Send + Sync + 'static,
    {
        let item = MenuItemBuilder::Action {
            label: label.into(),
            action: Arc::new(action),
            shortcut: None,
            enabled: None,
            checked: None,
        };
        MenuItemChain::new(self, item)
    }

    /// Add an action item using a MenuActionTrait enum.
    ///
    /// This is the recommended way to add menu items when using an action enum.
    /// The label and ID are derived from the action.
    ///
    /// # Example
    ///
    /// ```ignore
    /// .action(MenuAction::New)
    /// .action(MenuAction::Open).shortcut(CMD + Key::O)
    /// ```
    pub fn action<A>(self, action: A) -> MenuItemChain<Self, State, Action>
    where
        A: MenuActionTrait,
        Action: Default + 'static,
    {
        let label = action.label().to_string();
        let id = action.id().to_string();
        let shortcut = action.shortcut();

        let item = MenuItemBuilder::ActionEnum {
            label,
            id,
            action_value: Box::new(action) as Box<dyn std::any::Any + Send + Sync>,
            shortcut,
            enabled: None,
            checked: None,
        };
        MenuItemChain::new(self, item)
    }

    /// Add a separator.
    pub fn separator(mut self) -> Self {
        self.items.push(MenuItemBuilder::Separator);
        self
    }

    /// Add a submenu.
    pub fn submenu<F>(mut self, label: &str, builder: F) -> Self
    where
        F: FnOnce(MenuBuilder<State, Action>) -> MenuBuilder<State, Action>,
    {
        let submenu = builder(MenuBuilder::new(label));
        self.items.push(MenuItemBuilder::Submenu {
            label: label.to_string(),
            items: submenu.items,
        });
        self
    }

    /// Add a dynamic menu section that rebuilds based on state.
    pub fn dynamic<F>(mut self, builder: F) -> Self
    where
        F: Fn(&State) -> Vec<MenuItemBuilder<State, Action>> + Send + Sync + 'static,
    {
        self.items.push(MenuItemBuilder::Dynamic {
            builder: Arc::new(builder),
        });
        self
    }

    /// Internal: add a pre-built item.
    pub(crate) fn push_item(mut self, item: MenuItemBuilder<State, Action>) -> Self {
        self.items.push(item);
        self
    }
}

/// Menu item variants.
pub enum MenuItemBuilder<State, Action> {
    /// An actionable menu item with a closure.
    Action {
        label: String,
        action: Arc<dyn Fn(&mut State) -> Action + Send + Sync>,
        shortcut: Option<Shortcut>,
        enabled: Option<Arc<dyn Fn(&State) -> bool + Send + Sync>>,
        checked: Option<Arc<dyn Fn(&State) -> bool + Send + Sync>>,
    },
    /// An actionable menu item using a MenuActionTrait enum.
    ///
    /// This stores the action enum value for later dispatch via a handler.
    ActionEnum {
        label: String,
        id: String,
        action_value: Box<dyn std::any::Any + Send + Sync>,
        shortcut: Option<Shortcut>,
        enabled: Option<Arc<dyn Fn(&State) -> bool + Send + Sync>>,
        checked: Option<Arc<dyn Fn(&State) -> bool + Send + Sync>>,
    },
    /// A separator line.
    Separator,
    /// A submenu containing more items.
    Submenu {
        label: String,
        items: Vec<MenuItemBuilder<State, Action>>,
    },
    /// Dynamic items generated from state.
    Dynamic {
        builder: Arc<dyn Fn(&State) -> Vec<MenuItemBuilder<State, Action>> + Send + Sync>,
    },
}

impl<State, Action> MenuItemBuilder<State, Action> {
    /// Get the label if this is an action or submenu.
    pub fn label(&self) -> Option<&str> {
        match self {
            MenuItemBuilder::Action { label, .. } => Some(label),
            MenuItemBuilder::ActionEnum { label, .. } => Some(label),
            MenuItemBuilder::Submenu { label, .. } => Some(label),
            _ => None,
        }
    }

    /// Get the ID if this is an ActionEnum item.
    pub fn id(&self) -> Option<&str> {
        match self {
            MenuItemBuilder::ActionEnum { id, .. } => Some(id),
            _ => None,
        }
    }

    /// Check if this item is enabled based on state.
    pub fn is_enabled(&self, state: &State) -> bool {
        match self {
            MenuItemBuilder::Action { enabled: Some(f), .. } => f(state),
            MenuItemBuilder::Action { enabled: None, .. } => true,
            MenuItemBuilder::ActionEnum { enabled: Some(f), .. } => f(state),
            MenuItemBuilder::ActionEnum { enabled: None, .. } => true,
            MenuItemBuilder::Submenu { .. } => true,
            MenuItemBuilder::Separator => true,
            MenuItemBuilder::Dynamic { .. } => true,
        }
    }

    /// Check if this item is checked based on state.
    pub fn is_checked(&self, state: &State) -> bool {
        match self {
            MenuItemBuilder::Action { checked: Some(f), .. } => f(state),
            MenuItemBuilder::ActionEnum { checked: Some(f), .. } => f(state),
            _ => false,
        }
    }

    /// Get the shortcut if any.
    pub fn shortcut(&self) -> Option<&Shortcut> {
        match self {
            MenuItemBuilder::Action { shortcut, .. } => shortcut.as_ref(),
            MenuItemBuilder::ActionEnum { shortcut, .. } => shortcut.as_ref(),
            _ => None,
        }
    }

    /// Get the action value as a typed reference (for ActionEnum items).
    pub fn action_value<A: 'static>(&self) -> Option<&A> {
        match self {
            MenuItemBuilder::ActionEnum { action_value, .. } => {
                action_value.downcast_ref::<A>()
            }
            _ => None,
        }
    }
}

/// Chain type for fluent item configuration.
///
/// Allows chaining `.shortcut()`, `.enabled()`, `.checked()` after `.item()`.
pub struct MenuItemChain<Parent, State, Action> {
    parent: Parent,
    item: MenuItemBuilder<State, Action>,
}

impl<State, Action> MenuItemChain<MenuBuilder<State, Action>, State, Action> {
    /// Create a new chain.
    pub(crate) fn new(parent: MenuBuilder<State, Action>, item: MenuItemBuilder<State, Action>) -> Self {
        Self { parent, item }
    }

    /// Set the keyboard shortcut for this item.
    pub fn shortcut(mut self, shortcut: Shortcut) -> Self {
        if let MenuItemBuilder::Action { shortcut: ref mut s, .. } = self.item {
            *s = Some(shortcut);
        }
        self
    }

    /// Set a condition for when this item is enabled.
    pub fn enabled<F>(mut self, f: F) -> Self
    where
        F: Fn(&State) -> bool + Send + Sync + 'static,
    {
        if let MenuItemBuilder::Action { enabled: ref mut e, .. } = self.item {
            *e = Some(Arc::new(f));
        }
        self
    }

    /// Set a condition for when this item shows a checkmark.
    pub fn checked<F>(mut self, f: F) -> Self
    where
        F: Fn(&State) -> bool + Send + Sync + 'static,
    {
        if let MenuItemBuilder::Action { checked: ref mut c, .. } = self.item {
            *c = Some(Arc::new(f));
        }
        self
    }

    /// Finalize the item and continue building the menu.
    /// This is called implicitly when chaining to another item or separator.
    fn finalize(self) -> MenuBuilder<State, Action> {
        self.parent.push_item(self.item)
    }

    /// Add another item after this one.
    pub fn item<L, F>(self, label: L, action: F) -> MenuItemChain<MenuBuilder<State, Action>, State, Action>
    where
        L: Into<String>,
        F: Fn(&mut State) + Send + Sync + 'static,
        Action: Default + 'static,
    {
        self.finalize().item(label, action)
    }

    /// Add another item that returns an Action.
    pub fn item_with_action<L, F>(self, label: L, action: F) -> MenuItemChain<MenuBuilder<State, Action>, State, Action>
    where
        L: Into<String>,
        F: Fn(&mut State) -> Action + Send + Sync + 'static,
    {
        self.finalize().item_with_action(label, action)
    }

    /// Add a separator after this item.
    pub fn separator(self) -> MenuBuilder<State, Action> {
        self.finalize().separator()
    }

    /// Add a submenu after this item.
    pub fn submenu<F>(self, label: &str, builder: F) -> MenuBuilder<State, Action>
    where
        F: FnOnce(MenuBuilder<State, Action>) -> MenuBuilder<State, Action>,
    {
        self.finalize().submenu(label, builder)
    }

    /// Add dynamic items after this item.
    pub fn dynamic<F>(self, builder: F) -> MenuBuilder<State, Action>
    where
        F: Fn(&State) -> Vec<MenuItemBuilder<State, Action>> + Send + Sync + 'static,
    {
        self.finalize().dynamic(builder)
    }

    /// Add an action enum item after this one.
    pub fn action<A>(self, action: A) -> MenuItemChain<MenuBuilder<State, Action>, State, Action>
    where
        A: MenuActionTrait,
        Action: Default + 'static,
    {
        self.finalize().action(action)
    }
}

// Allow MenuItemChain to be used as the final return type
impl<State, Action> From<MenuItemChain<MenuBuilder<State, Action>, State, Action>> for MenuBuilder<State, Action> {
    fn from(chain: MenuItemChain<MenuBuilder<State, Action>, State, Action>) -> Self {
        chain.finalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_menu::shortcut::{CMD, Key};

    #[derive(Default)]
    struct TestAction;

    struct TestState {
        can_undo: bool,
    }

    #[test]
    fn test_menu_builder() {
        let menu: MenuBuilder<TestState, TestAction> = MenuBuilder::new("File")
            .item("New", |_state| {})
            .shortcut(CMD + Key::N)
            .item("Open", |_state| {})
            .shortcut(CMD + Key::O)
            .separator()
            .item("Quit", |_state| {})
            .shortcut(CMD + Key::Q)
            .into();

        assert_eq!(menu.title, "File");
        assert_eq!(menu.items.len(), 4); // New, Open, Separator, Quit
    }

    #[test]
    fn test_enabled_check() {
        let menu: MenuBuilder<TestState, TestAction> = MenuBuilder::new("Edit")
            .item("Undo", |_state| {})
            .enabled(|state| state.can_undo)
            .into();

        let state_can = TestState { can_undo: true };
        let state_cannot = TestState { can_undo: false };

        if let MenuItemBuilder::Action { .. } = &menu.items[0] {
            assert!(menu.items[0].is_enabled(&state_can));
            assert!(!menu.items[0].is_enabled(&state_cannot));
        }
    }
}
