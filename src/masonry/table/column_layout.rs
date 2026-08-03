// Copyright 2026 the Xilem Authors
// SPDX-License-Identifier: Apache-2.0

//! Column width/position math shared by `ResizableHeader`, `TableWidget`,
//! and `TableView` — consolidated here so the three don't each carry a
//! subtly-different copy of the same formulas, which is exactly what caused
//! earlier drift bugs (the header rendering a frame ahead of the
//! highlight/rows, cursor/hit-test mismatches).

use std::sync::Arc;

use crate::masonry::flow_direction::FlowDirection;

pub(crate) const MIN_COLUMN_WIDTH: f64 = 40.0;
pub(crate) const DIVIDER_WIDTH: f64 = 2.0;
pub(crate) const DIVIDER_HIT_AREA: f64 = 8.0;

/// How a table's columns behave when the sum of their configured widths
/// would exceed the available viewport width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColumnResizeMode {
    /// Columns always render at their configured width; if the total
    /// exceeds the viewport, the table simply overflows (typically paired
    /// with wrapping the table in a horizontal-scrolling `portal(...)`).
    /// Matches Apple Numbers' actual resize behavior.
    #[default]
    Overflow,
    /// The table never exceeds its container: resizing a column compresses
    /// the columns *after* it (never before) down to `MIN_COLUMN_WIDTH`
    /// each, and further growth simply stops once they're all at that
    /// floor. Matches AG Grid's per-column `suppressSizeToFit`-style "fit
    /// to viewport" behavior.
    FixedViewport,
}

/// A column's resolved position and width, ready to place/paint.
#[derive(Debug, Clone)]
pub(crate) struct ColumnBox {
    pub key: Arc<str>,
    pub width: f64,
    pub x_offset: f64,
}

/// Maps a data-order column index to its visual (screen) position, or vice
/// versa — the mapping is self-inverse (applying it twice returns the
/// original index), so this one function serves both directions.
///
/// LTR: identity (visual order matches data order, visual position 0 is
/// leftmost). RTL: reversed (data index 0 — the first column — renders
/// rightmost, matching "the first column is read first in RTL, which
/// starts from the right").
///
/// Public so callers building row content (e.g. a `table_styled` row
/// builder) can reorder their own cells to match the header's visual
/// order — the header and row widgets are laid out independently, so
/// nothing does this for them automatically.
pub fn visual_index(idx: usize, n: usize, direction: FlowDirection) -> usize {
    match direction {
        FlowDirection::Ltr => idx,
        FlowDirection::Rtl => n - 1 - idx,
    }
}

/// The single point-reflection primitive underlying every RTL mirror in
/// this module — "the same basis with the x-axis pointing the other way."
/// LTR: identity. RTL: reflects a point around `anchor_width`.
///
/// Nothing else in this module independently re-derives RTL mirroring:
/// `place_columns` applies this to a column's *local* (direction-agnostic,
/// always-left-to-right-accumulated) edge to get its screen edge;
/// `divider_start` and `flip_delta` are both provably consequences of the
/// same reflection (see their doc comments) rather than separately
/// maintained formulas. That's what keeps the header, the row-width
/// computation, and hit-testing from being able to silently drift apart —
/// they all bottom out at this one function.
fn to_screen(local_x: f64, anchor_width: f64, direction: FlowDirection) -> f64 {
    match direction {
        FlowDirection::Ltr => local_x,
        FlowDirection::Rtl => anchor_width - local_x,
    }
}

/// How a *delta* (the distance the pointer has moved during a drag) maps
/// through the same reflection as `to_screen`. `to_screen` is affine with
/// slope `+1` (LTR) or `-1` (RTL); a delta is a difference between two
/// points, so it inherits exactly that slope — unchanged in LTR, negated
/// in RTL. This is not an independent sign choice: it is *required* to be
/// consistent with `to_screen`, or the divider would stop tracking the
/// cursor 1:1 (verified by hand and by `column_layout`'s own tests —
/// see the plan file for the full derivation of why RTL needs exactly this
/// sign, no more, no less).
pub(crate) fn flip_delta(local_delta: f64, direction: FlowDirection) -> f64 {
    match direction {
        FlowDirection::Ltr => local_delta,
        FlowDirection::Rtl => -local_delta,
    }
}

