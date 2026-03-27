//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

pub mod icon;
mod row_button;
mod disclosure;
pub mod svg_icon;

#[cfg(feature = "rust-logos")]
pub mod rust_logos;

pub use icon::{icon, icon_sm, icon_md, icon_lg, Icon};
pub use row_button::{row_button, row_button_with_clicks, row_button_with_modifiers, row_button_with_press, RowButtonView, RowButtonPress};
pub use disclosure::{disclosure, Disclosure};
pub use svg_icon::{SvgIcon, SvgIconView, SvgIconWidget, svg_icon, ScaleMode};

#[cfg(feature = "rust-logos")]
pub use rust_logos::{rust_logo, rust_gear, rust_logo_complete, ferris};
