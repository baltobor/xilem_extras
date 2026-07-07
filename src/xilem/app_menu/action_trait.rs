//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Menu action trait for enum-based menu definitions.
//!
//! This trait allows using an enum to define menu actions with labels and IDs,
//! enabling a single menu definition that works on all platforms.

use super::shortcut::Shortcut;

/// Trait for menu action enums.
///
/// Implement this trait on your menu action enum to enable single-definition menus.
///
/// # Example
///
/// ```ignore
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// pub enum MenuAction {
///     New,
///     Open,
///     Save,
///     Quit,
/// }
///
/// impl MenuActionTrait for MenuAction {
///     fn label(&self) -> &'static str {
///         match self {
///             Self::New => "New",
///             Self::Open => "Open...",
///             Self::Save => "Save",
///             Self::Quit => "Quit",
///         }
///     }
///
///     fn id(&self) -> &'static str {
///         match self {
///             Self::New => "new",
///             Self::Open => "open",
///             Self::Save => "save",
///             Self::Quit => "quit",
///         }
///     }
/// }
/// ```
pub trait MenuActionTrait: Copy + Send + Sync + 'static {
    /// Display label for this action (shown in menu).
    fn label(&self) -> &'static str;

    /// Unique ID for this action (used by muda for event routing).
    fn id(&self) -> &'static str;

    /// Keyboard shortcut for this action (optional).
    fn shortcut(&self) -> Option<Shortcut> {
        None
    }

    /// Whether this action is currently enabled (default: true).
    fn is_enabled<State>(&self, _state: &State) -> bool {
        true
    }

    /// Whether this action shows a checkmark (default: false).
    fn is_checked<State>(&self, _state: &State) -> bool {
        false
    }
}

/// Handler function type for menu actions.
///
/// This is called when a menu action is triggered. The handler should
/// dispatch the action to the appropriate model method.
pub type MenuActionHandler<State, A> = fn(&mut State, A);