/// Computes each column's `x_offset` (and passes through `width`) given
/// per-column widths and the anchor width of the box they're placed in.
///
/// Columns accumulate in **local** coordinates identically for both
/// directions — this loop never branches on content, only on which local
/// edge becomes the screen-space left edge. A column occupies the local
/// interval `[local_x, local_x + width)`; reflecting an interval through
/// `to_screen` swaps its endpoints (the local *right* edge becomes the
/// screen *left* edge) in RTL, while LTR's `to_screen` is the identity, so
/// the local left edge already *is* the screen left edge. Concretely:
/// `x_offset(RTL) = to_screen(local_x + width) = anchor_width - local_x -
/// width` — `width` appears *inside* the reflection because it's part of
/// which endpoint gets reflected, not a separate adjustment. This is what
/// lets "divider `i` always resizes data column `i`" (the same rule as
/// LTR, unconditionally, no remapping) coexist with correct 1:1 cursor
/// tracking (`flip_delta` above). An earlier version of this function
/// instead reordered the *iteration* to place columns by visual position —
/// that made a column's leading edge independent of its own width, which
/// structurally prevents the lower-data-index column of any divider from
/// ever being draggable with correct cursor tracking. Reverted; see the
/// plan file for the full derivation.
pub(crate) fn place_columns(
    keys: &[Arc<str>],
    widths: &[f64],
    anchor_width: f64,
    direction: FlowDirection,
) -> Vec<ColumnBox> {
    let n = keys.len();
    let mut columns = Vec::with_capacity(n);
    let mut local_x = 0.0;
    for (i, key) in keys.iter().enumerate() {
        let width = widths.get(i).copied().unwrap_or(100.0);
        let local_left = local_x;
        let local_right = local_x + width;
        let x_offset = match direction {
            FlowDirection::Ltr => to_screen(local_left, anchor_width, direction),
            FlowDirection::Rtl => to_screen(local_right, anchor_width, direction),
        };
        columns.push(ColumnBox {
            key: key.clone(),
            width,
            x_offset,
        });
        if i < n - 1 {
            local_x += width + DIVIDER_WIDTH;
        } else {
            local_x += width;
        }
    }
    columns
}

/// The on-screen x-position of the divider just after `col` (i.e. the
/// boundary a user grabs to resize it).
///
/// In LTR, the divider is `col`'s own right edge — `to_screen` is the
/// identity, so this is just `x_offset + width`. In RTL, `col.x_offset`
/// (per `place_columns`) already equals `to_screen` of `col`'s local right
/// edge; the divider sits `DIVIDER_WIDTH` further in the local-forward
/// direction from that same edge, which — having already been reflected —
/// reads as screen-*left*: `x_offset - DIVIDER_WIDTH`. Both branches
/// describe the same reflected boundary; neither is an independently
/// chosen formula.
pub(crate) fn divider_start(col: &ColumnBox, direction: FlowDirection) -> f64 {
    match direction {
        FlowDirection::Ltr => col.x_offset + col.width,
        FlowDirection::Rtl => col.x_offset - DIVIDER_WIDTH,
    }
}

/// The maximum width `dragged_idx` can grow to in `FixedViewport` mode
/// before the columns after it would need to shrink past their own floor.
/// Shared by `compute_rendered_widths`'s cap branch and by the live drag
/// handler in `ResizableHeader`, so the *drag itself* (the raw, uncapped
/// delta accumulated from pointer movement) stops growing exactly where
/// rendering would have capped it anyway — not just the visual output,
/// which would otherwise let the pointer keep moving indefinitely while the
/// header silently desyncs from the (correctly-capped) row content.
pub(crate) fn max_dragged_width(configured: &[f64], dragged_idx: usize, available_width: f64) -> f64 {
    let n = configured.len();
    if n == 0 {
        return MIN_COLUMN_WIDTH;
    }
    let dragged_idx = dragged_idx.min(n - 1);
    let divider_space = n.saturating_sub(1) as f64 * DIVIDER_WIDTH;
    let before_total: f64 = configured[..dragged_idx].iter().sum();
    let after_len = n - dragged_idx - 1;
    let after_min_total = after_len as f64 * MIN_COLUMN_WIDTH;
    (available_width - divider_space - before_total - after_min_total).max(MIN_COLUMN_WIDTH)
}

