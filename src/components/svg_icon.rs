//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! SVG-based icons rendered using Vello paths.

use std::any::TypeId;

/// How the icon scales relative to its container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScaleMode {
    /// Scale to fill the height, width calculated from aspect ratio (default).
    /// The icon fills the full height and extends horizontally as needed.
    #[default]
    AspectFill,
    /// Scale to fit within a square of the given size.
    /// The icon fits entirely within the size, with empty space if not square.
    AspectFit,
}

use xilem::core::{MessageCtx, Mut, View, ViewMarker, MessageResult};
use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, EventCtx, LayoutCtx, MeasureCtx,
    PaintCtx, PointerEvent, PropertiesMut, PropertiesRef, RegisterCtx, TextEvent,
    Update, UpdateCtx, Widget, WidgetId,
};
use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::imaging::Painter;
use xilem::masonry::layout::LenReq;
use xilem::masonry::vello::kurbo::{Affine, BezPath, Size, Stroke};
use xilem::masonry::vello::peniko::Color;
use xilem::masonry::vello::kurbo::Axis;
use xilem::{Pod, ViewCtx};
use tracing::{Span, trace_span};

/// An SVG icon defined by a BezPath.
///
/// The icon is scaled according to the scale mode while maintaining aspect ratio.
#[derive(Debug, Clone)]
pub struct SvgIcon {
    path: BezPath,
    viewbox_width: f64,
    viewbox_height: f64,
    size: f64,
    color: Color,
    stroke_width: Option<f64>,
    scale_mode: ScaleMode,
}

impl SvgIcon {
    /// Creates a new SVG icon from a path and viewbox dimensions.
    pub fn new(path: BezPath, viewbox_width: f64, viewbox_height: f64) -> Self {
        Self {
            path,
            viewbox_width,
            viewbox_height,
            size: 24.0,
            color: Color::WHITE,
            stroke_width: None,
            scale_mode: ScaleMode::default(),
        }
    }

    /// Creates a new SVG icon from an SVG path string.
    pub fn from_svg(path_data: &str, viewbox_width: f64, viewbox_height: f64) -> Self {
        let path = BezPath::from_svg(path_data).unwrap_or_default();
        Self::new(path, viewbox_width, viewbox_height)
    }

    /// Sets the icon size.
    /// - AspectFill: size is the height, width calculated from aspect ratio
    /// - AspectFit: size is both width and height (square container)
    pub fn size(mut self, size: f64) -> Self {
        self.size = size;
        self
    }

    /// Sets the icon color.
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets the stroke width (for outline icons). If set, the path is stroked instead of filled.
    pub fn stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = Some(width);
        self
    }

    /// Sets the scale mode (AspectFill or AspectFit).
    pub fn scale_mode(mut self, mode: ScaleMode) -> Self {
        self.scale_mode = mode;
        self
    }

    /// Returns the current size.
    pub fn icon_size(&self) -> f64 {
        self.size
    }

    /// Returns the aspect ratio (width / height).
    pub fn aspect_ratio(&self) -> f64 {
        self.viewbox_width / self.viewbox_height
    }

    /// Returns the rendered width.
    pub fn width(&self) -> f64 {
        match self.scale_mode {
            ScaleMode::AspectFill => self.size * self.aspect_ratio(),
            ScaleMode::AspectFit => self.size,
        }
    }

    /// Returns the rendered height.
    pub fn height(&self) -> f64 {
        self.size
    }

    /// Returns the scale factor for rendering.
    fn scale(&self) -> f64 {
        match self.scale_mode {
            ScaleMode::AspectFill => self.size / self.viewbox_height,
            ScaleMode::AspectFit => self.size / self.viewbox_width.max(self.viewbox_height),
        }
    }

    /// Returns the scaled path for rendering.
    pub fn scaled_path(&self) -> BezPath {
        let transform = Affine::scale(self.scale());

        let mut scaled = BezPath::new();
        for el in self.path.elements() {
            scaled.push(transform * *el);
        }
        scaled
    }

    /// Returns the scaled stroke width, if set.
    pub fn scaled_stroke_width(&self) -> Option<f64> {
        self.stroke_width.map(|w| w * self.scale())
    }
}

