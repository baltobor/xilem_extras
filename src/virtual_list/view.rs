//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Virtual list view for Xilem.
//!
//! Provides a declarative API for virtualized lists that only render
//! visible items based on scroll state.

use xilem::masonry::peniko::Color;
use xilem::style::Style;

/// Style configuration for virtual list.
#[derive(Debug, Clone)]
pub struct VirtualListStyle {
    /// Height of each row in pixels.
    pub row_height: f64,
    /// Background color for visible rows.
    pub row_bg: Color,
    /// Alternating row background (if striped).
    pub stripe_bg: Color,
    /// Whether to use alternating backgrounds.
    pub striped: bool,
    /// Hover background color.
    pub hover_bg: Color,
    /// Selected row background.
    pub selected_bg: Color,
}

impl Default for VirtualListStyle {
    fn default() -> Self {
        Self {
            row_height: 28.0,
            row_bg: Color::TRANSPARENT,
            stripe_bg: Color::from_rgba8(45, 43, 40, 255),
            striped: false,
            hover_bg: Color::from_rgba8(55, 53, 50, 255),
            selected_bg: Color::from_rgba8(65, 62, 58, 255),
        }
    }
}

impl VirtualListStyle {
    /// Creates a new style with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the row height.
    pub fn row_height(mut self, height: f64) -> Self {
        self.row_height = height;
        self
    }

    /// Enables striped backgrounds.
    pub fn striped(mut self, striped: bool) -> Self {
        self.striped = striped;
        self
    }

    /// Sets the hover background color.
    pub fn hover_bg(mut self, color: Color) -> Self {
        self.hover_bg = color;
        self
    }

    /// Sets the selected row background.
    pub fn selected_bg(mut self, color: Color) -> Self {
        self.selected_bg = color;
        self
    }
}

use masonry::layout::{AsUnit, Length};
use xilem::view::{flex_col, portal, sized_box};
use xilem::{AnyWidgetView, WidgetView};

use super::VirtualListState;

/// Creates a virtualized list that only renders visible items.
///
/// Uses spacers above and below the visible items to maintain
/// correct scroll size and position.
///
/// # Arguments
///
/// * `state` - Scroll state tracking visible range
/// * `row_builder` - Function that builds a view for each visible row: `(index) -> View`
///
/// # Example
///
/// ```ignore
/// use xilem_extras::{virtual_list, VirtualListState};
///
/// // In your model:
/// pub scroll_state: VirtualListState,
///
/// // In your view:
/// virtual_list(
///     &model.scroll_state,
///     |idx| {
///         let item = &model.items[idx];
///         create_row(item, idx)
///     },
/// )
/// ```
pub fn virtual_list<'a, State, R, F>(
    scroll_state: &'a VirtualListState,
    row_builder: F,
) -> impl WidgetView<State, ()> + use<'a, State, R, F>
where
    State: 'static,
    R: WidgetView<State, ()> + 'static,
    F: Fn(usize) -> R + Clone + 'a,
{
    virtual_list_styled(scroll_state, VirtualListStyle::default(), row_builder)
}

/// Creates a virtualized list with custom styling.
pub fn virtual_list_styled<'a, State, R, F>(
    scroll_state: &'a VirtualListState,
    _style: VirtualListStyle,
    row_builder: F,
) -> impl WidgetView<State, ()> + use<'a, State, R, F>
where
    State: 'static,
    R: WidgetView<State, ()> + 'static,
    F: Fn(usize) -> R + Clone + 'a,
{
    let (first_visible, visible_count) = scroll_state.visible_range();
    let last_visible = first_visible + visible_count;
    let item_count = scroll_state.item_count;
    let row_height = scroll_state.row_height;

    // Space above visible items (represents items scrolled past)
    let space_above = first_visible as f64 * row_height;

    // Space below visible items (represents items not yet scrolled to)
    let remaining_items = item_count.saturating_sub(last_visible);
    let space_below = remaining_items as f64 * row_height;

    // Build only visible rows
    let mut rows: Vec<Box<AnyWidgetView<State, ()>>> = Vec::with_capacity(visible_count + 2);

    // Top spacer
    if space_above > 0.0 {
        rows.push(
            sized_box(xilem::view::label(""))
                .fixed_height(Length::px(space_above))
                .boxed()
        );
    }

    // Visible items
    for idx in first_visible..last_visible.min(item_count) {
        rows.push(row_builder(idx).boxed());
    }

    // Bottom spacer
    if space_below > 0.0 {
        rows.push(
            sized_box(xilem::view::label(""))
                .fixed_height(Length::px(space_below))
                .boxed()
        );
    }

    portal(flex_col(rows).gap(0.px()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_builder() {
        let style = VirtualListStyle::new()
            .row_height(32.0)
            .striped(true)
            .hover_bg(Color::from_rgb8(60, 60, 60));

        assert_eq!(style.row_height, 32.0);
        assert!(style.striped);
        assert_eq!(style.hover_bg, Color::from_rgb8(60, 60, 60));
    }

    #[test]
    fn style_default() {
        let style = VirtualListStyle::default();
        assert_eq!(style.row_height, 28.0);
        assert!(!style.striped);
    }
}
