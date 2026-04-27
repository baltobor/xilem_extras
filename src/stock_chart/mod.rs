//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Stock chart widget module.
//!
//! Provides candlestick, bar (OHLC), and line chart widgets for financial data visualization.
//!
//! # Chart Types
//!
//! - **Candlestick** - Traditional OHLC candles with filled bodies
//! - **OHLC Bar** - High-low lines with open/close tick marks
//! - **Line** - Simple line connecting close prices
//! - **Area** - Line chart with gradient fill below
//!
//! # Example
//!
//! ```ignore
//! use xilem_extras::stock_chart::{stock_chart, StockBar, StockChartMode};
//!
//! let bars = vec![
//!     StockBar::new("2024-01-01", 100.0, 105.0, 98.0, 103.0, 1000),
//!     StockBar::new("2024-01-02", 103.0, 108.0, 102.0, 107.0, 1200),
//! ];
//!
//! stock_chart(bars, StockChartMode::Candlestick, |date, price| {
//!     // Handle hover
//! })
//! ```

mod widget;
mod view;

pub use widget::{StockChartWidget, StockBar, StockChartMode, StockChartStyle};
pub use view::{stock_chart, StockChartView};
