//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

use std::marker::PhantomData;

/// A section with a header and content.
///
/// Sections are used to group list items with a visual header.
pub struct Section<State, Action, H, C> {
    header: H,
    content: C,
    _phantom: PhantomData<(State, Action)>,
}

impl<State, Action, H, C> Section<State, Action, H, C> {
    /// Creates a new section.
    pub fn new(header: H, content: C) -> Self {
        Self {
            header,
            content,
            _phantom: PhantomData,
        }
    }
}

/// Creates a section with a header and content.
///
/// # Example
///
/// ```ignore
/// section(
///     label("Favorites").bold(),
///     flex_col(
///         model.favorites.iter().map(|item| {
///             list_row(item)
///         })
///     ),
/// )
/// ```
pub fn section<State, Action, H, C>(header: H, content: C) -> Section<State, Action, H, C> {
    Section::new(header, content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_creation() {
        let _s: Section<(), (), &str, &str> = section("Header", "Content");
    }
}