/// Widget that renders an SVG icon.
pub struct SvgIconWidget {
    icon: SvgIcon,
}

impl SvgIconWidget {
    pub fn new(icon: SvgIcon) -> Self {
        Self { icon }
    }

    pub fn set_icon(&mut self, icon: SvgIcon) {
        self.icon = icon;
    }

    pub fn set_color(&mut self, color: Color) {
        self.icon.color = color;
    }
}

impl Widget for SvgIconWidget {
    type Action = ();

    fn on_pointer_event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &PointerEvent,
    ) {}

    fn on_text_event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &TextEvent,
    ) {}

    fn on_access_event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &AccessEvent,
    ) {}

    fn update(&mut self, _ctx: &mut UpdateCtx<'_>, _props: &mut PropertiesMut<'_>, _event: &Update) {}

    fn register_children(&mut self, _ctx: &mut RegisterCtx<'_>) {}

    fn property_changed(&mut self, _ctx: &mut UpdateCtx<'_>, _property_type: TypeId) {}

    fn measure(
        &mut self,
        _ctx: &mut MeasureCtx<'_>,
        _props: &PropertiesRef<'_>,
        axis: Axis,
        _len_req: LenReq,
        _cross_length: Option<f64>,
    ) -> f64 {
        match axis {
            Axis::Horizontal => self.icon.width(),
            Axis::Vertical => self.icon.height(),
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, _size: Size) {}

    fn paint(&mut self, _ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, painter: &mut Painter<'_>) {
        let path = self.icon.scaled_path();
        if let Some(stroke_width) = self.icon.scaled_stroke_width() {
            painter.stroke(&path, &Stroke::new(stroke_width), self.icon.color).draw();
        } else {
            painter.fill(&path, self.icon.color).draw();
        }
    }

    fn accessibility_role(&self) -> Role {
        Role::Image
    }

    fn accessibility(&mut self, _ctx: &mut AccessCtx<'_>, _props: &PropertiesRef<'_>, node: &mut Node) {
        node.set_label("icon");
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::from_slice(&[])
    }

    fn accepts_focus(&self) -> bool {
        false
    }

    fn accepts_text_input(&self) -> bool {
        false
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("SvgIcon", id = id.trace())
    }
}

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
        // Check if dimensions changed (requires layout)
        let dimensions_changed = self.icon.size != prev.icon.size
            || self.icon.scale_mode != prev.icon.scale_mode
            || self.icon.viewbox_width != prev.icon.viewbox_width
            || self.icon.viewbox_height != prev.icon.viewbox_height
            || self.icon.path != prev.icon.path;

        // Check if only appearance changed (paint only)
        let appearance_changed = self.icon.color != prev.icon.color
            || self.icon.stroke_width != prev.icon.stroke_width;

        if dimensions_changed {
            element.widget.set_icon(self.icon.clone());
            element.ctx.request_layout();
        } else if appearance_changed {
            element.widget.set_icon(self.icon.clone());
            element.ctx.request_paint_only();
        }
    }

    fn teardown(
        &self,
        _view_state: &mut Self::ViewState,
        _ctx: &mut ViewCtx,
        _element: Mut<'_, Self::Element>,
    ) {}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_icon_default_size() {
        let icon = SvgIcon::new(BezPath::new(), 100.0, 100.0);
        assert_eq!(icon.icon_size(), 24.0);
    }

    #[test]
    fn svg_icon_custom_size() {
        let icon = SvgIcon::new(BezPath::new(), 100.0, 100.0).size(48.0);
        assert_eq!(icon.icon_size(), 48.0);
    }

    #[test]
    fn svg_icon_color() {
        let red = Color::from_rgb8(255, 0, 0);
        let icon = SvgIcon::new(BezPath::new(), 100.0, 100.0).color(red);
        assert_eq!(icon.color, red);
    }

    #[test]
    fn svg_icon_from_svg() {
        let icon = SvgIcon::from_svg("M 0 0 L 10 10", 100.0, 100.0);
        assert!(!icon.path.elements().is_empty());
    }
}
