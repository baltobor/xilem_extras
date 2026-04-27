//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Chart widget module.
//!
//! Provides bar and line chart widgets for data visualization.

mod widget;
mod view;

pub use widget::{ChartWidget, ChartAction, ChartMode};
pub use view::{chart, ChartView};
