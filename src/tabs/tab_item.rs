//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Tab item trait for tab bar data.

/// Trait for items that can be displayed in a tab bar.
///
/// Implement this trait for your tab data structure to use with [`TabBar`].
///
/// # Example
///
/// ```
/// use xilem_extras::tabs::TabItem;
///
/// struct DocumentTab {
///     name: String,
///     path: String,
///     modified: bool,
/// }
///
/// impl TabItem for DocumentTab {
///     fn title(&self) -> &str {
///         &self.name
///     }
///
///     fn is_dirty(&self) -> bool {
///         self.modified
///     }
/// }
/// ```
///
/// [`TabBar`]: super::TabBar
pub trait TabItem {
    /// Returns the display title for this tab.
    fn title(&self) -> &str;

    /// Returns whether this tab has unsaved changes.
    ///
    /// When true, an asterisk (*) is appended to the title.
    fn is_dirty(&self) -> bool {
        false
    }

    /// Returns an optional tooltip for this tab.
    fn tooltip(&self) -> Option<&str> {
        None
    }
}

/// A simple tab item implementation for basic use cases.
#[derive(Debug, Clone)]
pub struct SimpleTab {
    /// Tab title.
    pub title: String,
    /// Whether the tab has unsaved changes.
    pub dirty: bool,
}

impl SimpleTab {
    /// Creates a new tab with the given title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            dirty: false,
        }
    }

    /// Sets the dirty state.
    pub fn dirty(mut self, dirty: bool) -> Self {
        self.dirty = dirty;
        self
    }
}

impl TabItem for SimpleTab {
    fn title(&self) -> &str {
        &self.title
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_tab_title() {
        let tab = SimpleTab::new("Test");
        assert_eq!(tab.title(), "Test");
    }

    #[test]
    fn simple_tab_dirty() {
        let tab = SimpleTab::new("Test").dirty(true);
        assert!(tab.is_dirty());
    }

    #[test]
    fn simple_tab_clean_by_default() {
        let tab = SimpleTab::new("Test");
        assert!(!tab.is_dirty());
    }
}
