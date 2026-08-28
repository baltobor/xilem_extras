// Copyright 2026 the Xilem Authors
// SPDX-License-Identifier: Apache-2.0

//! Xilem view for placing row cells at exact column positions.
//!
//! Rows must be placed at the *absolute* `column_x_offsets` a `table`
//! row builder receives, matching the header's own placement exactly — a
//! plain sequential layout (e.g. `flex_row`) cannot make one column's
//! growth shift a column before it, which RTL requires. `row_cells` places
//! each cell via `masonry::table::row_cells::RowCells`, which uses the
//! *identical* placement code `ResizableHeader` uses for its own header
//! cells (see `column_layout::place_children`), so header and row content
//! can never independently drift apart.

use std::marker::PhantomData;
use std::sync::Arc;

use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewId, ViewMarker, ViewPathTracker};
use xilem::{Pod, ViewCtx, WidgetView};

use crate::masonry::table::column_layout::ColumnBox;
use crate::masonry::table::row_cells::RowCells;

fn build_columns(widths: &[f64], x_offsets: &[f64]) -> Vec<ColumnBox> {
    widths
        .iter()
        .zip(x_offsets.iter())
        .enumerate()
        .map(|(i, (&width, &x_offset))| ColumnBox {
            key: Arc::from(i.to_string()),
            width,
            x_offset,
        })
        .collect()
}

/// Places `cells` at the given `widths`/`x_offsets` — see the module doc
/// comment. `widths[i]`/`x_offsets[i]` place `cells[i]`.
pub fn row_cells<State: 'static, Action: 'static, V>(
    cells: Vec<V>,
    widths: &[f64],
    x_offsets: &[f64],
) -> RowCellsView<V, State, Action>
where
    V: WidgetView<State, Action>,
{
    RowCellsView {
        cells,
        widths: widths.to_vec(),
        x_offsets: x_offsets.to_vec(),
        _phantom: PhantomData,
    }
}

/// The view type for [`row_cells`].
pub struct RowCellsView<V, State, Action> {
    cells: Vec<V>,
    widths: Vec<f64>,
    x_offsets: Vec<f64>,
    _phantom: PhantomData<fn(&mut State) -> Action>,
}

impl<V, State, Action> ViewMarker for RowCellsView<V, State, Action> {}

impl<V, State, Action> View<State, Action, ViewCtx> for RowCellsView<V, State, Action>
where
    V: WidgetView<State, Action>,
    State: 'static,
    Action: 'static,
{
    type Element = Pod<RowCells>;
    type ViewState = Vec<V::ViewState>;

    fn build(&self, ctx: &mut ViewCtx, app_state: &mut State) -> (Self::Element, Self::ViewState) {
        let mut child_pods = Vec::new();
        let mut child_states = Vec::new();

        for (i, cell) in self.cells.iter().enumerate() {
            let (pod, state) = ctx.with_id(ViewId::new(i as u64), |ctx| cell.build(ctx, app_state));
            child_pods.push(pod.new_widget.erased());
            child_states.push(state);
        }

        let columns = build_columns(&self.widths, &self.x_offsets);
        let widget = RowCells::new(child_pods, columns);
        let pod = ctx.create_pod(widget);

        (pod, child_states)
    }

    fn rebuild(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) {
        if self.widths != prev.widths || self.x_offsets != prev.x_offsets {
            RowCells::set_columns(&mut element, build_columns(&self.widths, &self.x_offsets));
        }

        for (i, (cell, prev_cell)) in self.cells.iter().zip(prev.cells.iter()).enumerate() {
            if let Some(state) = view_state.get_mut(i) {
                if let Some(mut child_element) = RowCells::child_mut(&mut element, i) {
                    ctx.with_id(ViewId::new(i as u64), |ctx| {
                        cell.rebuild(prev_cell, state, ctx, child_element.downcast(), app_state);
                    });
                }
            }
        }
    }

    fn teardown(
        &self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
    ) {
        for (i, cell) in self.cells.iter().enumerate() {
            if let Some(state) = view_state.get_mut(i) {
                if let Some(mut child_element) = RowCells::child_mut(&mut element, i) {
                    ctx.with_id(ViewId::new(i as u64), |ctx| {
                        cell.teardown(state, ctx, child_element.downcast());
                    });
                }
            }
        }
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        message: &mut MessageCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) -> MessageResult<Action> {
        match message.take_first() {
            Some(id) => {
                let idx = id.routing_id() as usize;
                if let (Some(cell), Some(state)) = (self.cells.get(idx), view_state.get_mut(idx)) {
                    if let Some(mut child_element) = RowCells::child_mut(&mut element, idx) {
                        return cell.message(state, message, child_element.downcast(), app_state);
                    }
                }
                MessageResult::Stale
            }
            None => MessageResult::Stale,
        }
    }
}
