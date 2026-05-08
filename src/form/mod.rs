//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! SwiftUI-inspired Form container.
//!
//! A `Form` is "a container for grouping controls used for data
//! entry, such as in settings or inspectors". In SwiftUI's words:
//! *"Forms are regular containers just like VStack, so you can
//! switch between the two freely depending on your purpose."*
//!
//! `form((row1, row2, row3))` accepts the same kind of children
//! tuple a `flex_col` would, so swapping the call site between
//! `form(...)` and `flex_col(...)` is a one-word change. The
//! Form adds form-style chrome: consistent padding, gap, and
//! background.
//!
//! Sections are built with `form_section(header, content)`,
//! which uses [`crate::group_box`] under the hood — the header
//! label colour is derived from the section background via APCA
//! perceptual contrast, so any tint stays readable.
//!
//! Rows that pair a label with a control use
//! [`form_row`], mirroring SwiftUI's `Toggle("Label", isOn: …)`
//! pattern: label on the left (flex-grown so multiple rows align)
//! and control on the right.

mod view;

pub use view::{
    form, form_section, form_row, form_toggle, form_radio, form_checkbox,
    form_themed, form_section_themed, form_row_themed,
    form_toggle_themed, form_radio_themed, form_checkbox_themed,
};
