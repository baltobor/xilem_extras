//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

mod disclosure;
mod styled_checkbox;
mod styled_text_input;

pub use disclosure::*;
pub use styled_checkbox::*;
pub use styled_text_input::*;

pub use crate::masonry::components::{
    ClickInterceptorWidget, ClippedWidget, GroupBox, LabelAlign, ParamSelectorWidget, RadioToggled,
    RadioWidget, RowButton, RowButtonPress, ScaleMode, SvgIcon, SvgIconWidget, SwitchToggled,
    SwitchWidget, inverse_contrast_color,
};

pub use crate::xilem_masonry::components::{
    ClickInterceptorView, ClippedView, GroupBoxView, ParamSelectorView, RowButtonView, SvgIconView,
    SynthRadio, SynthSwitch, click_interceptor, clipped, group_box, param_selector, row_button,
    row_button_with_clicks, row_button_with_modifiers, row_button_with_press, svg_icon, synth_radio,
    synth_switch,
};

#[cfg(feature = "rust-logos")]
pub use crate::xilem_masonry::components::rust_logos::{ferris, rust_gear, rust_logo, rust_logo_complete};
