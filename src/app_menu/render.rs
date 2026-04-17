//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Render helpers for application menu bar.
//!
//! Due to Xilem's type system, rendering menu bars from a builder requires
//! a macro or manual construction. This module provides helper constants.

use xilem::masonry::peniko::Color;

use crate::menu_button::DEFAULT_ITEM_HEIGHT;

/// Default menu bar background color.
pub const MENU_BAR_BG: Color = Color::from_rgb8(45, 43, 40);
/// Default menu text color.
pub const MENU_TEXT_COLOR: Color = Color::from_rgb8(220, 218, 214);
/// Font size for menu labels.
pub const MENU_TEXT_SIZE: f32 = (DEFAULT_ITEM_HEIGHT * 0.43) as f32;
/// Vertical padding.
pub const MENU_PADDING_V: f64 = (DEFAULT_ITEM_HEIGHT - MENU_TEXT_SIZE as f64) / 2.0;
/// Horizontal padding.
pub const MENU_PADDING_H: f64 = 10.0;

/// Placeholder - actual menu bar rendering requires a macro or manual construction.
///
/// For Linux menu bars, use `menu_button` directly. For example:
///
/// ```ignore
/// use xilem_extras::{menu_button, menu_item, separator};
/// use xilem::view::flex_row;
///
/// fn linux_menu_bar(model: &mut AppModel) -> impl WidgetView<AppModel> {
///     flex_row((
///         menu_button(
///             label("File"),
///             (
///                 menu_item("New", |m: &mut AppModel| m.new_file()),
///                 separator(),
///                 menu_item("Quit", |m| m.quit()),
///             ),
///         ),
///         menu_button(
///             label("Edit"),
///             (
///                 menu_item("Undo", |m: &mut AppModel| m.undo()),
///                 menu_item("Redo", |m| m.redo()),
///             ),
///         ),
///     ))
/// }
/// ```
pub fn render_menu_bar() {
    // Placeholder - see documentation above
}
