//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Table cell helper for automatic content clipping.
//!
//! Provides a convenient wrapper that clips cell content to prevent
//! text overflow into adjacent columns when columns are resized.

use masonry::layout::AsUnit;
use xilem::WidgetView;
use xilem::style::Style;

use crate::xilem::components::ClippedView;

/// Wraps table cell content with automatic clipping and width constraint.
///
/// This helper ensures that cell content is clipped to the column width,
/// preventing text from overflowing into adjacent cells when columns
/// are resized smaller than their content.
///
/// # Example
///
/// ```ignore
/// use xilem_extras::xilem::table::{row_cells, table_cell};
///
/// // In a row builder, given `widths: &[f64]` and `x_offsets: &[f64]`:
/// row_cells(
///     vec![
///         table_cell(label(name).text_size(13.0).padding(4.0), widths[0]),
///         table_cell(label(route).text_size(13.0).padding(4.0), widths[1]),
///     ],
///     widths,
///     x_offsets,
/// )
/// ```
///
/// `row_cells` places each cell at its exact `x_offset` — the same
/// mechanism the header uses for its own cells — rather than a sequential
/// layout like `flex_row`, which cannot reproduce the header's positions
/// in RTL (a plain sequential layout can't make a column's growth shift a
/// column that comes before it in iteration order).
pub fn table_cell<State: 'static, Action: 'static, V: WidgetView<State, Action>>(
    content: V,
    width: f64,
) -> impl WidgetView<State, Action> {
    ClippedView::new(content).width(width.px())
}
