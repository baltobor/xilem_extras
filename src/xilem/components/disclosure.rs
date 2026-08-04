//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

use xilem::masonry::peniko::Color;
use xilem::{AnyWidgetView, WidgetView};

use crate::masonry::components::chevron;
use crate::xilem_masonry::components::svg_icon;

/// Default icon size, matching `xilem_material_icons::ICON_SIZE_SM` (the
/// size this component used before switching to a vector chevron).
const DEFAULT_SIZE: f32 = 16.0;

/// A disclosure indicator (chevron) for expand/collapse.
///
/// Renders as a right-pointing chevron when collapsed, and the same
/// chevron rotated 90° (pointing down) when expanded — a single vector
/// path drawn directly, not a glyph from an icon font. That avoids the
/// expand/collapse arrow depending on a font's outlines being loaded.
#[derive(Debug, Clone)]
pub struct Disclosure {
    is_expanded: bool,
    size: f32,
    color: Option<Color>,
}

impl Disclosure {
    /// Creates a new disclosure indicator.
    pub fn new(is_expanded: bool) -> Self {
        Self {
            is_expanded,
            size: DEFAULT_SIZE,
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

    /// Returns whether this disclosure is expanded.
    pub fn is_expanded(&self) -> bool {
        self.is_expanded
    }

    /// Returns the chevron's rotation for the current state: 0° pointing
    /// right (collapsed), 90° pointing down (expanded).
    pub fn rotation_degrees(&self) -> f64 {
        if self.is_expanded { 90.0 } else { 0.0 }
    }

    /// Returns the configured size.
    pub fn icon_size(&self) -> f32 {
        self.size
    }

    /// Returns the configured color, if any.
    pub fn icon_color(&self) -> Option<Color> {
        self.color
    }

    /// Builds the disclosure as a xilem view.
    pub fn build<State: 'static, Action: 'static>(self) -> Box<AnyWidgetView<State, Action>> {
        let mut ic = chevron()
            .size(self.size as f64)
            .rotation_degrees(self.rotation_degrees());
        if let Some(color) = self.color {
            ic = ic.color(color);
        }
        svg_icon(ic).boxed()
    }
}

/// Creates a disclosure indicator.
///
/// # Arguments
///
/// * `is_expanded` - Whether the disclosure shows as expanded (down chevron)
///   or collapsed (right chevron).
pub fn disclosure(is_expanded: bool) -> Disclosure {
    Disclosure::new(is_expanded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disclosure_expanded() {
        let d = disclosure(true);
        assert!(d.is_expanded());
    }

    #[test]
    fn disclosure_collapsed() {
        let d = disclosure(false);
        assert!(!d.is_expanded());
    }

    #[test]
    fn disclosure_default_size() {
        let d = disclosure(false);
        assert_eq!(d.size, DEFAULT_SIZE);
    }

    #[test]
    fn disclosure_custom_size() {
        let d = disclosure(false).size(24.0);
        assert_eq!(d.size, 24.0);
    }

    #[test]
    fn disclosure_color() {
        let red = Color::from_rgb8(255, 0, 0);
        let d = disclosure(false).color(red);
        assert!(d.color.is_some());
    }

    #[test]
    fn disclosure_rotation_matches_expanded_state() {
        assert_eq!(disclosure(false).rotation_degrees(), 0.0);
        assert_eq!(disclosure(true).rotation_degrees(), 90.0);
    }

    #[test]
    fn disclosure_chaining() {
        let blue = Color::from_rgb8(0, 0, 255);
        let d = disclosure(true).size(20.0).color(blue);
        assert!(d.is_expanded());
        assert_eq!(d.size, 20.0);
        assert!(d.color.is_some());
    }
}
