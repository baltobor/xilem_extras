//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Xilem view for the SVG icon widget.

use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewMarker};
use xilem::{Pod, ViewCtx};

use crate::masonry::components::svg_icon::{SvgIcon, SvgIconWidget};

/// Xilem view for an SVG icon.
pub struct SvgIconView {
    icon: SvgIcon,
}

/// Creates an SVG icon view.
pub fn svg_icon(icon: SvgIcon) -> SvgIconView {
    SvgIconView { icon }
}

impl ViewMarker for SvgIconView {}

impl<State: 'static, Action: 'static> View<State, Action, ViewCtx> for SvgIconView {
    type Element = Pod<SvgIconWidget>;
    type ViewState = ();

    fn build(&self, ctx: &mut ViewCtx, _app_state: &mut State) -> (Self::Element, Self::ViewState) {
        let widget = SvgIconWidget::new(self.icon.clone());
        let pod = ctx.create_pod(widget);
        (pod, ())
    }

    fn rebuild(
        &self,
        prev: &Self,
        _view_state: &mut Self::ViewState,
        _ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        _app_state: &mut State,
    ) {
        let dimensions_changed = self.icon.size != prev.icon.size
            || self.icon.scale_mode != prev.icon.scale_mode
            || self.icon.color != prev.icon.color
            || self.icon.stroke_width != prev.icon.stroke_width;

        if dimensions_changed {
            element.widget.set_icon(self.icon.clone());
            element.ctx.request_layout();
        }
    }

    fn teardown(
        &self,
        _view_state: &mut Self::ViewState,
        _ctx: &mut ViewCtx,
        _element: Mut<'_, Self::Element>,
    ) {
    }

    fn message(
        &self,
        _view_state: &mut Self::ViewState,
        _message: &mut MessageCtx,
        _element: Mut<'_, Self::Element>,
        _app_state: &mut State,
    ) -> MessageResult<Action> {
        MessageResult::Nop
    }
}
