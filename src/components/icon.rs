//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

use xilem::masonry::vello::peniko::Color;

/// Material Symbols font family name.
pub const MATERIAL_SYMBOLS_FAMILY: &str = "Material Symbols Outlined";

/// Standard icon sizes.
pub const ICON_SIZE_SM: f32 = 16.0;
pub const ICON_SIZE_MD: f32 = 20.0;
pub const ICON_SIZE_LG: f32 = 24.0;

/// An icon view using Material Symbols.
///
/// Uses a label with the Material Symbols font family to render icons
/// as text glyphs.
#[derive(Debug, Clone)]
pub struct Icon {
    codepoint: String,
    size: f32,
    color: Option<Color>,
}

impl Icon {
    /// Creates a new icon with the given codepoint.
    pub fn new(codepoint: impl Into<String>) -> Self {
        Self {
            codepoint: codepoint.into(),
            size: ICON_SIZE_MD,
            color: None,
        }
    }

    /// Sets the icon size.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Sets the icon color.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Returns the codepoint string.
    pub fn codepoint(&self) -> &str {
        &self.codepoint
    }

    /// Returns the size.
    pub fn icon_size(&self) -> f32 {
        self.size
    }

}

/// Creates an icon with the given codepoint and default size (20px).
pub fn icon(codepoint: impl Into<String>) -> Icon {
    Icon::new(codepoint)
}

/// Creates a small icon (16px).
pub fn icon_sm(codepoint: impl Into<String>) -> Icon {
    Icon::new(codepoint).size(ICON_SIZE_SM)
}

/// Creates a medium icon (20px).
pub fn icon_md(codepoint: impl Into<String>) -> Icon {
    Icon::new(codepoint).size(ICON_SIZE_MD)
}

/// Creates a large icon (24px).
pub fn icon_lg(codepoint: impl Into<String>) -> Icon {
    Icon::new(codepoint).size(ICON_SIZE_LG)
}

/// Common Material Symbols icon codepoints.
pub mod icons {
    // Navigation
    pub const CHEVRON_LEFT: &str = "\u{e5cb}";
    pub const CHEVRON_RIGHT: &str = "\u{e5cc}";
    pub const EXPAND_MORE: &str = "\u{e5cf}";
    pub const EXPAND_LESS: &str = "\u{e5ce}";
    pub const ARROW_DROP_DOWN: &str = "\u{e5c5}";
    pub const ARROW_DROP_UP: &str = "\u{e5c7}";
    pub const ARROW_UPWARD: &str = "\u{e5d8}";
    pub const ARROW_DOWNWARD: &str = "\u{e5db}";

    // Files
    pub const FOLDER: &str = "\u{e2c7}";
    pub const FOLDER_OPEN: &str = "\u{e2c8}";
    pub const DESCRIPTION: &str = "\u{e873}";
    pub const INSERT_DRIVE_FILE: &str = "\u{e24d}";

    // Actions
    pub const CHECK: &str = "\u{e5ca}";
    pub const CLOSE: &str = "\u{e5cd}";
    pub const ADD: &str = "\u{e145}";
    pub const REMOVE: &str = "\u{e15b}";
    pub const DELETE: &str = "\u{e872}";
    pub const EDIT: &str = "\u{e3c9}";

    // Status
    pub const ERROR: &str = "\u{e000}";
    pub const WARNING: &str = "\u{e002}";
    pub const INFO: &str = "\u{e88e}";
    pub const CHECK_CIRCLE: &str = "\u{e86c}";

    // Sort
    pub const SORT: &str = "\u{e164}";
    pub const UNFOLD_MORE: &str = "\u{e5d7}";

    // Active mobility
    pub const PEDAL_BIKE: &str = "\u{eb29}";
    pub const DIRECTIONS_WALK: &str = "\u{e536}";
    pub const DIRECTIONS_RUN: &str = "\u{e566}";
    pub const PARK: &str = "\u{ea63}";
    pub const NATURE: &str = "\u{e406}";
    pub const FAVORITE: &str = "\u{e87d}";
    pub const SENTIMENT_SATISFIED: &str = "\u{e813}";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_default_size() {
        let i = icon("test");
        assert_eq!(i.icon_size(), ICON_SIZE_MD);
    }

    #[test]
    fn icon_sm_size() {
        let i = icon_sm("test");
        assert_eq!(i.icon_size(), ICON_SIZE_SM);
    }

    #[test]
    fn icon_md_size() {
        let i = icon_md("test");
        assert_eq!(i.icon_size(), ICON_SIZE_MD);
    }

    #[test]
    fn icon_lg_size() {
        let i = icon_lg("test");
        assert_eq!(i.icon_size(), ICON_SIZE_LG);
    }

    #[test]
    fn icon_custom_size() {
        let i = icon("test").size(32.0);
        assert_eq!(i.icon_size(), 32.0);
    }

    #[test]
    fn icon_codepoint() {
        let i = icon("test_icon");
        assert_eq!(i.codepoint(), "test_icon");
    }

    #[test]
    fn icon_color() {
        let red = Color::from_rgb8(255, 0, 0);
        let i = icon("test").color(red);
        assert!(i.color.is_some());
    }

    #[test]
    fn icon_chaining() {
        let blue = Color::from_rgb8(0, 0, 255);
        let i = icon("test")
            .size(32.0)
            .color(blue);
        assert_eq!(i.icon_size(), 32.0);
        assert!(i.color.is_some());
    }
}
