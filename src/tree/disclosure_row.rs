//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! `disclosure_row` — the SwiftUI-style row primitive: indent + chevron + content.
//!
//! The chevron is the only thing that toggles expansion (matches SwiftUI's
//! `DisclosureGroup` default and the user's "chevron-only toggle" requirement).
//! Whatever clicks the caller wants on the row body (select, double-click,
//! context menu) are wired via the `content` view's own handlers.
//!
//! This is pure xilem composition — no masonry widget, no type erasure.

use masonry::layout::{AsUnit, Length};
use masonry::properties::Padding;
use xilem::masonry::peniko::Color;
use xilem::style::Style;
use xilem::view::{flex_row, label, sized_box, CrossAxisAlignment};
use xilem::AnyWidgetView;

use crate::components::{disclosure, row_button};

/// One unit of indentation per depth level, in pixels. Sized so each
/// level shifts a child's chevron clear of the parent's icon column —
/// chevron glyph + 4px gap is roughly 16px, so 20px gives a clearly
/// visible nesting step rather than children rendering under the
/// parent's icon.
pub const DEFAULT_INDENT_PER_DEPTH: f64 = 20.0;

/// Default fixed width of the chevron column, in pixels. Both branches
/// (expandable: chevron + row_button; leaf: transparent placeholder) are
/// wrapped in a `sized_box` of this width so a leaf at depth N aligns
/// horizontally with an expandable sibling at the same depth. Override
/// per-call by passing a different `chevron_col_width` to `disclosure_row`.
pub const DEFAULT_CHEVRON_COL_WIDTH: f64 = 16.0;

/// A row with optional chevron + content, indented by depth.
///
/// - `is_expandable: true` → the chevron column holds a chevron wrapped in
///   a `row_button` that fires `on_toggle` on primary click. The content
///   sits to the right and is *not* part of the toggle hit area.
/// - `is_expandable: false` → the chevron column holds a transparent
///   placeholder so leaf rows align with their expandable siblings.
///
/// Both branches occupy exactly `chevron_col_width` pixels; pass
/// [`DEFAULT_CHEVRON_COL_WIDTH`] for a sane default. `indent_per_depth`
/// scales the left padding linearly with `depth`; pass
/// [`DEFAULT_INDENT_PER_DEPTH`] for the default.
///
/// `row_background` paints behind the entire row (chevron column + content).
/// Use `Color::TRANSPARENT` when you don't want a row-wide highlight; the
/// `tree_view` builder uses this for full-row selection fills. Note that
/// for the bg to actually span the *full panel width*, the parent
/// `flex_col` must use [`CrossAxisAlignment::Stretch`] — otherwise the row
/// is sized to its content and the bg covers only that.
///
/// The function returns a `Box<AnyWidgetView<...>>` so callers building a
/// `Vec` of heterogeneous rows (different content types per node) can collect
/// them uniformly.
pub fn disclosure_row<State, Toggle>(
    depth: usize,
    indent_per_depth: f64,
    chevron_col_width: f64,
    is_expanded: bool,
    is_expandable: bool,
    chevron_color: Color,
    content: Box<AnyWidgetView<State, ()>>,
    on_toggle: Toggle,
    row_background: Color,
) -> Box<AnyWidgetView<State, ()>>
where
    State: 'static,
    Toggle: Fn(&mut State) + Send + Sync + 'static,
{
    let chevron_view: Box<AnyWidgetView<State, ()>> = if is_expandable {
        let glyph = disclosure(is_expanded).color(chevron_color).build();
        let btn = row_button(glyph, move |state: &mut State| {
            on_toggle(state);
        });
        Box::new(sized_box(btn).width(chevron_col_width.px()))
    } else {
        // Empty label as a transparent stand-in; sized_box pins the column
        // to the same width as the expandable branch.
        Box::new(sized_box(label("")).width(chevron_col_width.px()))
    };

    let row = flex_row((chevron_view, content))
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .gap(4.px())
        .padding(Padding {
            top: Length::px(2.0),
            bottom: Length::px(2.0),
            left: Length::px(depth as f64 * indent_per_depth),
            right: Length::px(4.0),
        });

    if row_background != Color::TRANSPARENT {
        Box::new(row.background_color(row_background))
    } else {
        Box::new(row)
    }
}
