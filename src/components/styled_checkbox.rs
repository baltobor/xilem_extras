//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Styled Checkbox - Light mode checkbox for xilem
//!
//! Provides a checkbox with explicit light mode styling that works
//! regardless of system dark/light mode. Uses xilem's property system
//! to override theme defaults per-widget.

use xilem::view::checkbox;
use xilem::WidgetView;
use masonry::peniko::color::{AlphaColor, Srgb};
use masonry::peniko::Color;
use masonry::properties::{Background, BorderColor, CheckmarkColor};

/// Light mode checkbox colors
#[derive(Clone, Debug)]
pub struct CheckboxColors {
    /// Background color (default: white)
    pub background: AlphaColor<Srgb>,
    /// Border color (default: dark gray)
    pub border: AlphaColor<Srgb>,
    /// Checkmark color (default: black)
    pub checkmark: AlphaColor<Srgb>,
    /// Label text color (default: black)
    pub label: Color,
}

impl Default for CheckboxColors {
    fn default() -> Self {
        Self {
            background: AlphaColor::new([1.0, 1.0, 1.0, 1.0]),      // white
            border: AlphaColor::new([0.4, 0.4, 0.4, 1.0]),          // dark gray
            checkmark: AlphaColor::new([0.0, 0.0, 0.0, 1.0]),       // black
            label: Color::BLACK,
        }
    }
}

impl CheckboxColors {
    /// Create colors for light mode (white background, dark checkmark)
    pub fn light() -> Self {
        Self::default()
    }

    /// Create colors for dark mode (dark background, light checkmark)
    pub fn dark() -> Self {
        Self {
            background: AlphaColor::new([0.2, 0.2, 0.2, 1.0]),      // dark gray
            border: AlphaColor::new([0.5, 0.5, 0.5, 1.0]),          // medium gray
            checkmark: AlphaColor::new([1.0, 1.0, 1.0, 1.0]),       // white
            label: Color::WHITE,
        }
    }

    /// Create custom colors
    pub fn custom(
        background: impl Into<AlphaColor<Srgb>>,
        border: impl Into<AlphaColor<Srgb>>,
        checkmark: impl Into<AlphaColor<Srgb>>,
        label: Color,
    ) -> Self {
        Self {
            background: background.into(),
            border: border.into(),
            checkmark: checkmark.into(),
            label,
        }
    }
}

/// Creates a styled checkbox with explicit colors that override system theme.
///
/// This ensures the checkbox is visible regardless of system dark/light mode.
///
/// # Arguments
/// * `label` - Text label next to the checkbox
/// * `checked` - Current checked state
/// * `on_toggle` - Callback when checkbox is toggled
///
/// # Example
/// ```ignore
/// styled_checkbox(
///     "Enable feature",
///     model.feature_enabled,
///     |model: &mut AppModel, checked: bool| {
///         model.feature_enabled = checked;
///     }
/// )
/// ```
pub fn styled_checkbox<State, Action, F>(
    label: impl Into<masonry::core::ArcStr>,
    checked: bool,
    on_toggle: F,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
{
    let colors = CheckboxColors::default();
    checkbox(label, checked, on_toggle)
        .prop(Background::Color(colors.background))
        .prop(BorderColor { color: colors.border })
        .prop(CheckmarkColor { color: colors.checkmark })
}

/// Creates a styled checkbox with custom colors.
///
/// # Arguments
/// * `label` - Text label next to the checkbox
/// * `checked` - Current checked state
/// * `colors` - Custom color scheme
/// * `on_toggle` - Callback when checkbox is toggled
///
/// # Example
/// ```ignore
/// styled_checkbox_colored(
///     "Dark mode option",
///     model.dark_mode,
///     CheckboxColors::dark(),
///     |model: &mut AppModel, checked: bool| {
///         model.dark_mode = checked;
///     }
/// )
/// ```
pub fn styled_checkbox_colored<State, Action, F>(
    label: impl Into<masonry::core::ArcStr>,
    checked: bool,
    colors: CheckboxColors,
    on_toggle: F,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
{
    checkbox(label, checked, on_toggle)
        .prop(Background::Color(colors.background))
        .prop(BorderColor { color: colors.border })
        .prop(CheckmarkColor { color: colors.checkmark })
}