/// Distributes `budget` proportionally across `desired`, flooring each
/// entry at `floor`, via water-filling: a naive single-pass `w * scale`
/// can push a narrower entry below `floor` while a wider one still has
/// slack, which silently breaks the total-equals-budget invariant (the
/// narrow one gets clamped up to `floor`, past what the scale allowed, and
/// nothing gives that space back). Water-filling pins any entry that would
/// go below `floor` right at `floor`, removes it from the pool, and
/// re-derives the scale for the remaining entries against the remaining
/// budget — repeating until stable. If `budget` can't even cover
/// `desired.len() * floor`, every entry is simply floored (the caller is
/// expected to have already capped the driving/dragged column so this
/// doesn't happen, but it degrades safely either way).
fn distribute_with_floor(desired: &[f64], budget: f64, floor: f64) -> Vec<f64> {
    let n = desired.len();
    let mut result = vec![0.0; n];
    let mut pinned = vec![false; n];
    let mut remaining_budget = budget;
    let mut remaining_total: f64 = desired.iter().sum();

    loop {
        if remaining_total <= 0.0 {
            break;
        }
        let scale = (remaining_budget / remaining_total).min(1.0);
        let mut newly_pinned = false;
        for i in 0..n {
            if pinned[i] {
                continue;
            }
            let scaled = desired[i] * scale;
            if scaled < floor {
                result[i] = floor;
                pinned[i] = true;
                remaining_budget -= floor;
                remaining_total -= desired[i];
                newly_pinned = true;
            } else {
                result[i] = scaled;
            }
        }
        if !newly_pinned {
            break;
        }
    }
    result
}

