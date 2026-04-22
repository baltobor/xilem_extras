//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Virtual list scroll state tracking.
//!
//! This module provides scroll state management for virtualized lists.
//! The actual rendering is done by the view layer using spacers and
//! windowed item creation.

/// Scroll state for a virtual list.
///
/// Track this in your model and update it based on scroll events.
/// The virtual_list view uses this to determine which items to render.
#[derive(Debug, Clone, Default)]
pub struct VirtualListState {
    /// Current scroll offset in pixels
    pub scroll_offset: f64,
    /// Viewport height in pixels (set from layout)
    pub viewport_height: f64,
    /// Row height in pixels
    pub row_height: f64,
    /// Total item count
    pub item_count: usize,
}

impl VirtualListState {
    /// Creates a new scroll state.
    pub fn new(row_height: f64) -> Self {
        Self {
            scroll_offset: 0.0,
            viewport_height: 600.0, // Default estimate
            row_height,
            item_count: 0,
        }
    }

    /// Sets the item count.
    pub fn set_item_count(&mut self, count: usize) {
        self.item_count = count;
        // Clamp scroll if needed
        let max_scroll = self.max_scroll();
        if self.scroll_offset > max_scroll {
            self.scroll_offset = max_scroll;
        }
    }

    /// Maximum scroll offset.
    pub fn max_scroll(&self) -> f64 {
        let content_height = self.item_count as f64 * self.row_height;
        (content_height - self.viewport_height).max(0.0)
    }

    /// Scroll by a delta amount.
    pub fn scroll_by(&mut self, delta: f64) {
        self.scroll_offset = (self.scroll_offset + delta).clamp(0.0, self.max_scroll());
    }

    /// Scroll to a specific offset.
    pub fn scroll_to(&mut self, offset: f64) {
        self.scroll_offset = offset.clamp(0.0, self.max_scroll());
    }

    /// Calculate the visible range of items.
    ///
    /// Returns (first_visible_index, visible_count) with buffer items included.
    pub fn visible_range(&self) -> (usize, usize) {
        self.visible_range_with_buffer(3)
    }

    /// Calculate visible range with custom buffer.
    pub fn visible_range_with_buffer(&self, buffer: usize) -> (usize, usize) {
        if self.item_count == 0 || self.row_height <= 0.0 {
            return (0, 0);
        }

        // First visible item
        let first_raw = (self.scroll_offset / self.row_height).floor() as usize;
        let first = first_raw.saturating_sub(buffer);

        // Visible count
        let visible_in_viewport = (self.viewport_height / self.row_height).ceil() as usize;
        let count = visible_in_viewport + buffer * 2;

        // Clamp to item count
        let count = count.min(self.item_count.saturating_sub(first));

        (first, count)
    }

    /// Get the first visible index.
    pub fn first_visible(&self) -> usize {
        self.visible_range().0
    }

    /// Get the number of visible items.
    pub fn visible_count(&self) -> usize {
        self.visible_range().1
    }
}

/// Widget for tracking scroll state (placeholder for future implementation).
///
/// NOTE: Full widget implementation pending. Currently using view-based
/// virtualization with VirtualListState for scroll tracking.
pub struct VirtualListWidget;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state() {
        let state = VirtualListState::new(28.0);
        assert_eq!(state.row_height, 28.0);
        assert_eq!(state.scroll_offset, 0.0);
    }

    #[test]
    fn visible_range_at_top() {
        let mut state = VirtualListState::new(28.0);
        state.viewport_height = 280.0; // 10 rows visible
        state.item_count = 100;

        let (first, count) = state.visible_range();
        assert_eq!(first, 0);
        assert!(count >= 10); // At least viewport + buffer
    }

    #[test]
    fn visible_range_scrolled() {
        let mut state = VirtualListState::new(28.0);
        state.viewport_height = 280.0;
        state.item_count = 100;
        state.scroll_offset = 280.0; // Scrolled 10 rows

        let (first, count) = state.visible_range();
        assert!(first >= 7); // 10 - buffer
        assert!(count >= 10);
    }

    #[test]
    fn scroll_clamping() {
        let mut state = VirtualListState::new(28.0);
        state.viewport_height = 280.0;
        state.item_count = 100;

        state.scroll_to(10000.0);
        assert!(state.scroll_offset <= state.max_scroll());

        state.scroll_to(-100.0);
        assert_eq!(state.scroll_offset, 0.0);
    }

    #[test]
    fn empty_list() {
        let state = VirtualListState::new(28.0);
        let (first, count) = state.visible_range();
        assert_eq!(first, 0);
        assert_eq!(count, 0);
    }
}
