//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Keyboard shortcut types for application menu items.
//!
//! Provides a DSL for defining shortcuts: `CMD + SHIFT + Key::Z`

use std::ops::Add;

/// Keyboard key for shortcuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    // Numbers
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    // Special keys
    Escape, Tab, Space, Enter, Backspace, Delete,
    Left, Right, Up, Down,
    Home, End, PageUp, PageDown,
    // Punctuation
    Comma, Period, Slash, Backslash, Semicolon, Quote,
    BracketLeft, BracketRight, Minus, Equal, Grave,
}

impl Key {
    /// Convert to muda accelerator string representation.
    #[cfg(not(target_os = "linux"))]
    pub fn to_accelerator_key(&self) -> &'static str {
        match self {
            Key::A => "A", Key::B => "B", Key::C => "C", Key::D => "D",
            Key::E => "E", Key::F => "F", Key::G => "G", Key::H => "H",
            Key::I => "I", Key::J => "J", Key::K => "K", Key::L => "L",
            Key::M => "M", Key::N => "N", Key::O => "O", Key::P => "P",
            Key::Q => "Q", Key::R => "R", Key::S => "S", Key::T => "T",
            Key::U => "U", Key::V => "V", Key::W => "W", Key::X => "X",
            Key::Y => "Y", Key::Z => "Z",
            Key::Num0 => "0", Key::Num1 => "1", Key::Num2 => "2",
            Key::Num3 => "3", Key::Num4 => "4", Key::Num5 => "5",
            Key::Num6 => "6", Key::Num7 => "7", Key::Num8 => "8", Key::Num9 => "9",
            Key::F1 => "F1", Key::F2 => "F2", Key::F3 => "F3", Key::F4 => "F4",
            Key::F5 => "F5", Key::F6 => "F6", Key::F7 => "F7", Key::F8 => "F8",
            Key::F9 => "F9", Key::F10 => "F10", Key::F11 => "F11", Key::F12 => "F12",
            Key::Escape => "Escape", Key::Tab => "Tab", Key::Space => "Space",
            Key::Enter => "Enter", Key::Backspace => "Backspace", Key::Delete => "Delete",
            Key::Left => "Left", Key::Right => "Right", Key::Up => "Up", Key::Down => "Down",
            Key::Home => "Home", Key::End => "End", Key::PageUp => "PageUp", Key::PageDown => "PageDown",
            Key::Comma => ",", Key::Period => ".", Key::Slash => "/", Key::Backslash => "\\",
            Key::Semicolon => ";", Key::Quote => "'", Key::BracketLeft => "[",
            Key::BracketRight => "]", Key::Minus => "-", Key::Equal => "=", Key::Grave => "`",
        }
    }

    /// Display name for the key (used in fallback menu).
    pub fn display_name(&self) -> &'static str {
        match self {
            Key::A => "A", Key::B => "B", Key::C => "C", Key::D => "D",
            Key::E => "E", Key::F => "F", Key::G => "G", Key::H => "H",
            Key::I => "I", Key::J => "J", Key::K => "K", Key::L => "L",
            Key::M => "M", Key::N => "N", Key::O => "O", Key::P => "P",
            Key::Q => "Q", Key::R => "R", Key::S => "S", Key::T => "T",
            Key::U => "U", Key::V => "V", Key::W => "W", Key::X => "X",
            Key::Y => "Y", Key::Z => "Z",
            Key::Num0 => "0", Key::Num1 => "1", Key::Num2 => "2",
            Key::Num3 => "3", Key::Num4 => "4", Key::Num5 => "5",
            Key::Num6 => "6", Key::Num7 => "7", Key::Num8 => "8", Key::Num9 => "9",
            Key::F1 => "F1", Key::F2 => "F2", Key::F3 => "F3", Key::F4 => "F4",
            Key::F5 => "F5", Key::F6 => "F6", Key::F7 => "F7", Key::F8 => "F8",
            Key::F9 => "F9", Key::F10 => "F10", Key::F11 => "F11", Key::F12 => "F12",
            Key::Escape => "Esc", Key::Tab => "Tab", Key::Space => "Space",
            Key::Enter => "Enter", Key::Backspace => "Backspace", Key::Delete => "Del",
            Key::Left => "Left", Key::Right => "Right", Key::Up => "Up", Key::Down => "Down",
            Key::Home => "Home", Key::End => "End", Key::PageUp => "PgUp", Key::PageDown => "PgDn",
            Key::Comma => ",", Key::Period => ".", Key::Slash => "/", Key::Backslash => "\\",
            Key::Semicolon => ";", Key::Quote => "'", Key::BracketLeft => "[",
            Key::BracketRight => "]", Key::Minus => "-", Key::Equal => "=", Key::Grave => "`",
        }
    }
}

