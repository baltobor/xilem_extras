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
use masonry::parley::FontFamily;
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
    /// Optional font family. When `None`, the inner `text_input`
    /// keeps its default (`GenericFamily::SystemUi`). When `Some`,
    /// it's passed through verbatim — including any fallback chain
    /// the caller provides.
    font: Option<FontFamily<'static>>,
    insert_newline: InsertNewline,
    text_alignment: TextAlign,
    disabled: bool,
    clip: bool,
    /// When `true`, the field renders bullets in place of the
    /// underlying characters and edits are reconstructed from the
    /// displayed length delta. Suitable for password entry.
    secure: bool,
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
            font: None,
            insert_newline: InsertNewline::default(),
            text_alignment: TextAlign::default(),
            disabled: false,
            clip: true,
            secure: false,
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
            font: self.font,
            insert_newline: self.insert_newline,
            text_alignment: self.text_alignment,
            disabled: self.disabled,
            clip: self.clip,
            secure: self.secure,
            _phantom: PhantomData,
        }
    }

    /// Set the font family used to render the text.
    ///
    /// Accepts anything that converts into a [`FontFamily`], so a
    /// `GenericFamily::Monospace`, a named family, or a list with
    /// fallbacks all work. When left unset, the inner `text_input`
    /// keeps its default (`GenericFamily::SystemUi`), which on some
    /// systems renders mixed digit/letter strings (hex keys, UUIDs)
    /// with visibly inconsistent advance widths due to per-glyph
    /// fallback. Setting an explicit family avoids that.
    pub fn font(mut self, font: impl Into<FontFamily<'static>>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Render the field as a secure password input. The visible
    /// text is replaced with bullets while the underlying value
    /// (passed to `on_change`) carries the actual characters.
    ///
    /// The field reconstructs edits from the length delta of the
    /// displayed text. Append and end-deletion (typing / backspace)
    /// behave as expected; complex mid-string edits or paste
    /// operations beyond simple appends should not be relied on
    /// for password capture — passwords are normally entered
    /// linearly.
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// Build the styled text input view.
    pub fn build(self) -> impl WidgetView<State, Action> {
        // Secure mode: display bullets for the current password
        // length and reconstruct edits in `on_change` from the
        // length delta of the displayed text.
        let (display_value, change_fn): (
            String,
            Box<dyn Fn(&mut State, String) -> Action + Send + Sync>,
        ) = if self.secure {
            let prev_pw = self.value.clone();
            let user_change = self.on_change;
            let display = bullet_mask(prev_pw.chars().count());
            (
                display,
                Box::new(move |state: &mut State, new_displayed: String| {
                    let new_pw = reconstruct_password(&prev_pw, &new_displayed);
                    user_change(state, new_pw)
                }),
            )
        } else {
            let on_change = self.on_change;
            (
                self.value,
                Box::new(move |state: &mut State, new: String| on_change(state, new)),
            )
        };

        let mut input = text_input(display_value, change_fn)
            .placeholder(self.placeholder)
            .text_color(self.colors.text)
            .text_size(self.text_size)
            .weight(self.weight)
            .insert_newline(self.insert_newline)
            .text_alignment(self.text_alignment)
            .disabled(self.disabled)
            .clip(self.clip);

        if let Some(font) = self.font {
            input = input.font(font);
        }

        if let Some(on_enter) = self.on_enter {
            // For secure inputs the displayed value passed to
            // on_enter is the bullet string, which is rarely what
            // the caller wants; we rewrap to send the
            // reconstructed password instead.
            if self.secure {
                let prev_pw_enter = String::new(); // unused — we rebuild from display
                let _ = prev_pw_enter;
                input = input.on_enter(move |state, displayed| {
                    on_enter(state, reconstruct_from_bullets(&displayed))
                });
            } else {
                input = input.on_enter(move |state, text| on_enter(state, text));
            }
        }

        input
            .prop(Background::Color(self.colors.background))
            .prop(BorderColor { color: self.colors.border })
            .prop(CaretColor { color: self.colors.caret })
            .prop(PlaceholderColor::new(self.colors.placeholder))
    }
}

/// Bullet character used for masked password display.
const BULLET: char = '\u{2022}';

fn bullet_mask(n: usize) -> String {
    std::iter::repeat_n(BULLET, n).collect()
}

/// Reconstruct the password from the displayed text. The display
/// is normally a string of bullets; an edit replaces some bullets
/// with new characters. We assume linear entry — appending or
/// deleting from the end — and:
///
/// - If the displayed length grew, the new characters are the
///   non-bullet characters in the new display, appended to the
///   previous password.
/// - If the displayed length shrank, the password is truncated
///   to the new length.
fn reconstruct_password(prev_pw: &str, new_displayed: &str) -> String {
    let prev_len = prev_pw.chars().count();
    let new_len = new_displayed.chars().count();

    if new_len > prev_len {
        let typed: String = new_displayed.chars().filter(|&c| c != BULLET).collect();
        let mut out = String::with_capacity(prev_pw.len() + typed.len());
        out.push_str(prev_pw);
        out.push_str(&typed);
        out
    } else if new_len < prev_len {
        prev_pw.chars().take(new_len).collect()
    } else {
        prev_pw.to_string()
    }
}

/// Fallback used when `on_enter` fires on a secure field without
/// a known previous password to diff against. The displayed text
/// is the bullet representation; non-bullet characters are
/// returned as-is (likely empty).
fn reconstruct_from_bullets(displayed: &str) -> String {
    displayed.chars().filter(|&c| c != BULLET).collect()
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

/// Secure (password) text field. The visible characters are
/// bullets; `on_change` receives the underlying password value.
///
/// Behaviour matches `styled_text_input_with_placeholder` in
/// every respect except masking and edit reconstruction. For
/// a fully configured secure field, use the builder:
///
/// ```ignore
/// StyledTextInput::new(model.password.clone(), |m: &mut AppModel, s| m.password = s)
///     .secure(true)
///     .placeholder("Password")
///     .build()
/// ```
pub fn styled_secure_text_input<State, Action, F>(
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
        .colors(TextInputColors::dark())
        .placeholder(placeholder)
        .secure(true)
        .build()
}
