//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Form container, sections, and labeled rows.

use masonry::core::ArcStr;
use masonry::layout::{AsUnit, Length};
use xilem::masonry::peniko::Color;
use xilem::style::Style;
use xilem::view::{flex_col, flex_row, label, CrossAxisAlignment, FlexExt, FlexSequence};
use xilem::WidgetView;

use crate::components::{group_box, styled_check, styled_check_colored, CheckboxStyle};
use crate::theme::Theme;

/// Vertical gap between rows / sections inside a form.
const FORM_GAP: f64 = 10.0;
/// Inner padding of the form chrome.
const FORM_PADDING: f64 = 12.0;
/// Default form background. Neutral dark; suits the gallery theme
/// and stays readable next to other panels.
const FORM_BG: Color = Color::from_rgb8(0x22, 0x20, 0x1D);
/// Default label colour. Form rows tend to need a softer text
/// colour than primary content; sections override for headers.
const FORM_LABEL: Color = Color::from_rgb8(0xDC, 0xDA, 0xD6);

/// Wrap a children sequence in a form-styled vertical stack.
///
/// `form((row1, row2, row3))` accepts the same tuple shape as
/// `flex_col(...)` — swap one for the other freely. The form
/// adds consistent padding, vertical gap and a subtle background
/// so the controls visually group together.
///
/// # Example
///
/// ```ignore
/// use xilem_extras::{form, form_section, form_toggle, CheckboxStyle};
///
/// form((
///     form_section("Notifications", (
///         form_toggle("Play notification sounds",
///             model.play_sounds,
///             |m: &mut AppModel, v| m.play_sounds = v),
///         form_toggle("Send read receipts",
///             model.read_receipts,
///             |m, v| m.read_receipts = v),
///     )),
///     form_section("Display", (
///         form_toggle("Dark mode",
///             model.dark_mode,
///             |m, v| m.dark_mode = v),
///     )),
/// ))
/// ```
pub fn form<State, Action, Seq>(
    sequence: Seq,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    Seq: FlexSequence<State, Action> + Send + Sync + 'static,
{
    flex_col(sequence)
        .cross_axis_alignment(CrossAxisAlignment::Stretch)
        .gap(FORM_GAP.px())
        .padding(Length::px(FORM_PADDING))
        .background_color(FORM_BG)
        .corner_radius(Length::px(8.0))
}

// MARK: - Form section

/// A section inside a form, drawn as a [`crate::group_box`] with
/// the given header. Mirrors SwiftUI's `Section(header: Text(...))`.
///
/// Children follow the same tuple convention as [`form`].
pub fn form_section<State, Action, Seq>(
    header: impl Into<String>,
    content: Seq,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    Seq: FlexSequence<State, Action> + Send + Sync + 'static,
{
    let inner = flex_col(content)
        .cross_axis_alignment(CrossAxisAlignment::Stretch)
        .gap(6.0_f64.px());
    group_box(header, inner)
}

// MARK: - Form row

/// A `label : control` row, with the label on the left and the
/// control on the right. Mirrors SwiftUI's
/// `Toggle("Play sounds", isOn: $...)` pattern: the label name
/// is the row identity, the control is the editable value.
///
/// The label flexes (`flex 1.0`) so multi-row forms align their
/// controls down the right edge.
pub fn form_row<State, Action, V>(
    label_text: impl Into<ArcStr>,
    control: V,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    V: WidgetView<State, Action> + 'static,
{
    let label_text = label_text.into();
    flex_row((
        label(label_text)
            .text_size(12.0)
            .color(FORM_LABEL)
            .flex(1.0),
        control,
    ))
    .cross_axis_alignment(CrossAxisAlignment::Center)
    .gap(8.0_f64.px())
}

// MARK: - Form controls (label + boolean control)

/// Labeled toggle. Equivalent to SwiftUI's
/// `Toggle("Label", isOn: $value)` and renders as an on/off
/// switch by default.
pub fn form_toggle<State, Action, F>(
    label_text: impl Into<ArcStr>,
    on: bool,
    on_toggle: F,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
{
    let label_text = label_text.into();
    form_row(label_text, styled_check(CheckboxStyle::Switch, "", on, on_toggle))
}

/// Labeled checkbox using the classic checkmark style.
pub fn form_checkbox<State, Action, F>(
    label_text: impl Into<ArcStr>,
    checked: bool,
    on_toggle: F,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
{
    let label_text = label_text.into();
    form_row(label_text, styled_check(CheckboxStyle::Classic, "", checked, on_toggle))
}

/// Labeled radio button. Like SwiftUI's macOS `.pickerStyle(.inline)`
/// it renders one option of a radio group; the caller is
/// responsible for clearing the previously selected radio in the
/// `on_toggle` callback.
pub fn form_radio<State, Action, F>(
    label_text: impl Into<ArcStr>,
    checked: bool,
    on_toggle: F,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
{
    // The radio_button widget owns its own label, so we use it
    // directly rather than going through form_row — that keeps
    // the dot/label spacing native.
    styled_check(CheckboxStyle::Radio, label_text, checked, on_toggle)
}

// MARK: - Themed variants
//
// Existing functions above keep their default-dark behaviour
// unchanged so callers using `form((..))` etc. don't need to
// migrate. The `_themed` family below opts in to a [`Theme`]
// (light or dark) so the gallery — and any caller driving a
// dark/bright toggle — can hand one [`Theme`] down through the
// tree and have every form widget paint itself accordingly.

/// Themed [`form`]. Picks the form background from
/// [`Theme::bg`].
pub fn form_themed<State, Action, Seq>(
    sequence: Seq,
    theme: Theme,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    Seq: FlexSequence<State, Action> + Send + Sync + 'static,
{
    flex_col(sequence)
        .cross_axis_alignment(CrossAxisAlignment::Stretch)
        .gap(FORM_GAP.px())
        .padding(Length::px(FORM_PADDING))
        .background_color(theme.bg())
        .corner_radius(Length::px(8.0))
}

/// Themed [`form_section`]. The group-box tint follows
/// [`Theme::section_bg`]; the auto-contrast header label keeps
/// its readability on either palette.
pub fn form_section_themed<State, Action, Seq>(
    header: impl Into<String>,
    content: Seq,
    theme: Theme,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    Seq: FlexSequence<State, Action> + Send + Sync + 'static,
{
    let inner = flex_col(content)
        .cross_axis_alignment(CrossAxisAlignment::Stretch)
        .gap(6.0_f64.px());
    group_box(header, inner).tint(theme.section_bg())
}

/// Themed [`form_row`]. Label colour follows [`Theme::text`].
pub fn form_row_themed<State, Action, V>(
    label_text: impl Into<ArcStr>,
    control: V,
    theme: Theme,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    V: WidgetView<State, Action> + 'static,
{
    let label_text = label_text.into();
    flex_row((
        label(label_text)
            .text_size(12.0)
            .color(theme.text())
            .flex(1.0),
        control,
    ))
    .cross_axis_alignment(CrossAxisAlignment::Center)
    .gap(8.0_f64.px())
}

/// Themed [`form_toggle`].
pub fn form_toggle_themed<State, Action, F>(
    label_text: impl Into<ArcStr>,
    on: bool,
    theme: Theme,
    on_toggle: F,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
{
    let label_text = label_text.into();
    form_row_themed(
        label_text,
        styled_check_colored(
            CheckboxStyle::Switch,
            "",
            on,
            theme.checkbox_colors(),
            on_toggle,
        ),
        theme,
    )
}

/// Themed [`form_checkbox`].
pub fn form_checkbox_themed<State, Action, F>(
    label_text: impl Into<ArcStr>,
    checked: bool,
    theme: Theme,
    on_toggle: F,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
{
    let label_text = label_text.into();
    form_row_themed(
        label_text,
        styled_check_colored(
            CheckboxStyle::Classic,
            "",
            checked,
            theme.checkbox_colors(),
            on_toggle,
        ),
        theme,
    )
}

/// Themed [`form_radio`].
pub fn form_radio_themed<State, Action, F>(
    label_text: impl Into<ArcStr>,
    checked: bool,
    theme: Theme,
    on_toggle: F,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
{
    styled_check_colored(
        CheckboxStyle::Radio,
        label_text,
        checked,
        theme.checkbox_colors(),
        on_toggle,
    )
}
