//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Time picker widget module.
//!
//! Provides a simple time picker for hour:minute selection with:
//! - Stepper buttons for hours (0-23)
//! - Stepper buttons for minutes (0-59, configurable step)

mod widget;
mod view;

pub use widget::{TimePickerWidget, TimeAction};
pub use view::{time_picker, TimePickerView};
