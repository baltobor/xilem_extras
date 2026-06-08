//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Calendar picker widget module.
//!
//! Provides an date picker with:
//! - Month/year header with navigation arrows
//! - Weekday headers (Mo Tu We Th Fr Sa Su)
//! - 6x7 day grid with today highlighting
//! - Calendar week (KW) display
//! - German locale support

mod view;
mod widget;

pub use view::{CalendarPickerView, calendar_picker};
pub use widget::{CalendarAction, CalendarPickerWidget};
