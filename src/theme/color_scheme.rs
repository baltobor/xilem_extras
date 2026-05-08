//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Light / dark colour scheme for xilem_extras widgets.
//!
//! Theming is a *library* feature: widgets that need contrasting
//! palettes accept a [`Theme`] and pick their colours from it.
//! Callers driving a dark/bright toggle (system theme, a user
//! setting) compute one [`Theme`] per render and pass it through
//! the widget tree — application code never patches per-widget
//! colours.

use xilem::masonry::peniko::Color;
use xilem::masonry::peniko::color::AlphaColor;

use crate::CheckboxColors;

/// Display theme. Drives the foreground / background palette of
/// the themable widgets (form, sections, boolean controls, the
/// gallery shell).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Theme {
    /// Dark mode — light text on dark backgrounds. Default.
    #[default]
    Dark,
    /// Light mode — dark text on light backgrounds.
    Light,
}

impl Theme {
    /// `Theme::Dark` if `dark` is `true`, otherwise `Theme::Light`.
    pub fn from_dark(dark: bool) -> Self {
        if dark { Self::Dark } else { Self::Light }
    }

    /// Whether this is the dark variant.
    pub fn is_dark(self) -> bool {
        matches!(self, Self::Dark)
    }

    /// Page / form chrome background.
    pub fn bg(self) -> Color {
        match self {
            Self::Dark => Color::from_rgb8(0x22, 0x20, 0x1D),
            Self::Light => Color::from_rgb8(0xE8, 0xE6, 0xE0),
        }
    }

    /// Surface one level above [`Theme::bg`] — used for section
    /// panels and grouped controls.
    pub fn section_bg(self) -> Color {
        match self {
            Self::Dark => Color::from_rgb8(0x2A, 0x28, 0x25),
            Self::Light => Color::from_rgb8(0xF2, 0xF0, 0xEA),
        }
    }

    /// Outer page background (gallery main content area).
    pub fn page_bg(self) -> Color {
        match self {
            Self::Dark => Color::from_rgb8(0x23, 0x21, 0x1E),
            Self::Light => Color::from_rgb8(0xEE, 0xEC, 0xE6),
        }
    }

    /// Sidebar / nav background.
    pub fn nav_bg(self) -> Color {
        match self {
            Self::Dark => Color::from_rgb8(0x2D, 0x2B, 0x28),
            Self::Light => Color::from_rgb8(0xE0, 0xDD, 0xD5),
        }
    }

    /// Hover background for clickable rows.
    pub fn hover_bg(self) -> Color {
        match self {
            Self::Dark => Color::from_rgb8(0x37, 0x34, 0x30),
            Self::Light => Color::from_rgb8(0xD2, 0xCF, 0xC6),
        }
    }

    /// Active / selected background for nav rows and lists.
    pub fn active_bg(self) -> Color {
        match self {
            Self::Dark => Color::from_rgb8(0x41, 0x3E, 0x3A),
            Self::Light => Color::from_rgb8(0xC4, 0xC0, 0xB6),
        }
    }

    /// Primary text colour. Reads on `bg`, `section_bg`, `nav_bg`.
    pub fn text(self) -> Color {
        match self {
            Self::Dark => Color::from_rgb8(0xDC, 0xDA, 0xD6),
            Self::Light => Color::from_rgb8(0x14, 0x14, 0x14),
        }
    }

    /// Muted secondary text — captions and de-emphasised labels.
    pub fn text_secondary(self) -> Color {
        match self {
            Self::Dark => Color::from_rgb8(0xA0, 0x9C, 0x96),
            Self::Light => Color::from_rgb8(0x55, 0x55, 0x55),
        }
    }

    /// Default checkbox palette for this theme. The classic
    /// checkmark control needs its border / mark / label all
    /// flipped between modes; this packages the tuned values so
    /// callers don't need to re-derive them.
    pub fn checkbox_colors(self) -> CheckboxColors {
        match self {
            Self::Dark => CheckboxColors::custom(
                AlphaColor::new([0.0, 0.0, 0.0, 0.0]),
                AlphaColor::new([0.5, 0.5, 0.5, 1.0]),
                AlphaColor::new([1.0, 1.0, 1.0, 1.0]),
                self.text(),
            ),
            Self::Light => CheckboxColors::custom(
                AlphaColor::new([1.0, 1.0, 1.0, 1.0]),
                AlphaColor::new([0.4, 0.4, 0.4, 1.0]),
                AlphaColor::new([0.0, 0.0, 0.0, 1.0]),
                self.text(),
            ),
        }
    }
}
