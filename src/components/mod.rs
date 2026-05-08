//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

mod row_button;
mod disclosure;
mod click_interceptor;
mod clipped;
mod styled_text_input;
mod styled_checkbox;
mod group_box;
mod switch_widget;
mod radio_widget;
mod param_selector;
pub mod svg_icon;

#[cfg(feature = "rust-logos")]
pub mod rust_logos;

pub use row_button::{row_button, row_button_with_clicks, row_button_with_modifiers, row_button_with_press, RowButtonView, RowButtonPress};
pub use disclosure::{disclosure, Disclosure};
pub use click_interceptor::{click_interceptor, ClickInterceptorView, ClickInterceptorWidget};
pub use clipped::{clipped, ClippedView, ClippedWidget};
pub use svg_icon::{SvgIcon, SvgIconView, SvgIconWidget, svg_icon, ScaleMode};
pub use styled_text_input::{
    styled_text_input,
    styled_text_input_with_placeholder,
    styled_text_input_colored,
    styled_secure_text_input,
    TextInputColors,
    StyledTextInput,
};
pub use styled_checkbox::{
    styled_checkbox,
    styled_checkbox_colored,
    styled_switch,
    styled_switch_colored,
    styled_radio,
    styled_radio_colored,
    styled_check,
    styled_check_colored,
    CheckboxColors,
    CheckboxStyle,
};
pub use group_box::{
    group_box,
    GroupBox,
    GroupBoxView,
    inverse_contrast_color,
};
pub use switch_widget::{
    synth_switch,
    SwitchWidget,
    SwitchToggled,
    SynthSwitch,
};
pub use radio_widget::{
    synth_radio,
    RadioWidget,
    RadioToggled,
    SynthRadio,
};
pub use param_selector::{
    param_selector,
    LabelAlign,
    ParamSelectorWidget,
    ParamSelectorView,
};

#[cfg(feature = "rust-logos")]
pub use rust_logos::{rust_logo, rust_gear, rust_logo_complete, ferris};
