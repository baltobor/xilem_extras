//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Boolean controls with explicit styling — checkmark, on/off
//! switch, and radio dot, all sharing the same `(label, value,
//! on_toggle)` shape.
//!
//! Internally these wrap xilem's own `checkbox`, `switch`, and
//! `radio_button` views. The `CheckboxStyle` dispatcher lets
//! callers swap styles without rewriting the surrounding view —
//! a settings/inspector form can pick "Switch" for some rows and
//! "Radio" for others without changing the call shape.

use xilem::view::{checkbox, flex_row, label, CrossAxisAlignment};

use super::switch_widget::synth_switch;
use super::radio_widget::synth_radio;
use xilem::style::Style;
use xilem::{AnyWidgetView, WidgetView};
use masonry::layout::AsUnit;
use masonry::peniko::color::{AlphaColor, Srgb};
use masonry::peniko::Color;
use masonry::properties::{Background, BorderColor, CheckmarkColor};

/// Visual style applied to a boolean control. Lets callers reuse
/// the same code path while presenting checkmarks, switches, or
/// radio dots depending on the form's needs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CheckboxStyle {
    /// Classic checkbox with a checkmark glyph.
    #[default]
    Classic,
    /// iOS / macOS-style on/off slider switch.
    Switch,
    /// Single radio dot (for boolean toggles inside groups that
    /// otherwise use radio buttons — e.g. macOS-style "inline"
    /// pickers).
    Radio,
}

/// Light mode checkbox colors — applies to the `Classic` style.
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

/// Creates a styled checkbox (classic checkmark) with explicit
/// colors that override system theme.
pub fn styled_checkbox<State, Action, F>(
    label_text: impl Into<masonry::core::ArcStr>,
    checked: bool,
    on_toggle: F,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
{
    styled_checkbox_colored(label_text, checked, CheckboxColors::default(), on_toggle)
}

/// Creates a styled checkbox (classic checkmark) with custom colors.
pub fn styled_checkbox_colored<State, Action, F>(
    label_text: impl Into<masonry::core::ArcStr>,
    checked: bool,
    colors: CheckboxColors,
    on_toggle: F,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
{
    let label_text: masonry::core::ArcStr = label_text.into();
    let label_color = colors.label;

    flex_row((
        checkbox("", checked, on_toggle)
            .prop(Background::Color(colors.background))
            .prop(BorderColor { color: colors.border })
            .prop(CheckmarkColor { color: colors.checkmark }),
        label(label_text).text_size(12.0).color(label_color),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Center)
    .gap(6.0_f64.px())
}

/// Labeled on/off switch — same call shape as `styled_checkbox`.
/// Wraps the synth-style switch with a trailing label.
///
/// Defaults to a light label colour suitable for dark themes.
/// Use [`styled_switch_colored`] to drive the label colour from
/// the surrounding theme (e.g. flipping with a dark/light mode
/// toggle).
pub fn styled_switch<State, Action, F>(
    label_text: impl Into<masonry::core::ArcStr>,
    on: bool,
    on_toggle: F,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
{
    styled_switch_colored(label_text, on, Color::WHITE, on_toggle)
}

/// Labeled on/off switch with an explicit label colour. Pair
/// with the gallery's dark/light mode flag (or any caller-driven
/// theme) so labels read on either background.
pub fn styled_switch_colored<State, Action, F>(
    label_text: impl Into<masonry::core::ArcStr>,
    on: bool,
    label_color: Color,
    on_toggle: F,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
{
    let label_text: masonry::core::ArcStr = label_text.into();
    flex_row((
        synth_switch(on, on_toggle),
        label(label_text).text_size(12.0).color(label_color),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Center)
    .gap(6.0_f64.px())
}

/// Labeled radio button — synth-style capsule + dot, click
/// toggles. Defaults to a light label colour; see
/// [`styled_radio_colored`] for theming.
pub fn styled_radio<State, Action, F>(
    label_text: impl Into<masonry::core::ArcStr>,
    checked: bool,
    on_toggle: F,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
{
    styled_radio_colored(label_text, checked, Color::WHITE, on_toggle)
}

/// Labeled radio with an explicit label colour, for callers that
/// drive their own dark/light theme.
pub fn styled_radio_colored<State, Action, F>(
    label_text: impl Into<masonry::core::ArcStr>,
    checked: bool,
    label_color: Color,
    on_toggle: F,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
{
    let label_text: masonry::core::ArcStr = label_text.into();
    flex_row((
        synth_radio(checked, on_toggle),
        label(label_text).text_size(12.0).color(label_color),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Center)
    .gap(6.0_f64.px())
}

/// Boolean control with a runtime-selectable visual style.
///
/// All three styles share the same `(label, value, on_toggle)`
/// signature, so callers can swap a row's style without changing
/// the surrounding code:
///
/// ```ignore
/// use xilem_extras::{styled_check, CheckboxStyle};
///
/// styled_check(CheckboxStyle::Switch, "Dark mode", model.dark_mode,
///     |m: &mut AppModel, v| m.dark_mode = v)
/// ```
///
/// Returns a boxed `AnyWidgetView` because the underlying view
/// types differ between styles.
pub fn styled_check<State, Action, F>(
    style: CheckboxStyle,
    label_text: impl Into<masonry::core::ArcStr>,
    checked: bool,
    on_toggle: F,
) -> Box<AnyWidgetView<State, Action>>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
{
    let label_text = label_text.into();
    match style {
        CheckboxStyle::Classic => Box::new(styled_checkbox(label_text, checked, on_toggle)),
        CheckboxStyle::Switch => Box::new(styled_switch(label_text, checked, on_toggle)),
        CheckboxStyle::Radio => Box::new(styled_radio(label_text, checked, on_toggle)),
    }
}

/// `styled_check` with explicit per-style colours so callers can
/// drive a dark/light theme at the surrounding scope.
///
/// `colors` is consumed by `Classic`; the other styles take only
/// the label colour from it.
pub fn styled_check_colored<State, Action, F>(
    style: CheckboxStyle,
    label_text: impl Into<masonry::core::ArcStr>,
    checked: bool,
    colors: CheckboxColors,
    on_toggle: F,
) -> Box<AnyWidgetView<State, Action>>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
{
    let label_text = label_text.into();
    let label_color = colors.label;
    match style {
        CheckboxStyle::Classic => {
            Box::new(styled_checkbox_colored(label_text, checked, colors, on_toggle))
        }
        CheckboxStyle::Switch => {
            Box::new(styled_switch_colored(label_text, checked, label_color, on_toggle))
        }
        CheckboxStyle::Radio => {
            Box::new(styled_radio_colored(label_text, checked, label_color, on_toggle))
        }
    }
}
