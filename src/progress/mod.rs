//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Progress controls.
//!
//! Two non-interactive widgets for displaying progress / a
//! single bounded value:
//!
//! - **Linear progress bar** (`progress_bar`) — horizontal or
//!   vertical filled bar with three render styles
//!   (`Gradient` / `Tint` / `Monochrome`).
//! - **Round progress bar** (`round_progress`) — thin arc that
//!   sweeps from 0 % to 100 %, with normal and small (text-
//!   height) sizes. Doubles as a busy / indeterminate indicator
//!   when no value is supplied.
//!
//! Linear-only scaling — no logarithmic / dB transforms. The
//! caller's `min..max` is treated as the linear domain.

mod round_view;
mod round_widget;
mod view;
mod widget;

pub use round_view::{round_progress, RoundProgressView};
pub use round_widget::{RoundProgressSize, RoundProgressWidget};
pub use view::{progress_bar, ProgressBarView};
pub use widget::{ProgressBarWidget, ProgressOrientation, ProgressStyle};
