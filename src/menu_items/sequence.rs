//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Trait for collections of menu items (tuples, Vec).

use super::entry::{BoxedMenuEntry, MenuEntry};

/// Trait for collections of menu items.
///
/// Implemented for tuples of `MenuEntry` items (up to 12 elements) and `Vec<BoxedMenuEntry>`.
/// Allows using tuple syntax for static menus:
///
/// ```ignore
/// (
///     menu_item("Open", |s| s.open()),
///     separator(),
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

// Macro to generate tuple implementations
macro_rules! impl_menu_items_for_tuple {
    ($($idx:tt $T:ident),+) => {
        impl<State, Action, $($T),+> MenuItems<State, Action> for ($($T,)+)
        where
            State: 'static,
            Action: 'static,
            $($T: MenuEntry<State, Action>),+
        {
            fn collect_entries(self) -> Vec<BoxedMenuEntry<State, Action>> {
                vec![
                    $(Box::new(self.$idx) as BoxedMenuEntry<State, Action>,)+
                ]
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
    use crate::menu_items::{menu_item, separator};

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
}
