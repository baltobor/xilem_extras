//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

/// Spacing and sizing constants for consistent UI layout.
///
/// These tokens follow an 8px grid system for visual harmony.

/// Base unit for spacing calculations.
pub const SPACING_UNIT: f64 = 8.0;

/// Extra small spacing (4px).
pub const SPACING_XS: f64 = SPACING_UNIT / 2.0;

/// Small spacing (8px).
pub const SPACING_SM: f64 = SPACING_UNIT;

/// Medium spacing (16px).
pub const SPACING_MD: f64 = SPACING_UNIT * 2.0;

/// Large spacing (24px).
pub const SPACING_LG: f64 = SPACING_UNIT * 3.0;

/// Extra large spacing (32px).
pub const SPACING_XL: f64 = SPACING_UNIT * 4.0;

/// Standard row height for list/tree items.
pub const ROW_HEIGHT: f64 = 24.0;

/// Compact row height.
pub const ROW_HEIGHT_COMPACT: f64 = 20.0;

/// Standard header height.
pub const HEADER_HEIGHT: f64 = 28.0;

/// Standard indentation for nested items.
pub const INDENT_WIDTH: f64 = 16.0;

/// Icon sizes (in points, for Material Symbols).
pub const ICON_SIZE_SM: f32 = 16.0;
pub const ICON_SIZE_MD: f32 = 20.0;
pub const ICON_SIZE_LG: f32 = 24.0;

/// Text sizes.
pub const TEXT_SIZE_SM: f32 = 11.0;
pub const TEXT_SIZE_MD: f32 = 13.0;
pub const TEXT_SIZE_LG: f32 = 14.0;

/// Border radius values.
pub const BORDER_RADIUS_SM: f64 = 2.0;
pub const BORDER_RADIUS_MD: f64 = 4.0;
pub const BORDER_RADIUS_LG: f64 = 8.0;

/// Standard panel padding.
pub const PANEL_PADDING: f64 = SPACING_SM;

/// Item padding (within rows).
pub const ITEM_PADDING: f64 = SPACING_XS;

/// Gap between flex items.
pub const FLEX_GAP: f64 = SPACING_XS;

/// Minimum touch target size (accessibility).
pub const MIN_TOUCH_TARGET: f64 = 44.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spacing_progression() {
        assert!(SPACING_XS < SPACING_SM);
        assert!(SPACING_SM < SPACING_MD);
        assert!(SPACING_MD < SPACING_LG);
        assert!(SPACING_LG < SPACING_XL);
    }

    #[test]
    fn spacing_based_on_unit() {
        assert_eq!(SPACING_SM, SPACING_UNIT);
        assert_eq!(SPACING_MD, SPACING_UNIT * 2.0);
        assert_eq!(SPACING_LG, SPACING_UNIT * 3.0);
        assert_eq!(SPACING_XL, SPACING_UNIT * 4.0);
    }

    #[test]
    fn icon_size_progression() {
        assert!(ICON_SIZE_SM < ICON_SIZE_MD);
        assert!(ICON_SIZE_MD < ICON_SIZE_LG);
    }

    #[test]
    fn text_size_progression() {
        assert!(TEXT_SIZE_SM < TEXT_SIZE_MD);
        assert!(TEXT_SIZE_MD < TEXT_SIZE_LG);
    }
}
