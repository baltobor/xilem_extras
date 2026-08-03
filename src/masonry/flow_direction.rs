// Copyright 2026 the Xilem Authors
// SPDX-License-Identifier: Apache-2.0

//! Layout flow direction (left-to-right vs. right-to-left).

/// Which direction content logically flows in — left-to-right or right-to-left.
///
/// General-purpose, not tied to any single widget: this is a layout concern
/// (like [`kurbo::Axis`](xilem::masonry::kurbo::Axis)), not a table- or
/// calendar-specific one, so it lives at the masonry level and is reused
/// wherever LTR/RTL matters (e.g. `CalendarLocale::flow_direction`, the
/// table's column resize/layout).
///
/// Open question for contributors: should this always be set explicitly by
/// the app (e.g. from an app-level locale/settings flag), or is it fine to
/// keep auto-detecting it from rendered text as this crate currently does
/// for tables (see `xilem::table::direction_detect`)? What do you prefer —
/// should we explicitly mark LTR and RTL, or should we keep it autodetected
/// from the font?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlowDirection {
    #[default]
    Ltr,
    Rtl,
}

impl FlowDirection {
    /// Returns whether this is right-to-left.
    pub fn is_rtl(&self) -> bool {
        matches!(self, FlowDirection::Rtl)
    }
}
