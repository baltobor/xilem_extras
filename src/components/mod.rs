//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

mod click_interceptor;
mod clipped;
mod disclosure;
mod group_box;
mod param_selector;
mod radio_widget;
mod row_button;
mod styled_checkbox;
mod styled_text_input;
pub mod svg_icon;
mod switch_widget;

#[cfg(feature = "rust-logos")]
pub mod rust_logos;

pub use click_interceptor::{ClickInterceptorView, ClickInterceptorWidget, click_interceptor};
pub use clipped::{ClippedView, ClippedWidget, clipped};
pub use disclosure::{Disclosure, disclosure};
pub use group_box::{GroupBox, GroupBoxView, group_box, inverse_contrast_color};
pub use param_selector::{LabelAlign, ParamSelectorView, ParamSelectorWidget, param_selector};
pub use radio_widget::{RadioToggled, RadioWidget, SynthRadio, synth_radio};
pub use row_button::{
    RowButtonPress, RowButtonView, row_button, row_button_with_clicks, row_button_with_modifiers,
    row_button_with_press,
};
pub use styled_checkbox::{
    CheckboxColors, CheckboxStyle, styled_check, styled_check_colored, styled_checkbox,
    styled_checkbox_colored, styled_radio, styled_radio_colored, styled_switch,
    styled_switch_colored,
};
pub use styled_text_input::{
    StyledTextInput, TextInputColors, styled_secure_text_input, styled_text_input,
    styled_text_input_colored, styled_text_input_with_placeholder,
};
pub use svg_icon::{ScaleMode, SvgIcon, SvgIconView, SvgIconWidget, svg_icon};
pub use switch_widget::{SwitchToggled, SwitchWidget, SynthSwitch, synth_switch};

#[cfg(feature = "rust-logos")]
pub use rust_logos::{ferris, rust_gear, rust_logo, rust_logo_complete};