/// Resolves each column's *rendered* width from its *configured* (desired)
/// width, the column currently being interactively dragged (if any), the
/// available viewport width, and the resize mode.
///
/// Pure function of current state, recomputed fresh each call — not
/// incremental deltas — so e.g. shrinking a dragged column back always
/// restores previously-compressed columns immediately, with no "ratchet"
/// state to get out of sync.
///
/// `dragged_idx` is the index of the column currently being interactively
/// resized, if any — `configured[dragged_idx]` is already its live
/// (uncapped) drag target, kept up to date by the caller on every
/// pointer-move; everything before it is protected (untouched), everything
/// after it compresses to make room. When `None` (no active drag — initial
/// layout, or right after a commit, where only the just-dragged column's
/// desired width was persisted and the rest may momentarily not all fit
/// together), no column is arbitrarily protected — all columns compress
/// together proportionally instead, since there's no drag in progress to
/// justify treating any one of them as a fixed pivot.
pub(crate) fn compute_rendered_widths(
    configured: &[f64],
    dragged_idx: Option<usize>,
    available_width: f64,
    mode: ColumnResizeMode,
) -> Vec<f64> {
    match mode {
        ColumnResizeMode::Overflow => configured
            .iter()
            .map(|&w| w.max(MIN_COLUMN_WIDTH))
            .collect(),
        ColumnResizeMode::FixedViewport => {
            let n = configured.len();
            if n == 0 {
                return Vec::new();
            }
            let divider_space = n.saturating_sub(1) as f64 * DIVIDER_WIDTH;

            let Some(dragged_idx) = dragged_idx else {
                // No active drag: fit everything within the viewport
                // together, proportionally, with no protected column.
                let total_desired: f64 =
                    configured.iter().map(|&w| w.max(MIN_COLUMN_WIDTH)).sum::<f64>()
                        + divider_space;
                if total_desired <= available_width + 0.5 {
                    return configured.iter().map(|&w| w.max(MIN_COLUMN_WIDTH)).collect();
                }
                return distribute_with_floor(
                    configured,
                    available_width - divider_space,
                    MIN_COLUMN_WIDTH,
                );
            };
            let dragged_idx = dragged_idx.min(n - 1);
            let dragged_width = configured[dragged_idx].max(MIN_COLUMN_WIDTH);

            let before_total: f64 = configured[..dragged_idx].iter().sum();
            let after = &configured[dragged_idx + 1..];
            let after_total: f64 = after.iter().sum();

            let mut result = configured.to_vec();
            result[dragged_idx] = dragged_width;

            let total_desired = before_total + dragged_width + after_total + divider_space;
            if total_desired <= available_width + 0.5 {
                // Everything fits at its desired width.
                return result;
            }

            if after.is_empty() {
                // Nothing after the dragged column to compress — cap the
                // drag itself against whatever room is left.
                result[dragged_idx] =
                    (available_width - divider_space - before_total).max(MIN_COLUMN_WIDTH);
                return result;
            }

            let budget_for_after = available_width - divider_space - before_total - dragged_width;
            let after_min_total = after.len() as f64 * MIN_COLUMN_WIDTH;
            if budget_for_after >= after_min_total && after_total > 0.0 {
                // Compress the after-columns to fit the budget, never
                // below their own floor, preserving the total exactly even
                // when after-columns have unequal desired widths (water-
                // filling — see `distribute_with_floor`).
                let compressed = distribute_with_floor(after, budget_for_after, MIN_COLUMN_WIDTH);
                for (i, w) in compressed.into_iter().enumerate() {
                    result[dragged_idx + 1 + i] = w;
                }
            } else {
                // Even at everyone's floor there's no room — cap the
                // dragged column's growth instead and floor every
                // after-column.
                for i in 0..after.len() {
                    result[dragged_idx + 1 + i] = MIN_COLUMN_WIDTH;
                }
                result[dragged_idx] =
                    (available_width - divider_space - before_total - after_min_total)
                        .max(MIN_COLUMN_WIDTH);
            }

            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(n: usize) -> Vec<Arc<str>> {
        (0..n).map(|i| Arc::from(format!("col{i}"))).collect()
    }

    #[test]
    fn visual_index_ltr_is_identity() {
        assert_eq!(visual_index(0, 4, FlowDirection::Ltr), 0);
        assert_eq!(visual_index(3, 4, FlowDirection::Ltr), 3);
    }

    #[test]
    fn visual_index_rtl_reverses_and_is_self_inverse() {
        // Data column 0 (the first column) renders in the last (rightmost)
        // visual slot.
        assert_eq!(visual_index(0, 4, FlowDirection::Rtl), 3);
        assert_eq!(visual_index(3, 4, FlowDirection::Rtl), 0);
        // Self-inverse: applying it twice returns the original index.
        for i in 0..4 {
            let v = visual_index(i, 4, FlowDirection::Rtl);
            assert_eq!(visual_index(v, 4, FlowDirection::Rtl), i);
        }
    }

    #[test]
    fn flip_delta_matches_to_screen_slope() {
        // `flip_delta` must always agree with `to_screen`'s own slope:
        // reflecting two points and taking their difference must equal
        // `flip_delta` of the original difference, for any pair of points
        // and any anchor. This is the safety net for the "single affine
        // transform, not independently re-derived formulas" invariant —
        // if a future edit changes one without the other, this fails.
        for &(a, b) in &[(10.0, 25.0), (0.0, 100.0), (334.0, 50.0)] {
            for &anchor in &[200.0, 800.0] {
                let local_delta = b - a;
                for &direction in &[FlowDirection::Ltr, FlowDirection::Rtl] {
                    let screen_delta =
                        to_screen(b, anchor, direction) - to_screen(a, anchor, direction);
                    assert_eq!(screen_delta, flip_delta(local_delta, direction));
                }
            }
        }
    }

    #[test]
    fn divider_start_matches_place_columns_reflection() {
        // `divider_start`'s two branches are provably consequences of
        // `place_columns`'s own interval reflection, not independently
        // maintained — checked here directly by comparing against a
        // from-scratch `to_screen` computation of the same local boundary,
        // for several column counts and widths, in both directions.
        for &widths in &[
            &[100.0, 150.0, 80.0][..],
            &[200.0, 200.0, 100.0, 60.0][..],
            &[40.0, 40.0, 40.0][..],
        ] {
            for &anchor in &[334.0, 800.0, 500.0] {
                for &direction in &[FlowDirection::Ltr, FlowDirection::Rtl] {
                    let n = widths.len();
                    let columns = place_columns(&keys(n), widths, anchor, direction);
                    // Recompute each divider's local boundary independently
                    // (sum of widths + dividers up to and including column
                    // i) and reflect it through `to_screen` directly.
                    let mut local_x = 0.0;
                    for i in 0..n - 1 {
                        local_x += widths[i];
                        let local_divider_gap_start = local_x;
                        local_x += DIVIDER_WIDTH;
                        let expected = match direction {
                            FlowDirection::Ltr => {
                                to_screen(local_divider_gap_start, anchor, direction)
                            }
                            FlowDirection::Rtl => {
                                to_screen(local_divider_gap_start + DIVIDER_WIDTH, anchor, direction)
                            }
                        };
                        assert_eq!(divider_start(&columns[i], direction), expected);
                    }
                }
            }
        }
    }

    #[test]
    fn place_columns_rtl_gallery_baseline() {
        // 4 columns (Name, Route, Distance, Joy) at [200,200,100,60],
        // anchor=800 (fits with 234px leftover, trailing left in RTL).
        let columns = place_columns(
            &keys(4),
            &[200.0, 200.0, 100.0, 60.0],
            800.0,
            FlowDirection::Rtl,
        );
        assert_eq!(columns[0].x_offset, 600.0); // Name: rightmost
        assert_eq!(columns[1].x_offset, 398.0); // Route
        assert_eq!(columns[2].x_offset, 296.0); // Distance
        assert_eq!(columns[3].x_offset, 234.0); // Joy: leftmost
    }

    #[test]
    fn place_columns_rtl_dragging_name_pushes_everything_else() {
        // Final user-confirmed spec: divider 0 (between data columns 0 and
        // 1, i.e. Name/Route — the same unconditional "divider i resizes
        // column i" rule as LTR) grows Name; Route/Distance/Joy (all
        // "after", per the same before/after rule as LTR) shift left by
        // exactly Name's growth, with no sign-flip-induced surprises once
        // combined with `ResizableHeader`'s RTL delta flip.
        let before = place_columns(
            &keys(4),
            &[200.0, 200.0, 100.0, 60.0],
            800.0,
            FlowDirection::Rtl,
        );
        let after = place_columns(
            &keys(4),
            &[253.0, 200.0, 100.0, 60.0], // Name grown by 53
            800.0,
            FlowDirection::Rtl,
        );
        assert_eq!(after[1].x_offset, before[1].x_offset - 53.0); // Route shifted left
        assert_eq!(after[2].x_offset, before[2].x_offset - 53.0); // Distance shifted left
        assert_eq!(after[3].x_offset, before[3].x_offset - 53.0); // Joy shifted left
        assert_eq!(after[1].width, before[1].width); // shifted, not resized
        // The Name/Route divider moved left by exactly the growth amount —
        // tracks the cursor when combined with the RTL sign flip (mouse
        // left → raw_delta negative → signed_delta positive → grows).
        assert_eq!(
            divider_start(&after[0], FlowDirection::Rtl),
            divider_start(&before[0], FlowDirection::Rtl) - 53.0
        );
    }

    #[test]
    fn place_columns_rtl_self_referential_anchor_breaks_divider_tracking() {
        // Documents the root cause of a real bug (see plan file): in
        // `Overflow` mode, `ResizableHeader` is typically hosted in a
        // content-sized `portal(...)`, so its own `size.width` — the
        // anchor normally passed to `place_columns` — is *itself* the sum
        // of the very column widths being placed. If the anchor is
        // recomputed as exactly that sum on every call (i.e. NOT frozen
        // for the duration of a drag, which is the actual fix in
        // `ResizableHeader`/`TableWidget`), the dragged column's own width
        // cancels out of its own divider position entirely — verified
        // here directly against the pure function, independent of any
        // widget state.
        let before_widths = [200.0, 200.0, 100.0, 60.0];
        let before_anchor: f64 = before_widths.iter().sum::<f64>() + 3.0 * DIVIDER_WIDTH;
        let before = place_columns(&keys(4), &before_widths, before_anchor, FlowDirection::Rtl);

        let after_widths = [253.0, 200.0, 100.0, 60.0]; // Name grown by 53
        let after_anchor: f64 = after_widths.iter().sum::<f64>() + 3.0 * DIVIDER_WIDTH; // self-referential
        let after = place_columns(&keys(4), &after_widths, after_anchor, FlowDirection::Rtl);

        // With a self-referential anchor, the Name/Route divider does NOT
        // move at all despite Name growing by 53 — this is the bug, not
        // the desired behavior (contrast with the *frozen*-anchor test
        // above, `place_columns_rtl_dragging_name_pushes_everything_else`,
        // which uses the same fixed anchor for both calls and correctly
        // shows the divider moving by exactly 53).
        assert_eq!(
            divider_start(&after[0], FlowDirection::Rtl),
            divider_start(&before[0], FlowDirection::Rtl)
        );
    }

    #[test]
    fn place_columns_rtl_dragging_route_pushes_distance_and_joy_not_name() {
        // Same rule applied to divider 1 (Route, data idx 1): Name (data
        // idx 0, "before") stays completely untouched; Distance/Joy
        // ("after") shift left.
        let before = place_columns(
            &keys(4),
            &[200.0, 200.0, 100.0, 60.0],
            800.0,
            FlowDirection::Rtl,
        );
        let after = place_columns(
            &keys(4),
            &[200.0, 230.0, 100.0, 60.0], // Route grown by 30
            800.0,
            FlowDirection::Rtl,
        );
        assert_eq!(after[0].x_offset, before[0].x_offset); // Name untouched
        assert_eq!(after[0].width, before[0].width);
        assert_eq!(after[2].x_offset, before[2].x_offset - 30.0); // Distance shifted left
        assert_eq!(after[3].x_offset, before[3].x_offset - 30.0); // Joy shifted left
    }

    #[test]
    fn max_dragged_width_matches_compute_rendered_widths_cap() {
        // Same scenario as `fixed_viewport_caps_drag_once_after_columns_hit_floor`:
        // available=334, dragging column 0, after-columns [150,80] floored at
        // 40 each — the drag itself should stop at exactly the width that
        // computation caps rendering to (250), not keep growing unboundedly.
        let max = max_dragged_width(&[100_000.0, 150.0, 80.0], 0, 334.0);
        assert_eq!(max, 250.0);
    }

    #[test]
    fn place_columns_ltr_accumulates_left_to_right() {
        let widths = [100.0, 150.0, 80.0];
        let columns = place_columns(&keys(3), &widths, 334.0, FlowDirection::Ltr);
        assert_eq!(columns[0].x_offset, 0.0);
        assert_eq!(columns[1].x_offset, 102.0);
        assert_eq!(columns[2].x_offset, 254.0);
    }

    #[test]
    fn place_columns_rtl_mirrors_from_anchor_width() {
        let widths = [100.0, 150.0, 80.0];
        let columns = place_columns(&keys(3), &widths, 334.0, FlowDirection::Rtl);
        assert_eq!(columns[0].x_offset, 234.0);
        assert_eq!(columns[1].x_offset, 82.0);
        assert_eq!(columns[2].x_offset, 0.0);
    }

    #[test]
    fn divider_start_ltr_is_column_right_edge() {
        let columns = place_columns(&keys(3), &[100.0, 150.0, 80.0], 334.0, FlowDirection::Ltr);
        assert_eq!(divider_start(&columns[0], FlowDirection::Ltr), 100.0);
    }

    #[test]
    fn divider_start_rtl_is_column_left_edge() {
        let columns = place_columns(&keys(3), &[100.0, 150.0, 80.0], 334.0, FlowDirection::Rtl);
        assert_eq!(divider_start(&columns[0], FlowDirection::Rtl), 232.0);
    }

    #[test]
    fn overflow_mode_never_shrinks() {
        let widths =
            compute_rendered_widths(&[300.0, 150.0, 80.0], None, 200.0, ColumnResizeMode::Overflow);
        assert_eq!(widths, vec![300.0, 150.0, 80.0]);
    }

    #[test]
    fn fixed_viewport_fits_without_compression_when_there_is_room() {
        let widths = compute_rendered_widths(
            &[120.0, 150.0, 80.0],
            Some(0),
            400.0,
            ColumnResizeMode::FixedViewport,
        );
        assert_eq!(widths, vec![120.0, 150.0, 80.0]);
    }

    #[test]
    fn fixed_viewport_compresses_only_after_columns() {
        // available=334 exactly fits [100,150,80] + 2 dividers. Grow col0 to 150 (+50).
        let widths = compute_rendered_widths(
            &[150.0, 150.0, 80.0],
            Some(0),
            334.0,
            ColumnResizeMode::FixedViewport,
        );
        let expected_scale = 180.0 / 230.0; // budget_for_after / after_total
        assert_eq!(widths[0], 150.0);
        assert!((widths[1] - 150.0 * expected_scale).abs() < 0.01);
        assert!((widths[2] - 80.0 * expected_scale).abs() < 0.01);
        assert!(widths[1] >= MIN_COLUMN_WIDTH);
        assert!(widths[2] >= MIN_COLUMN_WIDTH);
    }

    #[test]
    fn fixed_viewport_unequal_after_columns_preserve_total_budget() {
        // Regression test: a naive single-pass `w * scale` can push a
        // narrower after-column below MIN_COLUMN_WIDTH while a wider one
        // still has slack, silently exceeding the budget once the narrow
        // one gets clamped back up. Gallery-shaped scenario: drag col0
        // (Route) wide, compressing col1 (Distance=100) and col2 (Joy=60)
        // — unequal desired widths, chosen so naive scaling would push Joy
        // below the floor while Distance still has room.
        // available=190: divider_space=4, budget_for_after = 190-4-150=36,
        // which is below after_min_total(80) once split evenly by weight —
        // pick numbers where budget_for_after(90) sits between: naive
        // scale = 90/160 = 0.5625 → Joy = 60*0.5625 = 33.75 (< 40 floor)
        // while Distance = 100*0.5625 = 56.25 (fine) — exactly the
        // violating case.
        let widths = compute_rendered_widths(
            &[300.0, 100.0, 60.0], // dragged, Distance, Joy
            Some(0),
            394.0, // divider_space(4) + before(0) + dragged(300) + budget_for_after(90)
            ColumnResizeMode::FixedViewport,
        );
        assert_eq!(widths[0], 300.0);
        assert_eq!(widths[2], MIN_COLUMN_WIDTH); // Joy pinned at the floor
        assert!((widths[1] - 50.0).abs() < 0.01); // Distance absorbs the rest
        // Total must exactly equal the budget — no silent overflow.
        let total = widths[0] + widths[1] + widths[2] + 2.0 * DIVIDER_WIDTH;
        assert!((total - 394.0).abs() < 0.01);
    }

    #[test]
    fn fixed_viewport_no_active_drag_compresses_all_columns_neutrally() {
        // Regression test: right after a commit, `dragged_idx` is `None`
        // (no active drag) but only the just-resized column's desired
        // width was persisted — the others' desired widths may not all fit
        // together anymore. `None` must not arbitrarily protect column 0;
        // every column should compress together, proportionally.
        let widths = compute_rendered_widths(
            &[200.0, 250.0, 100.0, 60.0], // Name, Route(just committed wide), Distance, Joy
            None,
            536.0, // exactly what fit during the drag (Name+Route+floor(Distance)+floor(Joy)+dividers)
            ColumnResizeMode::FixedViewport,
        );
        // Name must NOT be treated as an untouchable anchor — it should
        // compress along with everything else if space is tight.
        assert!(widths[0] <= 200.0);
        let total: f64 = widths.iter().sum::<f64>() + 3.0 * DIVIDER_WIDTH;
        assert!(total <= 536.0 + 0.5);
    }

    #[test]
    fn fixed_viewport_never_touches_before_columns() {
        let widths = compute_rendered_widths(
            &[100.0, 500.0, 80.0], // dragging the middle column very wide
            Some(1),
            334.0,
            ColumnResizeMode::FixedViewport,
        );
        assert_eq!(widths[0], 100.0); // untouched, before the dragged column
    }

    #[test]
    fn fixed_viewport_caps_drag_once_after_columns_hit_floor() {
        let widths = compute_rendered_widths(
            &[100_000.0, 150.0, 80.0], // absurdly large drag target
            Some(0),
            334.0,
            ColumnResizeMode::FixedViewport,
        );
        assert_eq!(widths[1], MIN_COLUMN_WIDTH);
        assert_eq!(widths[2], MIN_COLUMN_WIDTH);
        // dragged column capped to whatever's left: 334 - divider_space(4) - 0 - 2*40 = 250
        assert_eq!(widths[0], 250.0);
    }

    #[test]
    fn fixed_viewport_shrinking_dragged_column_restores_after_columns() {
        // Growing then shrinking back should restore the after-columns —
        // confirmed by recomputing fresh each call, no incremental state.
        let compressed = compute_rendered_widths(
            &[300.0, 150.0, 80.0],
            Some(0),
            334.0,
            ColumnResizeMode::FixedViewport,
        );
        assert!(compressed[1] < 150.0);

        let restored = compute_rendered_widths(
            &[50.0, 150.0, 80.0],
            Some(0),
            334.0,
            ColumnResizeMode::FixedViewport,
        );
        assert_eq!(restored[1], 150.0);
        assert_eq!(restored[2], 80.0);
    }
}
