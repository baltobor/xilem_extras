//! Xilem_masonry view wrappers for components.

pub mod click_interceptor;
pub mod clipped;
pub mod group_box;
pub mod param_selector;
pub mod radio_widget;
pub mod row_button;
pub mod rust_logos;
pub mod svg_icon;
pub mod switch_widget;

pub use click_interceptor::{ClickInterceptorView, click_interceptor};
pub use clipped::{ClippedView, clipped};
pub use group_box::{GroupBoxView, group_box};
pub use param_selector::{ParamSelectorView, param_selector};
pub use radio_widget::{SynthRadio, synth_radio};
pub use row_button::{
    RowButtonView, row_button, row_button_with_clicks, row_button_with_modifiers,
    row_button_with_press,
};
pub use svg_icon::{SvgIconView, svg_icon};
pub use switch_widget::{SynthSwitch, synth_switch};
