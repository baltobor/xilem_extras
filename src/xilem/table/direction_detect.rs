//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Auto-detects table layout direction from column header titles.
//!
//! Reuses `xilem::masonry::parley` (already re-exported through
//! masonry_core → masonry → xilem) rather than adding a direct `parley`
//! dependency, so this stays a zero-new-import addition to the stack — worth
//! keeping in mind since these widgets are meant to eventually be proposed
//! upstream to xilem itself.

use std::sync::{Mutex, OnceLock};

use xilem::masonry::parley::{FontContext, LayoutContext};

use crate::masonry::flow_direction::FlowDirection;

type Cache = Mutex<(FontContext, LayoutContext<()>)>;

fn cache() -> &'static Cache {
    static CACHE: OnceLock<Cache> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new((FontContext::new(), LayoutContext::new())))
}

fn is_rtl_text(fcx: &mut FontContext, lcx: &mut LayoutContext<()>, text: &str) -> bool {
    if text.trim().is_empty() {
        return false;
    }
    let layout = lcx.ranged_builder(fcx, text, 1.0, false).build(text);
    layout.is_rtl()
}

/// Detects the dominant layout direction from a set of column header titles.
///
/// Only header titles are ever inspected — never row/cell content — so an
/// Arabic-context table stays right-to-left even if its data happens to be
/// non-Arabic. Defaults to [`FlowDirection::Ltr`] unless a strict majority of
/// titles resolve to right-to-left text via parley's bidi analysis (ties,
/// all-neutral, or empty input stay LTR).
pub(crate) fn detect_direction<'a>(titles: impl Iterator<Item = &'a str>) -> FlowDirection {
    let cache = cache();
    let mut guard = cache.lock().unwrap_or_else(|poison| poison.into_inner());
    let (fcx, lcx) = &mut *guard;

    let mut rtl_count = 0usize;
    let mut ltr_count = 0usize;
    for title in titles {
        if is_rtl_text(fcx, lcx, title) {
            rtl_count += 1;
        } else if !title.trim().is_empty() {
            ltr_count += 1;
        }
    }

    if rtl_count > ltr_count {
        FlowDirection::Rtl
    } else {
        FlowDirection::Ltr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_defaults_to_ltr() {
        assert_eq!(detect_direction(std::iter::empty()), FlowDirection::Ltr);
    }

    #[test]
    fn latin_titles_are_ltr() {
        let titles = ["Name", "Age", "City"];
        assert_eq!(detect_direction(titles.into_iter()), FlowDirection::Ltr);
    }

    #[test]
    fn arabic_majority_is_rtl() {
        let titles = ["الاسم", "العمر", "المدينة"];
        assert_eq!(detect_direction(titles.into_iter()), FlowDirection::Rtl);
    }

    #[test]
    fn mixed_latin_majority_is_ltr() {
        let titles = ["Name", "Age", "المدينة"];
        assert_eq!(detect_direction(titles.into_iter()), FlowDirection::Ltr);
    }
}
