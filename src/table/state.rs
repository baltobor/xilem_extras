//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Table scroll state for virtualized rendering.
//!
//! # Anchor-Based Scrolling
//!
//! Instead of tracking an absolute scroll offset (which can lose precision at
//! large values), we track:
//!
//! - **anchor_index**: The row index at or above the viewport top
//! - **scroll_offset_from_anchor**: Pixel offset from the anchor's top edge
//!
//! This representation:
//! - Avoids floating-point precision issues with large scroll positions
//! - Naturally supports variable row heights (future enhancement)
//! - Makes hit-testing O(1) relative to visible area
//!
//! # Buffer Zones
//!
//! The `compute_target_range()` method returns a range larger than the visible
//! area to pre-load rows before they scroll into view:
//!
//! ```text
//! ┌─────────────────────────────────┐
//! │ 1.5x viewport ABOVE (buffer)   │  ← Pre-loaded for upward scroll
//! ├─────────────────────────────────┤
//! │ Visible viewport               │  ← Actually on screen
//! ├─────────────────────────────────┤
//! │ 2.5x viewport BELOW (buffer)   │  ← Pre-loaded for downward scroll
//! └─────────────────────────────────┘
//! ```
//!
//! The asymmetric buffer (more below) optimizes for the common case of scrolling
//! down through a list.

use std::ops::Range;

/// Scroll state for virtualized table.
///
/// Uses anchor-based positioning for smooth scrolling and efficient
/// row virtualization. The anchor is the first row at or above the
/// viewport top, with an offset for sub-row positioning.
#[derive(Debug, Clone)]
pub struct TableScrollState {
    /// Anchor row index (first row at/above viewport top).
    pub anchor_index: usize,
    /// Pixel offset from anchor row top (for smooth scrolling).
    /// Positive means the anchor is above the viewport top.
    pub scroll_offset_from_anchor: f64,
    /// Viewport height in pixels.
    pub viewport_height: f64,
    /// Row height in pixels (fixed for v1).
    pub row_height: f64,
    /// Total item count.
    pub item_count: usize,
    /// Currently loaded row range.
    pub active_range: Range<usize>,
}

impl Default for TableScrollState {
    fn default() -> Self {
        Self::new(28.0)
    }
}

impl TableScrollState {
    /// Creates a new scroll state with the given row height.
    pub fn new(row_height: f64) -> Self {
        Self {
            anchor_index: 0,
            scroll_offset_from_anchor: 0.0,
            viewport_height: 600.0, // Default estimate
            row_height,
            item_count: 0,
            active_range: 0..0,
        }
    }

    /// Sets the item count and clamps scroll position if needed.
    pub fn set_item_count(&mut self, count: usize) {
        self.item_count = count;
        // Ensure anchor doesn't exceed item count
        if self.anchor_index >= count && count > 0 {
            self.anchor_index = count.saturating_sub(1);
            self.scroll_offset_from_anchor = 0.0;
        }
    }

    /// Sets the viewport height.
    pub fn set_viewport_height(&mut self, height: f64) {
        self.viewport_height = height;
    }

    /// Total content height.
    pub fn content_height(&self) -> f64 {
        self.item_count as f64 * self.row_height
    }

    /// Maximum scroll offset (bottom of content visible).
    pub fn max_scroll_offset(&self) -> f64 {
        (self.content_height() - self.viewport_height).max(0.0)
    }

    /// Current absolute scroll position.
    pub fn scroll_position(&self) -> f64 {
        self.anchor_index as f64 * self.row_height + self.scroll_offset_from_anchor
    }

    /// Compute visible range with buffer zones.
    ///
    /// Buffer: 1.5x viewport above, 2.5x below for smooth scrolling.
    /// This prevents blank areas during fast scrolling.
    pub fn compute_target_range(&self) -> Range<usize> {
        if self.item_count == 0 || self.row_height <= 0.0 {
            return 0..0;
        }

        let buffer_above = (self.viewport_height * 1.5 / self.row_height).ceil() as usize;
        let buffer_below = (self.viewport_height * 2.5 / self.row_height).ceil() as usize;

        let first = self.anchor_index.saturating_sub(buffer_above);
        let visible = (self.viewport_height / self.row_height).ceil() as usize;
        let last = (self.anchor_index + visible + buffer_below).min(self.item_count);

        first..last
    }

    /// Compute strictly visible range (no buffer).
    pub fn visible_range(&self) -> Range<usize> {
        if self.item_count == 0 || self.row_height <= 0.0 {
            return 0..0;
        }

        let first_visible = self.anchor_index;
        let visible_count = (self.viewport_height / self.row_height).ceil() as usize + 1;
        let last = (first_visible + visible_count).min(self.item_count);

        first_visible..last
    }

