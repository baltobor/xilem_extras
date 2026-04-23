//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Styled Text Input - Text input with explicit light/dark mode styling
//!
//! Provides text inputs with explicit color styling that works regardless of
//! system dark/light mode. Uses xilem's property system to override theme defaults.
//!
//! ## Features
//!
//! - Light and dark mode color presets
//! - Custom color support
//! - Full support for `.on_enter()` callback
//! - Placeholder text support
//! - Builder pattern for configuration
//!
//! ## TODO: Programmatic Focus Support
//!
//! Add SwiftUI-like `@FocusState` support for programmatic focus control.
//! This would allow patterns like:
//!
//! ```ignore
//! StyledTextInput::new(value, on_change)
//!     .focused(model.search_focus_requested)
//!     .on_focus_change(|model, focused| model.search_focus_requested = false)
//!     .build()
//! ```
//!
//! Implementation requires a custom wrapper widget because `request_focus()` is only
//! available on `EventCtx`/`ActionCtx` (widget event handlers), not on `MutateCtx`
//! (what views access during rebuild). The wrapper widget would need to:
//! 1. Track a `focus_requested` flag
//! 2. Call `ctx.request_focus()` during pointer/text events when the flag is set
//! 3. Emit an action when focus changes so the model can be updated

use std::marker::PhantomData;

use masonry::core::ArcStr;
use masonry::parley::style::FontWeight;
use masonry::peniko::color::{AlphaColor, Srgb};
use masonry::peniko::Color;
use masonry::properties::{Background, BorderColor, CaretColor, PlaceholderColor};

use xilem::view::text_input;
use xilem::WidgetView;
use xilem::{InsertNewline, TextAlign};

/// Text input color scheme.
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
        Self::light()
    }
}

impl TextInputColors {
    /// Create colors for light mode (white background, dark text)
    pub fn light() -> Self {
        Self {
            background: AlphaColor::new([1.0, 1.0, 1.0, 1.0]),      // white
            text: Color::BLACK,
            border: AlphaColor::new([0.7, 0.7, 0.7, 1.0]),          // light gray
            caret: AlphaColor::new([0.0, 0.0, 0.0, 1.0]),           // black
            placeholder: AlphaColor::new([0.5, 0.5, 0.5, 1.0]),     // medium gray
        }
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

type Callback<State, Action> = Box<dyn Fn(&mut State, String) -> Action + Send + Sync + 'static>;

/// A styled text input builder with full configuration options.
///
/// This builder provides:
/// - Explicit light/dark mode colors
/// - Placeholder text
/// - `.on_enter()` callback for submit handling
/// - Text size and weight configuration
/// - Disabled state
///
/// # Example
///
/// ```ignore
/// use xilem_extras::{StyledTextInput, TextInputColors};
///
/// StyledTextInput::new(
///     model.search.clone(),
///     |model: &mut AppModel, val: String| model.search = val,
/// )
/// .colors(TextInputColors::dark())
/// .placeholder("Search...")
/// .on_enter(|model: &mut AppModel, val: String| model.do_search())
/// .build()
/// ```
#[must_use = "StyledTextInput does nothing until .build() is called"]
pub struct StyledTextInput<State, Action, F, E = ()> {
    value: String,
    on_change: F,
    on_enter: Option<Callback<State, Action>>,
    colors: TextInputColors,
    placeholder: ArcStr,
    text_size: f32,
    weight: FontWeight,
    insert_newline: InsertNewline,
    text_alignment: TextAlign,
    disabled: bool,
    clip: bool,
    _phantom: PhantomData<(State, Action, E)>,
}

impl<State, Action, F> StyledTextInput<State, Action, F, ()>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, String) -> Action + Send + Sync + 'static,
{
    /// Create a new styled text input builder.
    ///
    /// # Arguments
    /// * `value` - Current text value
    /// * `on_change` - Callback when text changes
    pub fn new(value: String, on_change: F) -> Self {
        Self {
            value,
            on_change,
            on_enter: None,
            colors: TextInputColors::dark(),
            placeholder: ArcStr::default(),
            text_size: 13.0,
            weight: FontWeight::NORMAL,
            insert_newline: InsertNewline::default(),
            text_alignment: TextAlign::default(),
            disabled: false,
            clip: true,
            _phantom: PhantomData,
        }
    }
}

impl<State, Action, F, E> StyledTextInput<State, Action, F, E>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, String) -> Action + Send + Sync + 'static,
{
    /// Set the color scheme.
    pub fn colors(mut self, colors: TextInputColors) -> Self {
        self.colors = colors;
        self
    }

