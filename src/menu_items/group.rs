//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Transparent grouping for menu items.
//!
//! `Group` is analogous to SwiftUI's `Group` — it lets you combine multiple menu items
//! into a single tuple element without creating visual nesting (unlike [`submenu`]).
//! This works around the 12-element tuple limit for [`MenuItems`].
//!
//! # Example
//!
//! ```ignore
//! use xilem_extras::menu_items::{menu_item, group, separator};
//!
//! // Without Group, this 15-item tuple would fail to compile.
//! // With Group, we split it into smaller chunks:
//! submenu("Quick Fixes", (
//!     group((
//!         menu_item("Fix A", |s: &mut State| s.fix_a()),
//!         menu_item("Fix B", |s| s.fix_b()),
//!         menu_item("Fix C", |s| s.fix_c()),
//!         // ... up to 12 items per group
//!     )),
//!     group((
//!         menu_item("Fix D", |s| s.fix_d()),
//!         menu_item("Fix E", |s| s.fix_e()),
//!     )),
//! ))
//! ```

use std::marker::PhantomData;

use super::entry::BoxedMenuEntry;
use super::sequence::MenuItems;

/// A transparent wrapper that groups menu items without creating a visual submenu.
///
/// Items inside a `Group` are flattened into the parent menu. This is purely a
/// compile-time construct to work around tuple size limits in [`MenuItems`].
///
/// Created via [`group`].
pub struct Group<State, Action, Items> {
    items: Items,
    _phantom: PhantomData<fn(&mut State) -> Action>,
}

impl<State, Action, Items> Clone for Group<State, Action, Items>
where
    Items: Clone,
{
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<State, Action, Items> Group<State, Action, Items> {
    /// Creates a new group with the given items.
    pub fn new(items: Items) -> Self {
        Self {
            items,
            _phantom: PhantomData,
        }
    }
}

/// Trait for types that can produce one or more menu entries.
///
/// This is implemented for:
/// - [`MenuItem`](super::MenuItem) (produces a single entry)
/// - [`SeparatorEntry`](super::SeparatorEntry) (produces a single entry)
/// - [`Submenu`](super::Submenu) (produces a single entry)
/// - [`Group`] (produces multiple entries by flattening)
///
/// The tuple implementations of [`MenuItems`] use this trait, allowing
/// both individual menu items and groups to appear as tuple elements.
pub trait IntoMenuEntries<State, Action>: Send + Sync + 'static {
    /// Converts this value into a vector of boxed menu entries.
    fn into_entries(self) -> Vec<BoxedMenuEntry<State, Action>>;
}

/// Helper to implement IntoMenuEntries for a single MenuEntry item.
macro_rules! impl_into_menu_entries_for_entry {
    ($ty:ty $(, $generics:ident)*) => {
        impl<State, Action $(, $generics)*> IntoMenuEntries<State, Action> for $ty
        where
            Self: super::entry::MenuEntry<State, Action>,
        {
            fn into_entries(self) -> Vec<BoxedMenuEntry<State, Action>> {
                vec![Box::new(self)]
            }
        }
    };
}

impl_into_menu_entries_for_entry!(super::item::MenuItem<State, Action>);
impl_into_menu_entries_for_entry!(super::separator::SeparatorEntry<State, Action>);
impl_into_menu_entries_for_entry!(super::submenu::Submenu<State, Action, Items>, Items);

// Group flattens its items into the parent.
impl<State, Action, Items> IntoMenuEntries<State, Action> for Group<State, Action, Items>
where
    State: 'static,
    Action: 'static,
    Items: MenuItems<State, Action> + Send + Sync + 'static,
{
    fn into_entries(self) -> Vec<BoxedMenuEntry<State, Action>> {
        self.items.collect_entries()
    }
}

/// Creates a transparent group of menu items.
///
/// Items inside the group are flattened into the parent menu — no visual
/// nesting is created. Use this to work around the 12-element tuple limit.
///
/// # Example
///
/// ```ignore
/// submenu("All Fixes", (
///     group((
///         menu_item("Fix 1", |s| s.fix1()),
///         menu_item("Fix 2", |s| s.fix2()),
///         // ...
///     )),
///     group((
///         menu_item("Fix 13", |s| s.fix13()),
///         menu_item("Fix 14", |s| s.fix14()),
///     )),
/// ))
/// ```
pub fn group<State, Action, Items>(
    items: Items,
) -> Group<State, Action, Items>
where
    Items: MenuItems<State, Action>,
{
    Group::new(items)
}
