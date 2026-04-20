//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Styled Text Input - Light mode text input for xilem
//!
//! Provides a text input with explicit light mode styling that works
//! regardless of system dark/light mode. Uses xilem's property system
//! to override theme defaults per-widget.
//!
//! This solves the common issue where text inputs become unreadable
//! in dark mode (dark text on dark background).

use xilem::view::text_input;
use xilem::WidgetView;
use masonry::peniko::color::{AlphaColor, Srgb};
use masonry::peniko::Color;
use masonry::properties::{Background, BorderColor, CaretColor, PlaceholderColor};

/// Light mode text input colors
#[derive(Clone, Debug)]
pub struct TextInputColors {
    /// Background color (default: white)
    pub background: AlphaColor<Srgb>,
    /// Text color (default: black)
    pub text: Color,
    /// Border color (default: light gray)
    pub border: AlphaColor<Srgb>,
    /// Caret/cursor color (default: black)
    pub caret: AlphaColor<Srgb>,
    /// Placeholder text color (default: medium gray)
    pub placeholder: AlphaColor<Srgb>,
}

impl Default for TextInputColors {
    fn default() -> Self {
        Self {
            background: AlphaColor::new([1.0, 1.0, 1.0, 1.0]),      // white
            text: Color::BLACK,
            border: AlphaColor::new([0.7, 0.7, 0.7, 1.0]),          // light gray
            caret: AlphaColor::new([0.0, 0.0, 0.0, 1.0]),           // black
            placeholder: AlphaColor::new([0.5, 0.5, 0.5, 1.0]),     // medium gray
        }
    }
}

impl TextInputColors {
    /// Create colors for light mode (white background, dark text)
    pub fn light() -> Self {
        Self::default()
    }

    /// Create colors for dark mode (dark background, light text)
    pub fn dark() -> Self {
        Self {
            background: AlphaColor::new([0.15, 0.15, 0.15, 1.0]),   // dark gray
            text: Color::WHITE,
            border: AlphaColor::new([0.4, 0.4, 0.4, 1.0]),          // medium gray
            caret: AlphaColor::new([1.0, 1.0, 1.0, 1.0]),           // white
            placeholder: AlphaColor::new([0.6, 0.6, 0.6, 1.0]),     // light gray
        }
    }

    /// Create custom colors
    pub fn custom(
        background: impl Into<AlphaColor<Srgb>>,
        text: Color,
        border: impl Into<AlphaColor<Srgb>>,
        caret: impl Into<AlphaColor<Srgb>>,
        placeholder: impl Into<AlphaColor<Srgb>>,
    ) -> Self {
        Self {
            background: background.into(),
            text,
            border: border.into(),
            caret: caret.into(),
            placeholder: placeholder.into(),
        }
    }
}

/// Creates a styled text input with explicit colors that override system theme.
///
/// This ensures the text input is readable regardless of system dark/light mode.
///
/// # Arguments
/// * `value` - Current text value
/// * `on_change` - Callback when text changes
///
/// # Example
/// ```ignore
/// styled_text_input(
///     model.my_input.clone(),
///     |model: &mut AppModel, new_value: String| {
///         model.my_input = new_value;
///     }
/// )
/// ```
pub fn styled_text_input<State, Action, F>(
    value: String,
    on_change: F,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, String) -> Action + Send + Sync + 'static,
{
    let colors = TextInputColors::default();
    text_input(value, on_change)
        .text_color(colors.text)
        .prop(Background::Color(colors.background))
        .prop(BorderColor { color: colors.border })
        .prop(CaretColor { color: colors.caret })
        .prop(PlaceholderColor::new(colors.placeholder))
}

/// Creates a styled text input with placeholder text.
///
/// # Arguments
/// * `value` - Current text value
/// * `placeholder` - Placeholder text shown when empty
/// * `on_change` - Callback when text changes
///
/// # Example
/// ```ignore
/// styled_text_input_with_placeholder(
///     model.search.clone(),
///     "Search...",
///     |model: &mut AppModel, new_value: String| {
///         model.search = new_value;
///     }
/// )
/// ```
pub fn styled_text_input_with_placeholder<State, Action, F>(
    value: String,
    placeholder: &str,
    on_change: F,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, String) -> Action + Send + Sync + 'static,
{
    let colors = TextInputColors::default();
    text_input(value, on_change)
        .placeholder(placeholder)
        .text_color(colors.text)
        .prop(Background::Color(colors.background))
        .prop(BorderColor { color: colors.border })
        .prop(CaretColor { color: colors.caret })
        .prop(PlaceholderColor::new(colors.placeholder))
}

/// Creates a styled text input with custom colors.
///
/// # Arguments
/// * `value` - Current text value
/// * `placeholder` - Placeholder text (use empty string for none)
/// * `colors` - Custom color scheme
/// * `on_change` - Callback when text changes
///
/// # Example
/// ```ignore
/// styled_text_input_colored(
///     model.input.clone(),
///     "Enter value...",
///     TextInputColors::dark(),
///     |model: &mut AppModel, new_value: String| {
///         model.input = new_value;
///     }
/// )
/// ```
pub fn styled_text_input_colored<State, Action, F>(
    value: String,
    placeholder: &str,
    colors: TextInputColors,
    on_change: F,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, String) -> Action + Send + Sync + 'static,
{
    // Note: placeholder() must be called before prop() due to type system constraints
    text_input(value, on_change)
        .placeholder(placeholder)
        .text_color(colors.text)
        .prop(Background::Color(colors.background))
        .prop(BorderColor { color: colors.border })
        .prop(CaretColor { color: colors.caret })
        .prop(PlaceholderColor::new(colors.placeholder))
}