    /// Set placeholder text shown when input is empty.
    pub fn placeholder(mut self, placeholder: impl Into<ArcStr>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set the text size.
    pub fn text_size(mut self, size: f32) -> Self {
        self.text_size = size;
        self
    }

    /// Set the font weight.
    pub fn weight(mut self, weight: FontWeight) -> Self {
        self.weight = weight;
        self
    }

    /// Set how Enter key is handled.
    ///
    /// Default is `InsertNewline::Never` (single-line mode).
    pub fn insert_newline(mut self, insert_newline: InsertNewline) -> Self {
        self.insert_newline = insert_newline;
        self
    }

    /// Set text alignment.
    pub fn text_alignment(mut self, alignment: TextAlign) -> Self {
        self.text_alignment = alignment;
        self
    }

    /// Set disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set whether text is clipped when it overflows.
    pub fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    /// Set callback for Enter key press (submit action).
    ///
    /// This is only called when `insert_newline` is not `OnEnter`.
    pub fn on_enter<G>(
        self,
        on_enter: G,
    ) -> StyledTextInput<State, Action, F, Callback<State, Action>>
    where
        G: Fn(&mut State, String) -> Action + Send + Sync + 'static,
    {
        StyledTextInput {
            value: self.value,
            on_change: self.on_change,
            on_enter: Some(Box::new(on_enter)),
            colors: self.colors,
            placeholder: self.placeholder,
            text_size: self.text_size,
            weight: self.weight,
            insert_newline: self.insert_newline,
            text_alignment: self.text_alignment,
            disabled: self.disabled,
            clip: self.clip,
            _phantom: PhantomData,
        }
    }

    /// Build the styled text input view.
    pub fn build(self) -> impl WidgetView<State, Action> {
        let mut input = text_input(self.value, self.on_change)
            .placeholder(self.placeholder)
            .text_color(self.colors.text)
            .text_size(self.text_size)
            .weight(self.weight)
            .insert_newline(self.insert_newline)
            .text_alignment(self.text_alignment)
            .disabled(self.disabled)
            .clip(self.clip);

        if let Some(on_enter) = self.on_enter {
            input = input.on_enter(move |state, text| on_enter(state, text));
        }

        input
            .prop(Background::Color(self.colors.background))
            .prop(BorderColor { color: self.colors.border })
            .prop(CaretColor { color: self.colors.caret })
            .prop(PlaceholderColor::new(self.colors.placeholder))
    }
}

// Convenience functions for backwards compatibility

/// Creates a styled text input with default light mode colors.
///
/// For more control, use [`StyledTextInput::new()`] builder.
pub fn styled_text_input<State, Action, F>(
    value: String,
    on_change: F,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, String) -> Action + Send + Sync + 'static,
{
    StyledTextInput::new(value, on_change)
        .colors(TextInputColors::light())
        .build()
}

/// Creates a styled text input with placeholder text.
///
/// For more control, use [`StyledTextInput::new()`] builder.
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
    StyledTextInput::new(value, on_change)
        .colors(TextInputColors::light())
        .placeholder(placeholder)
        .build()
}

/// Creates a styled text input with custom colors.
///
/// For more control, use [`StyledTextInput::new()`] builder.
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
    StyledTextInput::new(value, on_change)
        .colors(colors)
        .placeholder(placeholder)
        .build()
}
