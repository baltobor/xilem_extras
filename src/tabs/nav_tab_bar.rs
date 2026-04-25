//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Navigation tab bar view component.
//!
//! A simpler tab bar for fixed navigation tabs (view switching),
//! as opposed to document-style tabs that can be closed.

use std::marker::PhantomData;

use xilem::masonry::layout::AsUnit;
use xilem::masonry::peniko::Color;
use xilem::style::{Padding, Style};
use xilem::view::{button, flex_row, label, portal, FlexExt};
use xilem::{AnyWidgetView, WidgetView};

use super::{TabBarColors, TabItem};
use xilem_material_icons::{icon, icons, ICON_SIZE_SM};

/// Golden ratio for proportional spacing.
const PHI: f64 = 1.618;

/// Average character width in pixels at default text size (13pt).
const AVG_CHAR_WIDTH: f64 = 7.5;

/// Navigation button visibility mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NavButtonMode {
    /// Never show navigation buttons.
    Never,
    /// Always show navigation buttons.
    Always,
    /// Show navigation buttons automatically when tabs would overflow.
    /// The parameter is the maximum width in pixels before showing arrows.
    Auto(f64),
}

impl Default for NavButtonMode {
    fn default() -> Self {
        Self::Never
    }
}

/// A navigation tab bar for fixed view switching.
///
/// Unlike [`TabBar`], this component:
/// - Has no close buttons (tabs are fixed, not closable)
/// - Has no dirty state indicators
/// - Is designed for navigation between views, not document management
///
/// # Type Parameters
///
/// - `State` - Application state type
/// - `Action` - Action type for callbacks
/// - `T` - Tab item type implementing [`TabItem`]
/// - `S` - Select callback type
///
/// # Example
///
/// ```ignore
/// use xilem_extras::tabs::{NavTabBar, SimpleTab};
///
/// let tabs = vec![
///     SimpleTab::new("Overview"),
///     SimpleTab::new("Details"),
///     SimpleTab::new("Settings"),
/// ];
///
/// NavTabBar::new(&tabs, model.active_tab)
///     .on_select(|model, idx| model.active_tab = idx)
///     .build()
/// ```
///
/// [`TabBar`]: super::TabBar
pub struct NavTabBar<'a, State, Action, T, S> {
    tabs: &'a [T],
    active_index: usize,
    colors: TabBarColors,
    nav_button_mode: NavButtonMode,
    corner_radius: f64,
    text_size: f32,
    on_select: Option<S>,
    // Use fn-pointer PhantomData so `NavTabBar` is unconditionally `Send + Sync`,
    // regardless of whether `State: Sync`. Required so consumers whose State holds
    // non-Sync members (e.g. `std::sync::mpsc::Receiver`) can still use this view.
    _phantom: PhantomData<fn(&mut State) -> Action>,
}

impl<'a, State, Action, T: TabItem> NavTabBar<'a, State, Action, T, ()> {
    /// Creates a new navigation tab bar.
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
            nav_button_mode: NavButtonMode::Never,
            corner_radius: 4.0,
            text_size: 13.0,
            on_select: None,
            _phantom: PhantomData,
        }
    }
}

impl<'a, State, Action, T: TabItem, S> NavTabBar<'a, State, Action, T, S> {
    /// Sets the color configuration.
    ///
    /// Uses the same [`TabBarColors`] as [`TabBar`] for consistency.
    pub fn colors(mut self, colors: TabBarColors) -> Self {
        self.colors = colors;
        self
    }

    /// Sets whether to show navigation buttons (< >) for cycling through tabs.
    ///
    /// Default is `false` for navigation tabs.
    pub fn show_nav_buttons(mut self, show: bool) -> Self {
        self.nav_button_mode = if show {
            NavButtonMode::Always
        } else {
            NavButtonMode::Never
        };
        self
    }

    /// Automatically show navigation buttons when tabs would overflow.
    ///
    /// The `max_width` parameter specifies the available width in pixels.
    /// When the estimated total tab width exceeds this, arrows appear.
    ///
    /// # Example
    ///
    /// ```ignore
    /// NavTabBar::new(&tabs, active)
    ///     .auto_nav_buttons(300.0)  // Show arrows if tabs exceed 300px
    ///     .on_select(|model, idx| model.active = idx)
    ///     .build()
    /// ```
    pub fn auto_nav_buttons(mut self, max_width: f64) -> Self {
        self.nav_button_mode = NavButtonMode::Auto(max_width);
        self
    }

    /// Sets the corner radius for tab buttons.
    ///
    /// Default is `4.0`.
    pub fn corner_radius(mut self, radius: f64) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Sets the text size for tab labels.
    ///
    /// Default is `13.0`.
    pub fn text_size(mut self, size: f32) -> Self {
        self.text_size = size;
        self
    }

