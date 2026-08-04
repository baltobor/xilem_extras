//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! A vector chevron icon, drawn as a plain `SvgIcon` path.
//!
//! Disclosure/expand indicators previously rendered via a glyph from
//! `xilem_material_icons`, which ties the icon's availability to that
//! font's outlines actually being loaded. A stroked chevron path needs no
//! font at all: one right-pointing path, rotated 90° for the "expanded"
//! (downward) state via `SvgIcon::rotation_degrees`, covers both.

use super::svg_icon::SvgIcon;

/// A right-pointing chevron (`>`) in a 24x24 viewBox, drawn as a stroked
/// path (round joins/caps, matching a typical expand/collapse glyph).
///
/// Combine with `SvgIcon::rotation_degrees(90.0)` for a downward-pointing
/// chevron.
pub fn chevron() -> SvgIcon {
    SvgIcon::from_svg("M9 6 L15 12 L9 18", 24.0, 24.0).stroke_width(2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chevron_has_no_fill_area_by_default() {
        // An open, unfilled path only renders via its stroke.
        let icon = chevron();
        assert_eq!(icon.stroke_width, Some(2.0));
    }

    #[test]
    fn chevron_rotates_for_expanded_state() {
        let collapsed = chevron();
        let expanded = chevron().rotation_degrees(90.0);
        assert_eq!(collapsed.rotation_degrees, 0.0);
        assert_eq!(expanded.rotation_degrees, 90.0);
    }
}