    /// Apply scroll delta, adjusting anchor when needed.
    ///
    /// Positive delta scrolls down (content moves up).
    pub fn scroll_by(&mut self, delta: f64) {
        if self.item_count == 0 {
            return;
        }

        self.scroll_offset_from_anchor += delta;

        // Adjust anchor when offset goes out of bounds (scrolling up)
        while self.scroll_offset_from_anchor < 0.0 && self.anchor_index > 0 {
            self.anchor_index -= 1;
            self.scroll_offset_from_anchor += self.row_height;
        }

        // Adjust anchor when offset exceeds row height (scrolling down)
        while self.scroll_offset_from_anchor >= self.row_height
            && self.anchor_index < self.item_count.saturating_sub(1)
        {
            self.anchor_index += 1;
            self.scroll_offset_from_anchor -= self.row_height;
        }

        // Clamp at top boundary
        if self.anchor_index == 0 && self.scroll_offset_from_anchor < 0.0 {
            self.scroll_offset_from_anchor = 0.0;
        }

        // Clamp at bottom boundary
        let max_scroll = self.max_scroll_offset();
        let current_scroll = self.scroll_position();
        if current_scroll > max_scroll {
            // Recalculate anchor for max scroll position
            let max_anchor = (max_scroll / self.row_height).floor() as usize;
            self.anchor_index = max_anchor.min(self.item_count.saturating_sub(1));
            self.scroll_offset_from_anchor = max_scroll - (self.anchor_index as f64 * self.row_height);
            self.scroll_offset_from_anchor = self.scroll_offset_from_anchor.max(0.0);
        }
    }

    /// Scroll to a specific absolute offset.
    pub fn scroll_to(&mut self, offset: f64) {
        let target = offset.clamp(0.0, self.max_scroll_offset());
        self.anchor_index = (target / self.row_height).floor() as usize;
        self.scroll_offset_from_anchor = target - (self.anchor_index as f64 * self.row_height);
    }

    /// Scroll to make a specific row visible.
    pub fn scroll_to_row(&mut self, row_index: usize) {
        if row_index >= self.item_count {
            return;
        }

        let row_top = row_index as f64 * self.row_height;
        let row_bottom = row_top + self.row_height;
        let viewport_top = self.scroll_position();
        let viewport_bottom = viewport_top + self.viewport_height;

        if row_top < viewport_top {
            // Row is above viewport, scroll up
            self.scroll_to(row_top);
        } else if row_bottom > viewport_bottom {
            // Row is below viewport, scroll down
            self.scroll_to(row_bottom - self.viewport_height);
        }
    }

    /// Y position for a given row index (relative to viewport top).
    ///
    /// Returns the y coordinate where the row should be painted.
    pub fn row_y(&self, row_index: usize) -> f64 {
        let rows_from_anchor = row_index as i64 - self.anchor_index as i64;
        rows_from_anchor as f64 * self.row_height - self.scroll_offset_from_anchor
    }

    /// Hit test: which row index is at the given y position (relative to viewport)?
    ///
    /// Returns None if position is outside content bounds.
    pub fn row_at_y(&self, y: f64) -> Option<usize> {
        let doc_y = y + self.scroll_offset_from_anchor;
        let row_offset = (doc_y / self.row_height).floor() as i64;
        let row_index = self.anchor_index as i64 + row_offset;

        if row_index >= 0 && (row_index as usize) < self.item_count {
            Some(row_index as usize)
        } else {
            None
        }
    }

    /// Returns true if the active range should be updated.
    pub fn needs_range_update(&self) -> bool {
        let target = self.compute_target_range();
        target != self.active_range
    }

    /// Scrollbar thumb position (0.0 to 1.0).
    pub fn scrollbar_thumb_position(&self) -> f64 {
        let max = self.max_scroll_offset();
        if max <= 0.0 {
            0.0
        } else {
            self.scroll_position() / max
        }
    }

