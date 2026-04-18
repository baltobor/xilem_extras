//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Trait for collections of menu items (tuples, Vec).

use super::entry::BoxedMenuEntry;
use super::group::IntoMenuEntries;

/// Trait for collections of menu items.
///
/// Implemented for tuples of [`IntoMenuEntries`] items (up to 12 elements)
/// and `Vec<BoxedMenuEntry>`. Allows using tuple syntax for static menus:
///
/// ```ignore
/// (
///     menu_item("Open", |s| s.open()),
///     separator(),
///     group((
///         menu_item("Extra 1", |s| s.extra1()),
///         menu_item("Extra 2", |s| s.extra2()),
///     )),
///     menu_item("Close", |s| s.close()),
/// )
/// ```
pub trait MenuItems<State, Action>: Send + Sync + 'static {
    /// Collects all menu entries into a boxed vector.
    fn collect_entries(self) -> Vec<BoxedMenuEntry<State, Action>>;
}

// Implementation for Vec
impl<State, Action> MenuItems<State, Action> for Vec<BoxedMenuEntry<State, Action>>
where
    State: 'static,
    Action: 'static,
{
    fn collect_entries(self) -> Vec<BoxedMenuEntry<State, Action>> {
        self
    }
}

// Macro to generate tuple implementations.
// Each element can be any `IntoMenuEntries` — either a single `MenuEntry` item
// or a `Group` that flattens multiple items into the parent.
macro_rules! impl_menu_items_for_tuple {
    ($($idx:tt $T:ident),+) => {
        impl<State, Action, $($T),+> MenuItems<State, Action> for ($($T,)+)
        where
            State: 'static,
            Action: 'static,
            $($T: IntoMenuEntries<State, Action>),+
        {
            fn collect_entries(self) -> Vec<BoxedMenuEntry<State, Action>> {
                let mut entries = Vec::new();
                $(entries.extend(self.$idx.into_entries());)+
                entries
            }
        }
    };
}

// Implement for tuples of 1-12 elements
impl_menu_items_for_tuple!(0 T0);
impl_menu_items_for_tuple!(0 T0, 1 T1);
impl_menu_items_for_tuple!(0 T0, 1 T1, 2 T2);
impl_menu_items_for_tuple!(0 T0, 1 T1, 2 T2, 3 T3);
impl_menu_items_for_tuple!(0 T0, 1 T1, 2 T2, 3 T3, 4 T4);
impl_menu_items_for_tuple!(0 T0, 1 T1, 2 T2, 3 T3, 4 T4, 5 T5);
impl_menu_items_for_tuple!(0 T0, 1 T1, 2 T2, 3 T3, 4 T4, 5 T5, 6 T6);
impl_menu_items_for_tuple!(0 T0, 1 T1, 2 T2, 3 T3, 4 T4, 5 T5, 6 T6, 7 T7);
impl_menu_items_for_tuple!(0 T0, 1 T1, 2 T2, 3 T3, 4 T4, 5 T5, 6 T6, 7 T7, 8 T8);
impl_menu_items_for_tuple!(0 T0, 1 T1, 2 T2, 3 T3, 4 T4, 5 T5, 6 T6, 7 T7, 8 T8, 9 T9);
impl_menu_items_for_tuple!(0 T0, 1 T1, 2 T2, 3 T3, 4 T4, 5 T5, 6 T6, 7 T7, 8 T8, 9 T9, 10 T10);
impl_menu_items_for_tuple!(0 T0, 1 T1, 2 T2, 3 T3, 4 T4, 5 T5, 6 T6, 7 T7, 8 T8, 9 T9, 10 T10, 11 T11);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_items::{menu_item, separator, group};

    struct TestState {
        called: String,
    }

    #[test]
    fn tuple_of_two_items() {
        let items = (
            menu_item("Open", |s: &mut TestState| {
                s.called = "open".to_string();
            }),
            menu_item("Close", |s: &mut TestState| {
                s.called = "close".to_string();
            }),
        );

        let entries = items.collect_entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].label(), Some("Open"));
        assert_eq!(entries[1].label(), Some("Close"));
    }

    #[test]
    fn tuple_with_separator() {
        let items = (
            menu_item("Cut", |_: &mut TestState| {}),
            separator(),
            menu_item("Paste", |_: &mut TestState| {}),
        );

        let entries = items.collect_entries();
        assert_eq!(entries.len(), 3);
        assert!(entries[0].is_actionable());
        assert!(!entries[1].is_actionable());
        assert!(entries[2].is_actionable());
    }

    #[test]
    fn execute_action() {
        let items = (
            menu_item("Test", |s: &mut TestState| {
                s.called = "test".to_string();
            }),
        );

        let entries = items.collect_entries();
        let mut state = TestState { called: String::new() };

        entries[0].execute(&mut state);
        assert_eq!(state.called, "test");
    }

    #[test]
    fn group_flattens_into_parent() {
        let items = (
            menu_item("Before", |_: &mut TestState| {}),
            group((
                menu_item("A", |_: &mut TestState| {}),
                menu_item("B", |_: &mut TestState| {}),
                menu_item("C", |_: &mut TestState| {}),
            )),
            menu_item("After", |_: &mut TestState| {}),
        );

        let entries = items.collect_entries();
        assert_eq!(entries.len(), 5);
        assert_eq!(entries[0].label(), Some("Before"));
        assert_eq!(entries[1].label(), Some("A"));
        assert_eq!(entries[2].label(), Some("B"));
        assert_eq!(entries[3].label(), Some("C"));
        assert_eq!(entries[4].label(), Some("After"));
    }

    #[test]
    fn nested_groups() {
        let items = (
            group((
                menu_item("A", |_: &mut TestState| {}),
                menu_item("B", |_: &mut TestState| {}),
            )),
            group((
                menu_item("C", |_: &mut TestState| {}),
                menu_item("D", |_: &mut TestState| {}),
            )),
        );

        let entries = items.collect_entries();
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].label(), Some("A"));
        assert_eq!(entries[1].label(), Some("B"));
        assert_eq!(entries[2].label(), Some("C"));
        assert_eq!(entries[3].label(), Some("D"));
    }

    #[test]
    fn group_with_separator() {
        let items = (
            group((
                menu_item("Cut", |_: &mut TestState| {}),
                separator(),
                menu_item("Paste", |_: &mut TestState| {}),
            )),
        );

        let entries = items.collect_entries();
        assert_eq!(entries.len(), 3);
        assert!(entries[0].is_actionable());
        assert!(!entries[1].is_actionable());
        assert!(entries[2].is_actionable());
    }
}
