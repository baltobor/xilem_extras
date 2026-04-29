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

use masonry::layout::AsUnit;
use masonry::properties::Padding;
use xilem::masonry::peniko::Color;
use xilem::style::Style;
use xilem::view::{flex_row, label, CrossAxisAlignment};
use xilem::AnyWidgetView;

use crate::components::{disclosure, row_button};

/// One unit of indentation per depth level, in pixels.
pub const DEFAULT_INDENT_PER_DEPTH: f64 = 16.0;

/// A row with optional chevron + content, indented by depth.
///
/// - `is_expandable: true` → renders a chevron wrapped in a `row_button`
///   that fires `on_toggle` on primary click. The content is laid out next
///   to the chevron and is *not* part of the toggle hit area.
/// - `is_expandable: false` → renders a transparent spacer the same width
///   as the chevron so leaf rows align with their expandable siblings.
///
/// The function returns a `Box<AnyWidgetView<...>>` so callers building a
/// `Vec` of heterogeneous rows (different content types per node) can collect
/// them uniformly.
pub fn disclosure_row<State, Toggle>(
    depth: usize,
    is_expanded: bool,
    is_expandable: bool,
    chevron_color: Color,
    content: Box<AnyWidgetView<State, ()>>,
    on_toggle: Toggle,
) -> Box<AnyWidgetView<State, ()>>
where
    State: 'static,
    Toggle: Fn(&mut State) + Send + Sync + 'static,
{
    let chevron_view: Box<AnyWidgetView<State, ()>> = if is_expandable {
        let glyph = disclosure(is_expanded).color(chevron_color).build();
        Box::new(row_button(glyph, move |state: &mut State| {
            on_toggle(state);
        }))
    } else {
        // Invisible spacer: two non-breaking spaces matches the chevron glyph's
        // approximate width at default sizes and keeps labels visually aligned.
        Box::new(label("\u{00A0}\u{00A0}").color(Color::TRANSPARENT))
    };

    let row = flex_row((chevron_view, content))
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .gap(4.px())
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: depth as f64 * DEFAULT_INDENT_PER_DEPTH,
            right: 4.0,
        });

    Box::new(row)
}
