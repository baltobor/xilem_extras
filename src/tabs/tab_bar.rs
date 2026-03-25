//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Tab bar view component.

use std::marker::PhantomData;

use xilem::masonry::vello::peniko::Color;

use super::TabItem;

/// Golden ratio for proportional spacing.
const PHI: f64 = 1.618;

/// Color configuration for the tab bar.
///
/// Provides colors for active tabs, inactive tabs, and the bar background.
#[derive(Debug, Clone, Copy)]
pub struct TabBarColors {
    /// Background color for the active tab.
    pub active_bg: Color,
    /// Background color for inactive tabs.
    pub inactive_bg: Color,
    /// Background color for the tab bar itself.
    pub bar_bg: Color,
    /// Primary text color.
    pub text: Color,
    /// Secondary text color (for close button, dirty indicator).
    pub text_secondary: Color,
}

impl Default for TabBarColors {
    fn default() -> Self {
        Self {
            active_bg: Color::from_rgb8(55, 53, 50),
            inactive_bg: Color::from_rgb8(45, 43, 40),
            bar_bg: Color::from_rgb8(38, 36, 34),
            text: Color::from_rgb8(220, 218, 214),
            text_secondary: Color::from_rgb8(160, 156, 150),
        }
    }
}

/// A tab bar component for document-style interfaces.
///
/// Renders a horizontal strip of tabs with:
/// - Scrollable tab area (via portal)
/// - Previous/Next navigation buttons
/// - Close button per tab
/// - Dirty state indicator
///
/// # Type Parameters
///
/// - `State` - Application state type
/// - `Action` - Action type for callbacks
/// - `T` - Tab item type implementing [`TabItem`]
/// - `S` - Select callback type
/// - `C` - Close callback type
///
/// # Example
///
/// ```ignore
/// TabBar::new(&model.tabs, model.active_tab)
///     .colors(TabBarColors::default())
///     .on_select(|model, idx| model.active_tab = idx)
///     .on_close(|model, idx| model.close_tab(idx))
/// ```
pub struct TabBar<'a, State, Action, T, S, C> {
    tabs: &'a [T],
    active_index: usize,
    colors: TabBarColors,
    on_select: Option<S>,
    on_close: Option<C>,
    _phantom: PhantomData<(State, Action)>,
}

impl<'a, State, Action, T: TabItem> TabBar<'a, State, Action, T, (), ()> {
    /// Creates a new tab bar.
    ///
    /// # Arguments
    ///
    /// * `tabs` - Slice of tab items to display
    /// * `active_index` - Index of the currently active tab
    pub fn new(tabs: &'a [T], active_index: usize) -> Self {
        Self {
            tabs,
            active_index,
            colors: TabBarColors::default(),
            on_select: None,
            on_close: None,
            _phantom: PhantomData,
        }
    }
}

impl<'a, State, Action, T: TabItem, S, C> TabBar<'a, State, Action, T, S, C> {
    /// Sets the color configuration.
    pub fn colors(mut self, colors: TabBarColors) -> Self {
        self.colors = colors;
        self
    }

    /// Sets the callback for tab selection.
    ///
    /// Called when a tab is clicked (not the close button).
    pub fn on_select<F>(self, callback: F) -> TabBar<'a, State, Action, T, F, C>
    where
        F: Fn(&mut State, usize) -> Action + Clone + Send + Sync + 'static,
    {
        TabBar {
            tabs: self.tabs,
            active_index: self.active_index,
            colors: self.colors,
            on_select: Some(callback),
            on_close: self.on_close,
            _phantom: PhantomData,
        }
    }

    /// Sets the callback for tab close.
    ///
    /// Called when the close button (X) on a tab is clicked.
    pub fn on_close<F>(self, callback: F) -> TabBar<'a, State, Action, T, S, F>
    where
        F: Fn(&mut State, usize) -> Action + Clone + Send + Sync + 'static,
    {
        TabBar {
            tabs: self.tabs,
            active_index: self.active_index,
            colors: self.colors,
            on_select: self.on_select,
            on_close: Some(callback),
            _phantom: PhantomData,
        }
    }

    /// Returns the configured colors.
    pub fn get_colors(&self) -> &TabBarColors {
        &self.colors
    }

    /// Returns the tabs slice.
    pub fn get_tabs(&self) -> &[T] {
        self.tabs
    }

    /// Returns the active tab index.
    pub fn get_active_index(&self) -> usize {
        self.active_index
    }

    /// Returns whether navigation to previous tab is possible.
    pub fn can_go_prev(&self) -> bool {
        self.active_index > 0
    }

    /// Returns whether navigation to next tab is possible.
    pub fn can_go_next(&self) -> bool {
        !self.tabs.is_empty() && self.active_index + 1 < self.tabs.len()
    }

    /// Returns the golden ratio padding for tabs.
    pub fn tab_padding() -> (f64, f64) {
        let pad_v = 3.0;
        let pad_h = (pad_v * PHI).round();
        (pad_h, pad_v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tabs::SimpleTab;

    #[test]
    fn tab_bar_navigation() {
        let tabs = vec![
            SimpleTab::new("Tab 1"),
            SimpleTab::new("Tab 2"),
            SimpleTab::new("Tab 3"),
        ];

        let bar: TabBar<(), (), _, (), ()> = TabBar::new(&tabs, 0);
        assert!(!bar.can_go_prev());
        assert!(bar.can_go_next());

        let bar: TabBar<(), (), _, (), ()> = TabBar::new(&tabs, 1);
        assert!(bar.can_go_prev());
        assert!(bar.can_go_next());

        let bar: TabBar<(), (), _, (), ()> = TabBar::new(&tabs, 2);
        assert!(bar.can_go_prev());
        assert!(!bar.can_go_next());
    }

    #[test]
    fn tab_bar_empty() {
        let tabs: Vec<SimpleTab> = vec![];
        let bar: TabBar<(), (), _, (), ()> = TabBar::new(&tabs, 0);
        assert!(!bar.can_go_prev());
        assert!(!bar.can_go_next());
    }

    #[test]
    fn tab_padding_golden_ratio() {
        let (h, v) = TabBar::<(), (), SimpleTab, (), ()>::tab_padding();
        assert!((h / v - PHI).abs() < 0.1);
    }

    #[test]
    fn default_colors() {
        let colors = TabBarColors::default();
        assert_ne!(colors.active_bg, colors.inactive_bg);
    }
}
