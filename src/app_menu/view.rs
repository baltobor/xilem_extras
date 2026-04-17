//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Xilem View helpers for application menu bar.
//!
//! This module provides convenience functions for building application menu bars
//! using the existing menu_button infrastructure.

use xilem::masonry::peniko::Color;
use xilem::style::{Padding, Style};
use xilem::view::{flex_col, label, CrossAxisAlignment};
use xilem::WidgetView;

use super::builder::MenuBarBuilder;
use crate::menu_button::DEFAULT_ITEM_HEIGHT;

/// Default menu bar background color.
const MENU_BAR_BG: Color = Color::from_rgb8(45, 43, 40);
/// Default menu text color.
const MENU_TEXT_COLOR: Color = Color::from_rgb8(220, 218, 214);
/// Font size for menu labels.
const MENU_TEXT_SIZE: f32 = (DEFAULT_ITEM_HEIGHT * 0.43) as f32;
/// Vertical padding.
const MENU_PADDING_V: f64 = (DEFAULT_ITEM_HEIGHT - MENU_TEXT_SIZE as f64) / 2.0;
/// Horizontal padding.
const MENU_PADDING_H: f64 = 10.0;

/// Creates an application menu bar view.
///
/// This is a convenience wrapper that returns a view you can use directly in your app.
/// For full control over styling, use `MenuBarBuilder` directly.
///
/// **Note:** The builder API (app_menu_bar) defines menu structure but currently
/// requires manual conversion to menu_button widgets. For native menus on macOS/Windows,
/// use the muda backend directly with the builder.
///
/// # Example
///
/// ```ignore
/// use xilem_extras::app_menu::{app_menu_bar, Key, CMD, SHIFT};
///
/// // Build a menu bar (returns the builder for inspection)
/// let menu_def = app_menu_bar(|m| m
///     .menu("File", |m| m
///         .item("New", |s: &mut AppModel| s.new_file())
///             .shortcut(CMD + Key::N)
///         .item("Quit", |s| s.quit())
///             .shortcut(CMD + Key::Q)
///     )
/// );
///
/// // For actual rendering, use menu_button directly:
/// menu_button(label("File"), (
///     menu_item("New", |s| s.new_file()),
///     separator(),
///     menu_item("Quit", |s| s.quit()),
/// ))
/// ```
pub fn app_menu_bar<State, Action, F>(
    builder: F,
) -> MenuBarBuilder<State, Action>
where
    State: 'static,
    Action: 'static,
    F: FnOnce(MenuBarBuilder<State, Action>) -> MenuBarBuilder<State, Action>,
{
    builder(MenuBarBuilder::new())
}

/// Xilem View wrapper for application menu bar.
///
/// **Note:** This is a placeholder. For now, use menu_button directly
/// for the xilem view, and the builder API for muda integration.
#[must_use = "View values do nothing unless provided to Xilem."]
pub struct AppMenuBarView<State, Action> {
    menu_bar: MenuBarBuilder<State, Action>,
    _bg_color: Color,
    _text_color: Color,
}

impl<State, Action> AppMenuBarView<State, Action> {
    /// Create from a builder.
    pub fn new(menu_bar: MenuBarBuilder<State, Action>) -> Self {
        Self {
            menu_bar,
            _bg_color: MENU_BAR_BG,
            _text_color: MENU_TEXT_COLOR,
        }
    }

    /// Set custom background color.
    pub fn background_color(mut self, color: Color) -> Self {
        self._bg_color = color;
        self
    }

    /// Set custom text color.
    pub fn text_color(mut self, color: Color) -> Self {
        self._text_color = color;
        self
    }

    /// Access the menu bar builder for native menu integration.
    pub fn builder(&self) -> &MenuBarBuilder<State, Action> {
        &self.menu_bar
    }
}

/// Wraps content with an application menu bar at the top.
///
/// This is a convenience function that takes a menu builder and content,
/// returning a composed view with the menu bar on top.
///
/// **Note:** This currently creates an empty placeholder.
/// For actual menu bars, build them using menu_button directly.
///
/// # Example
///
/// ```ignore
/// use xilem_extras::app_menu::{with_app_menu, Key, CMD};
///
/// fn app_view(model: &mut AppModel) -> impl WidgetView<AppModel> {
///     with_app_menu(
///         |m| m.menu("File", |m| m.item("Quit", |s| s.quit())),
///         main_content(model),
///     )
/// }
/// ```
pub fn with_app_menu<State, Action, F, V>(
    menu_builder: F,
    content: V,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    F: FnOnce(MenuBarBuilder<State, Action>) -> MenuBarBuilder<State, Action>,
    V: WidgetView<State, Action>,
{
    // Build the menu definition (for future use with muda)
    let _menu_def = menu_builder(MenuBarBuilder::new());

    // For now, just return the content
    // In a full implementation, this would wrap content with a menu bar view
    flex_col((content,))
        .cross_axis_alignment(CrossAxisAlignment::Stretch)
}

/// Creates a styled label for menu bar buttons.
///
/// Use this helper when building menu bars with menu_button.
pub fn menu_bar_label<'a, State: 'static, Action: 'static>(text: &'a str) -> impl WidgetView<State, Action> + use<'a, State, Action> {
    label(text.to_string())
        .text_size(MENU_TEXT_SIZE)
        .color(MENU_TEXT_COLOR)
        .padding(Padding {
            top: MENU_PADDING_V,
            bottom: MENU_PADDING_V,
            left: MENU_PADDING_H,
            right: MENU_PADDING_H,
        })
}
