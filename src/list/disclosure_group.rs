//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

use std::marker::PhantomData;

/// A collapsible group with a disclosure indicator.
///
/// Combines a clickable header with expandable content.
pub struct DisclosureGroup<State, Action, L, C, F> {
    label: L,
    is_expanded: bool,
    on_toggle: F,
    content: C,
    _phantom: PhantomData<(State, Action)>,
}

impl<State, Action, L, C, F> DisclosureGroup<State, Action, L, C, F>
where
    F: Fn(&mut State, bool) -> Action,
{
    /// Creates a new disclosure group.
    pub fn new(label: L, is_expanded: bool, on_toggle: F, content: C) -> Self {
        Self {
            label,
            is_expanded,
            on_toggle,
            content,
            _phantom: PhantomData,
        }
    }

    /// Returns whether the group is expanded.
    pub fn is_expanded(&self) -> bool {
        self.is_expanded
    }
}

/// Creates a disclosure group.
///
/// # Arguments
///
/// * `label` - The header label view
/// * `is_expanded` - Whether the group is currently expanded
/// * `on_toggle` - Callback when the group is toggled
/// * `content` - Function returning the expandable content
///
/// # Example
///
/// ```ignore
/// disclosure_group(
///     label("Advanced Options"),
///     model.advanced_expanded,
///     |model, expanded| model.advanced_expanded = expanded,
///     || flex_col((
///         option_row("Enable feature X"),
///         option_row("Enable feature Y"),
///     )),
/// )
/// ```
pub fn disclosure_group<State, Action, L, C, F>(
    label: L,
    is_expanded: bool,
    on_toggle: F,
    content: C,
) -> DisclosureGroup<State, Action, L, C, F>
where
    F: Fn(&mut State, bool) -> Action,
{
    DisclosureGroup::new(label, is_expanded, on_toggle, content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disclosure_group_expanded() {
        let group: DisclosureGroup<(), (), &str, &str, _> = disclosure_group(
            "Label",
            true,
            |_, _| {},
            "Content",
        );
        assert!(group.is_expanded());
    }

    #[test]
    fn disclosure_group_collapsed() {
        let group: DisclosureGroup<(), (), &str, &str, _> = disclosure_group(
            "Label",
            false,
            |_, _| {},
            "Content",
        );
        assert!(!group.is_expanded());
    }
}
