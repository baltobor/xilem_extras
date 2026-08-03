// Copyright 2026 the Xilem Authors
// SPDX-License-Identifier: Apache-2.0

//! Row cell placement widget.

use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::core::{
    AccessCtx, ChildrenIds, LayoutCtx, MeasureCtx, NewWidget, PaintCtx, PropertiesRef, RegisterCtx,
    Widget, WidgetMut, WidgetPod,
};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Axis, Size};
use xilem::masonry::layout::{LayoutSize, LenReq, Length};

use crate::masonry::table::column_layout::{self, ColumnBox};

/// Places a fixed set of cell widgets at exact column positions (widths +
/// x_offsets) computed once upstream by `ResizableHeader` — the single
/// source of truth for column layout (see `ColumnLayoutAction`) — and
/// pushed down via `set_columns`. Placement goes through
/// `column_layout::place_children`, the *identical* mechanism
/// `ResizableHeader` uses for its own header cells, so header and row
/// content can never independently drift apart the way they did when row
/// content was built via a generic sequential/stacking xilem layout view.
///
/// No dragging, hit-testing, or painting of its own — children paint
/// themselves at their placed positions.
pub struct RowCells {
    children: Vec<WidgetPod<dyn Widget>>,
    columns: Vec<ColumnBox>,
}

impl RowCells {
    pub fn new(children: Vec<NewWidget<dyn Widget>>, columns: Vec<ColumnBox>) -> Self {
        Self {
            children: children.into_iter().map(|c| c.to_pod()).collect(),
            columns,
        }
    }

    /// Updates the resolved column layout — index `i` places child `i`.
    pub fn set_columns(this: &mut WidgetMut<'_, Self>, columns: Vec<ColumnBox>) {
        if this.widget.columns != columns {
            this.widget.columns = columns;
            this.ctx.request_layout();
        }
    }

    pub fn child_mut<'t>(
        this: &'t mut WidgetMut<'_, Self>,
        index: usize,
    ) -> Option<WidgetMut<'t, dyn Widget>> {
        this.widget
            .children
            .get_mut(index)
            .map(|child| this.ctx.get_mut(child))
    }
}

impl Widget for RowCells {
    type Action = ();

    fn register_children(&mut self, ctx: &mut RegisterCtx<'_>) {
        for child in &mut self.children {
            ctx.register_child(child);
        }
    }

    fn measure(
        &mut self,
        ctx: &mut MeasureCtx<'_>,
        _props: &PropertiesRef<'_>,
        axis: Axis,
        len_req: LenReq,
        _cross_length: Option<Length>,
    ) -> Length {
        match axis {
            Axis::Horizontal => Length::px(self.columns.iter().map(|c| c.width).sum()),
            Axis::Vertical => {
                let mut max_height = Length::ZERO;
                for (i, child) in self.children.iter_mut().enumerate() {
                    let col_width = self.columns.get(i).map(|c| Length::px(c.width));
                    let height = ctx.compute_length(
                        child,
                        len_req.into(),
                        LayoutSize::maybe(Axis::Horizontal, col_width),
                        axis,
                        col_width,
                    );
                    if height.get() > max_height.get() {
                        max_height = height;
                    }
                }
                max_height
            }
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, size: Size) {
        column_layout::place_children(ctx, &mut self.children, &self.columns, size.height);
    }

    fn paint(&mut self, _ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, _painter: &mut Painter<'_>) {
        // Nothing of its own to paint — children paint themselves at their
        // placed positions.
    }

    fn accessibility_role(&self) -> Role {
        Role::GenericContainer
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        _node: &mut Node,
    ) {
    }

    fn children_ids(&self) -> ChildrenIds {
        let ids: Vec<_> = self.children.iter().map(|c| c.id()).collect();
        ChildrenIds::from_slice(&ids)
    }
}