/// Modifier keys for shortcuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Modifiers {
    /// Command on macOS, Ctrl on Windows/Linux (platform-aware primary modifier)
    pub cmd: bool,
    /// Shift key
    pub shift: bool,
    /// Alt/Option key
    pub alt: bool,
    /// Control key (always Ctrl, even on macOS)
    pub ctrl: bool,
}

impl Modifiers {
    /// No modifiers.
    pub const NONE: Modifiers = Modifiers {
        cmd: false,
        shift: false,
        alt: false,
        ctrl: false,
    };

    /// Check if no modifiers are set.
    pub fn is_empty(&self) -> bool {
        !self.cmd && !self.shift && !self.alt && !self.ctrl
    }

    /// Display string for modifiers (e.g., "Cmd+Shift+").
    pub fn display_prefix(&self) -> String {
        let mut result = String::new();
        if self.ctrl {
            result.push_str("Ctrl+");
        }
        if self.cmd {
            #[cfg(target_os = "macos")]
            result.push_str("Cmd+");
            #[cfg(not(target_os = "macos"))]
            result.push_str("Ctrl+");
        }
        if self.alt {
            #[cfg(target_os = "macos")]
            result.push_str("Opt+");
            #[cfg(not(target_os = "macos"))]
            result.push_str("Alt+");
        }
        if self.shift {
            result.push_str("Shift+");
        }
        result
    }
}

/// Platform-aware command modifier (Cmd on macOS, Ctrl on Windows/Linux).
pub const CMD: Modifiers = Modifiers {
    cmd: true,
    shift: false,
    alt: false,
    ctrl: false,
};

/// Shift modifier.
pub const SHIFT: Modifiers = Modifiers {
    cmd: false,
    shift: true,
    alt: false,
    ctrl: false,
};

/// Alt/Option modifier.
pub const ALT: Modifiers = Modifiers {
    cmd: false,
    shift: false,
    alt: true,
    ctrl: false,
};

/// Control modifier (always Ctrl, not platform-aware).
pub const CTRL: Modifiers = Modifiers {
    cmd: false,
    shift: false,
    alt: false,
    ctrl: true,
};

/// Combine two modifiers.
impl Add for Modifiers {
    type Output = Modifiers;

    fn add(self, rhs: Modifiers) -> Modifiers {
        Modifiers {
            cmd: self.cmd || rhs.cmd,
            shift: self.shift || rhs.shift,
            alt: self.alt || rhs.alt,
            ctrl: self.ctrl || rhs.ctrl,
        }
    }
}

/// A keyboard shortcut combining modifiers and a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Shortcut {
    pub modifiers: Modifiers,
    pub key: Key,
}

impl Shortcut {
    /// Create a new shortcut.
    pub fn new(modifiers: Modifiers, key: Key) -> Self {
        Self { modifiers, key }
    }

    /// Display string for the shortcut (e.g., "Cmd+Shift+Z").
    pub fn display(&self) -> String {
        format!("{}{}", self.modifiers.display_prefix(), self.key.display_name())
    }

    /// Convert to muda accelerator string.
    #[cfg(not(target_os = "linux"))]
    pub fn to_accelerator(&self) -> String {
        let mut parts = Vec::new();

        if self.modifiers.ctrl {
            parts.push("Control");
        }
        if self.modifiers.cmd {
            #[cfg(target_os = "macos")]
            parts.push("Command");
            #[cfg(not(target_os = "macos"))]
            parts.push("Control");
        }
        if self.modifiers.alt {
            parts.push("Alt");
        }
        if self.modifiers.shift {
            parts.push("Shift");
        }

        parts.push(self.key.to_accelerator_key());
        parts.join("+")
    }
}

/// Create a shortcut from modifiers + key.
impl Add<Key> for Modifiers {
    type Output = Shortcut;

    fn add(self, key: Key) -> Shortcut {
        Shortcut::new(self, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_dsl() {
        let shortcut = CMD + Key::N;
        assert!(shortcut.modifiers.cmd);
        assert!(!shortcut.modifiers.shift);
        assert_eq!(shortcut.key, Key::N);
    }

    #[test]
    fn test_modifier_combination() {
        let mods = CMD + SHIFT;
        assert!(mods.cmd);
        assert!(mods.shift);
        assert!(!mods.alt);

        let shortcut = mods + Key::Z;
        assert!(shortcut.modifiers.cmd);
        assert!(shortcut.modifiers.shift);
        assert_eq!(shortcut.key, Key::Z);
    }

    #[test]
    fn test_shortcut_display() {
        let shortcut = CMD + Key::S;
        let display = shortcut.display();
        // Platform-dependent, but should contain "S"
        assert!(display.contains("S"));
    }
}
