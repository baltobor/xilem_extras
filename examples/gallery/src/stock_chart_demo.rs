//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Stock chart demo for the gallery.

use masonry::layout::AsUnit;
use xilem::masonry::peniko::Color;
use xilem::style::Style;
use xilem::view::{button, flex_col, flex_row, label, CrossAxisAlignment};
use xilem::WidgetView;
use xilem_extras::stock_chart::{stock_chart, StockBar, StockChartMode};

use crate::app_model::AppModel;

const TEXT_COLOR: Color = Color::from_rgb8(220, 218, 214);
const TEXT_SECONDARY: Color = Color::from_rgb8(160, 156, 150);

/// Generate sample stock data.
fn sample_stock_data() -> Vec<StockBar> {
    vec![
        StockBar::new("Jan 02", 100.0, 105.0, 98.0, 103.0, 1_200_000),
        StockBar::new("Jan 03", 103.0, 108.0, 101.0, 106.0, 1_500_000),
        StockBar::new("Jan 04", 106.0, 107.0, 102.0, 104.0, 900_000),
        StockBar::new("Jan 05", 104.0, 110.0, 103.0, 109.0, 2_100_000),
        StockBar::new("Jan 08", 109.0, 112.0, 107.0, 111.0, 1_800_000),
        StockBar::new("Jan 09", 111.0, 113.0, 108.0, 108.5, 1_400_000),
        StockBar::new("Jan 10", 108.5, 109.0, 104.0, 105.0, 2_000_000),
        StockBar::new("Jan 11", 105.0, 107.0, 103.0, 106.5, 1_300_000),
        StockBar::new("Jan 12", 106.5, 111.0, 105.5, 110.0, 1_600_000),
        StockBar::new("Jan 15", 110.0, 115.0, 109.0, 114.0, 2_500_000),
        StockBar::new("Jan 16", 114.0, 116.0, 112.0, 113.0, 1_900_000),
        StockBar::new("Jan 17", 113.0, 114.5, 110.0, 111.0, 1_700_000),
        StockBar::new("Jan 18", 111.0, 113.0, 109.0, 112.5, 1_200_000),
        StockBar::new("Jan 19", 112.5, 118.0, 112.0, 117.0, 3_000_000),
        StockBar::new("Jan 22", 117.0, 120.0, 115.0, 118.5, 2_200_000),
        StockBar::new("Jan 23", 118.5, 119.0, 114.0, 115.0, 1_800_000),
        StockBar::new("Jan 24", 115.0, 117.0, 113.0, 116.0, 1_500_000),
        StockBar::new("Jan 25", 116.0, 122.0, 115.5, 121.0, 2_800_000),
        StockBar::new("Jan 26", 121.0, 123.0, 118.0, 119.0, 2_100_000),
        StockBar::new("Jan 29", 119.0, 121.0, 117.0, 120.5, 1_600_000),
    ]
}

fn mode_from_index(index: usize) -> StockChartMode {
    match index {
        0 => StockChartMode::Candlestick,
        1 => StockChartMode::OhlcBar,
        2 => StockChartMode::Line,
        3 => StockChartMode::Area,
        _ => StockChartMode::Candlestick,
    }
}

fn mode_name(index: usize) -> &'static str {
    match index {
        0 => "Candlestick",
        1 => "OHLC Bar",
        2 => "Line",
        3 => "Area",
        _ => "Candlestick",
    }
}

pub fn stock_chart_demo(model: &mut AppModel) -> impl WidgetView<AppModel> + use<> {
    let bars = sample_stock_data();
    let mode = mode_from_index(model.stock_chart_mode);

    let hover_info = if let Some((label, price)) = &model.stock_chart_hover {
        format!("{}: ${:.2}", label, price)
    } else {
        "Hover over chart to see values".to_string()
    };

    flex_col((
        // Header
        label("Stock Chart")
            .text_size(18.0)
            .weight(xilem::FontWeight::BOLD)
            .color(TEXT_COLOR),
        label("OHLCV financial data visualization")
            .text_size(12.0)
            .color(TEXT_SECONDARY),

        // Mode selector buttons
        flex_row((
            button(
                label("Candlestick").text_size(11.0),
                |m: &mut AppModel| m.stock_chart_mode = 0,
            ),
            button(
                label("OHLC Bar").text_size(11.0),
                |m: &mut AppModel| m.stock_chart_mode = 1,
            ),
            button(
                label("Line").text_size(11.0),
                |m: &mut AppModel| m.stock_chart_mode = 2,
            ),
            button(
                label("Area").text_size(11.0),
                |m: &mut AppModel| m.stock_chart_mode = 3,
            ),
        ))
        .gap(8.px()),

        // Current mode and hover info
        flex_row((
            label(format!("Mode: {}", mode_name(model.stock_chart_mode)))
                .text_size(12.0)
                .color(TEXT_COLOR),
            label(hover_info)
                .text_size(12.0)
                .color(TEXT_SECONDARY),
        ))
        .gap(20.px()),

        // The stock chart
        stock_chart(bars, mode, |m: &mut AppModel, hover| {
            m.stock_chart_hover = hover.map(|h| (h.label, h.close));
        })
        .show_volume(true)
        .show_grid(true)
        .show_crosshair(true),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Stretch)
    .gap(12.px())
    .padding(16.0)
}
