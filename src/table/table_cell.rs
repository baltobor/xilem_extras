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
use xilem::style::Style;
use xilem::WidgetView;

use crate::components::ClippedView;

/// Wraps table cell content with automatic clipping and width constraint.
///
/// This helper ensures that cell content is clipped to the column width,
/// preventing text from overflowing into adjacent cells when columns
/// are resized smaller than their content.
///
/// # Example
///
/// ```ignore
/// use xilem_extras::table_cell;
///
/// // In a row builder:
/// flex_row((
///     table_cell(label(name).text_size(13.0).padding(4.0), w0),
///     table_cell(label(route).text_size(13.0).padding(4.0), w1),
///     table_cell(label(distance).text_size(13.0).padding(4.0), w2),
/// ))
/// .gap(2.px())
/// ```
pub fn table_cell<State: 'static, Action: 'static, V: WidgetView<State, Action>>(
    content: V,
    width: f64,
) -> impl WidgetView<State, Action> {
    ClippedView::new(content).width(width.px())
}