    /// Sets the callback for tab selection.
    ///
    /// Called when a tab is clicked.
    pub fn on_select<F>(self, callback: F) -> NavTabBar<'a, State, Action, T, F>
    where
        F: Fn(&mut State, usize) -> Action + Clone + Send + Sync + 'static,
    {
        NavTabBar {
            tabs: self.tabs,
            active_index: self.active_index,
            colors: self.colors,
            nav_button_mode: self.nav_button_mode,
            corner_radius: self.corner_radius,
            text_size: self.text_size,
            on_select: Some(callback),
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

    /// Estimates the total width of all tabs in pixels.
    ///
    /// Uses average character width and padding to approximate layout.
    pub fn estimated_tabs_width(&self) -> f64 {
        let (pad_h, _) = Self::tab_padding();
        let scale = self.text_size as f64 / 13.0; // Scale relative to default
        let char_width = AVG_CHAR_WIDTH * scale;
        let gap = 4.0; // Gap between tabs

        self.tabs
            .iter()
            .map(|tab| {
                let text_width = tab.title().len() as f64 * char_width;
                text_width + pad_h * 2.0
            })
            .sum::<f64>()
            + (self.tabs.len().saturating_sub(1)) as f64 * gap
    }

    /// Returns whether navigation buttons should be shown based on mode.
    pub fn should_show_nav(&self) -> bool {
        match self.nav_button_mode {
            NavButtonMode::Never => false,
            NavButtonMode::Always => true,
            NavButtonMode::Auto(max_width) => self.estimated_tabs_width() > max_width,
        }
    }
}

impl<'a, State, Action, T, S> NavTabBar<'a, State, Action, T, S>
where
    State: 'static,
    Action: 'static,
    T: TabItem,
    S: Fn(&mut State, usize) -> Action + Clone + Send + Sync + 'static,
{
    /// Builds the navigation tab bar view.
    ///
    /// Returns a tab bar widget with optional navigation arrows,
    /// but without close buttons or dirty indicators.
    pub fn build(self) -> impl WidgetView<State, Action> + use<'a, State, Action, T, S> {
        let colors = self.colors;
        let active_idx = self.active_index;
        let can_prev = self.can_go_prev();
        let can_next = self.can_go_next();
        let show_nav = self.should_show_nav();
        let corner_radius = self.corner_radius;
        let text_size = self.text_size;

        let (pad_h, pad_v) = Self::tab_padding();
        let tab_padding = Padding {
            top: pad_v,
            bottom: pad_v,
            left: pad_h,
            right: pad_h,
        };

        // Build tab buttons (without close button)
        let tab_buttons: Vec<Box<AnyWidgetView<State, Action>>> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(i, tab)| {
                let is_active = i == active_idx;
                let bg = if is_active {
                    colors.active_bg
                } else {
                    colors.inactive_bg
                };
                let text_color = if is_active {
                    colors.text
                } else {
                    colors.text_secondary
                };

                let on_select = self.on_select.clone();
                button(
                    label(tab.title()).text_size(text_size).color(text_color),
                    move |state: &mut State| {
                        if let Some(ref cb) = on_select {
                            cb(state, i)
                        } else {
                            unreachable!()
                        }
                    },
                )
                .background_color(bg)
                .border(Color::TRANSPARENT, 0.0)
                .corner_radius(corner_radius)
                .padding(tab_padding)
                .boxed()
            })
            .collect();

        // Scrollable tabs via portal
        let scrollable_tabs = portal(flex_row(tab_buttons).gap(4.px())).flex(1.0);

        if show_nav {
            // Navigation buttons
            let on_select_prev = self.on_select.clone();
            let prev_color = if can_prev {
                colors.text
            } else {
                colors.text_secondary
            };
            let prev_btn = button(
                icon(icons::CHEVRON_LEFT)
                    .size(ICON_SIZE_SM)
                    .color(prev_color)
                    .build(),
                move |state: &mut State| {
                    if can_prev {
                        if let Some(ref cb) = on_select_prev {
                            cb(state, active_idx - 1)
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                },
            )
            .disabled(!can_prev)
            .background_color(colors.bar_bg)
            .border(Color::TRANSPARENT, 0.0)
            .padding(2.0);

            let on_select_next = self.on_select.clone();
            let next_color = if can_next {
                colors.text
            } else {
                colors.text_secondary
            };
            let next_btn = button(
                icon(icons::CHEVRON_RIGHT)
                    .size(ICON_SIZE_SM)
                    .color(next_color)
                    .build(),
                move |state: &mut State| {
                    if can_next {
                        if let Some(ref cb) = on_select_next {
                            cb(state, active_idx + 1)
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                },
            )
            .disabled(!can_next)
            .background_color(colors.bar_bg)
            .border(Color::TRANSPARENT, 0.0)
            .padding(2.0);

            flex_row((scrollable_tabs, prev_btn, next_btn))
                .gap(4.px())
                .background_color(colors.bar_bg)
                .boxed()
        } else {
            flex_row((scrollable_tabs,))
                .background_color(colors.bar_bg)
                .boxed()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tabs::SimpleTab;

    #[test]
    fn nav_tab_bar_navigation() {
        let tabs = vec![
            SimpleTab::new("Tab 1"),
            SimpleTab::new("Tab 2"),
            SimpleTab::new("Tab 3"),
        ];

        let bar: NavTabBar<(), (), _, ()> = NavTabBar::new(&tabs, 0);
        assert!(!bar.can_go_prev());
        assert!(bar.can_go_next());

        let bar: NavTabBar<(), (), _, ()> = NavTabBar::new(&tabs, 1);
        assert!(bar.can_go_prev());
        assert!(bar.can_go_next());

        let bar: NavTabBar<(), (), _, ()> = NavTabBar::new(&tabs, 2);
        assert!(bar.can_go_prev());
        assert!(!bar.can_go_next());
    }

    #[test]
    fn nav_tab_bar_empty() {
        let tabs: Vec<SimpleTab> = vec![];
        let bar: NavTabBar<(), (), _, ()> = NavTabBar::new(&tabs, 0);
        assert!(!bar.can_go_prev());
        assert!(!bar.can_go_next());
    }

    #[test]
    fn nav_tab_padding_golden_ratio() {
        let (h, v) = NavTabBar::<(), (), SimpleTab, ()>::tab_padding();
        assert!((h / v - PHI).abs() < 0.1);
    }

    #[test]
    fn show_nav_buttons_default_off() {
        let tabs = vec![SimpleTab::new("Tab 1")];
        let bar: NavTabBar<(), (), _, ()> = NavTabBar::new(&tabs, 0);
        assert_eq!(bar.nav_button_mode, NavButtonMode::Never);
        assert!(!bar.should_show_nav());
    }

    #[test]
    fn show_nav_buttons_enabled() {
        let tabs = vec![SimpleTab::new("Tab 1")];
        let bar: NavTabBar<(), (), _, ()> = NavTabBar::new(&tabs, 0).show_nav_buttons(true);
        assert_eq!(bar.nav_button_mode, NavButtonMode::Always);
        assert!(bar.should_show_nav());
    }

    #[test]
    fn auto_nav_buttons_hides_for_few_tabs() {
        let tabs = vec![
            SimpleTab::new("Tab 1"),
            SimpleTab::new("Tab 2"),
        ];
        let bar: NavTabBar<(), (), _, ()> = NavTabBar::new(&tabs, 0).auto_nav_buttons(300.0);
        // Two short tabs should fit in 300px
        assert!(!bar.should_show_nav());
    }

    #[test]
    fn auto_nav_buttons_shows_for_many_tabs() {
        let tabs = vec![
            SimpleTab::new("Overview"),
            SimpleTab::new("Details"),
            SimpleTab::new("Settings"),
            SimpleTab::new("History"),
            SimpleTab::new("Advanced Options"),
        ];
        let bar: NavTabBar<(), (), _, ()> = NavTabBar::new(&tabs, 0).auto_nav_buttons(200.0);
        // Five tabs with longer names should exceed 200px
        assert!(bar.should_show_nav());
    }

    #[test]
    fn estimated_width_scales_with_text_size() {
        let tabs = vec![SimpleTab::new("Test")];
        let bar1: NavTabBar<(), (), _, ()> = NavTabBar::new(&tabs, 0).text_size(13.0);
        let bar2: NavTabBar<(), (), _, ()> = NavTabBar::new(&tabs, 0).text_size(26.0);
        // Larger text size should increase width
        assert!(bar2.estimated_tabs_width() > bar1.estimated_tabs_width());
    }

    #[test]
    fn corner_radius_default() {
        let tabs = vec![SimpleTab::new("Tab 1")];
        let bar: NavTabBar<(), (), _, ()> = NavTabBar::new(&tabs, 0);
        assert_eq!(bar.corner_radius, 4.0);
    }

    #[test]
    fn corner_radius_custom() {
        let tabs = vec![SimpleTab::new("Tab 1")];
        let bar: NavTabBar<(), (), _, ()> = NavTabBar::new(&tabs, 0).corner_radius(8.0);
        assert_eq!(bar.corner_radius, 8.0);
    }

    #[test]
    fn text_size_default() {
        let tabs = vec![SimpleTab::new("Tab 1")];
        let bar: NavTabBar<(), (), _, ()> = NavTabBar::new(&tabs, 0);
        assert_eq!(bar.text_size, 13.0);
    }

    #[test]
    fn text_size_custom() {
        let tabs = vec![SimpleTab::new("Tab 1")];
        let bar: NavTabBar<(), (), _, ()> = NavTabBar::new(&tabs, 0).text_size(16.0);
        assert_eq!(bar.text_size, 16.0);
    }
}