    /// Scrollbar thumb size as fraction of track (0.0 to 1.0).
    pub fn scrollbar_thumb_size(&self) -> f64 {
        let content = self.content_height();
        if content <= 0.0 {
            1.0
        } else {
            (self.viewport_height / content).clamp(0.05, 1.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state() {
        let state = TableScrollState::new(28.0);
        assert_eq!(state.row_height, 28.0);
        assert_eq!(state.anchor_index, 0);
        assert_eq!(state.scroll_offset_from_anchor, 0.0);
    }

    #[test]
    fn scroll_by_within_row() {
        let mut state = TableScrollState::new(28.0);
        state.item_count = 100;
        state.viewport_height = 280.0;

        state.scroll_by(10.0);
        assert_eq!(state.anchor_index, 0);
        assert!((state.scroll_offset_from_anchor - 10.0).abs() < 0.01);
    }

    #[test]
    fn scroll_by_crosses_row() {
        let mut state = TableScrollState::new(28.0);
        state.item_count = 100;
        state.viewport_height = 280.0;

        state.scroll_by(30.0); // More than one row
        assert_eq!(state.anchor_index, 1);
        assert!((state.scroll_offset_from_anchor - 2.0).abs() < 0.01);
    }

    #[test]
    fn scroll_by_multiple_rows() {
        let mut state = TableScrollState::new(28.0);
        state.item_count = 100;
        state.viewport_height = 280.0;

        state.scroll_by(280.0); // 10 rows
        assert_eq!(state.anchor_index, 10);
        assert!((state.scroll_offset_from_anchor - 0.0).abs() < 0.01);
    }

    #[test]
    fn scroll_clamps_at_top() {
        let mut state = TableScrollState::new(28.0);
        state.item_count = 100;
        state.viewport_height = 280.0;

        state.scroll_by(-100.0);
        assert_eq!(state.anchor_index, 0);
        assert_eq!(state.scroll_offset_from_anchor, 0.0);
    }

    #[test]
    fn scroll_clamps_at_bottom() {
        let mut state = TableScrollState::new(28.0);
        state.item_count = 20;
        state.viewport_height = 280.0;

        state.scroll_by(10000.0);
        let max_scroll = state.max_scroll_offset();
        assert!((state.scroll_position() - max_scroll).abs() < 1.0);
    }

    #[test]
    fn compute_target_range_at_top() {
        let mut state = TableScrollState::new(28.0);
        state.item_count = 100;
        state.viewport_height = 280.0; // ~10 rows visible

        let range = state.compute_target_range();
        assert_eq!(range.start, 0);
        // Should include buffer below
        assert!(range.end > 10);
    }

    #[test]
    fn compute_target_range_scrolled() {
        let mut state = TableScrollState::new(28.0);
        state.item_count = 100;
        state.viewport_height = 280.0;
        state.anchor_index = 50;

        let range = state.compute_target_range();
        // Should include buffer above and below
        assert!(range.start < 50);
        assert!(range.end > 60);
    }

    #[test]
    fn row_y_positioning() {
        let mut state = TableScrollState::new(28.0);
        state.item_count = 100;
        state.viewport_height = 280.0;

        // At top, row 0 is at y=0
        assert!((state.row_y(0) - 0.0).abs() < 0.01);
        assert!((state.row_y(1) - 28.0).abs() < 0.01);

        // Scroll half a row
        state.scroll_by(14.0);
        assert!((state.row_y(0) - (-14.0)).abs() < 0.01);
        assert!((state.row_y(1) - 14.0).abs() < 0.01);
    }

    #[test]
    fn row_at_y_hit_test() {
        let mut state = TableScrollState::new(28.0);
        state.item_count = 100;
        state.viewport_height = 280.0;

        assert_eq!(state.row_at_y(0.0), Some(0));
        assert_eq!(state.row_at_y(27.0), Some(0));
        assert_eq!(state.row_at_y(28.0), Some(1));
        assert_eq!(state.row_at_y(55.0), Some(1));
    }

    #[test]
    fn scrollbar_calculations() {
        let mut state = TableScrollState::new(28.0);
        state.item_count = 100;
        state.viewport_height = 280.0;

        assert!((state.scrollbar_thumb_position() - 0.0).abs() < 0.01);
        assert!((state.scrollbar_thumb_size() - 0.1).abs() < 0.01);

        state.scroll_to(state.max_scroll_offset());
        assert!((state.scrollbar_thumb_position() - 1.0).abs() < 0.01);
    }

    #[test]
    fn scroll_to_row() {
        let mut state = TableScrollState::new(28.0);
        state.item_count = 100;
        state.viewport_height = 280.0;

        // Row 50 is off screen, should scroll to it
        state.scroll_to_row(50);
        let range = state.visible_range();
        assert!(range.contains(&50));
    }

    #[test]
    fn empty_list() {
        let state = TableScrollState::new(28.0);
        let range = state.compute_target_range();
        assert_eq!(range, 0..0);
    }
}
