//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! SVG icon widget and data type.

use std::any::TypeId;

use tracing::{Span, trace_span};
use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, EventCtx, LayoutCtx, MeasureCtx, PaintCtx, PointerEvent,
    PropertiesMut, PropertiesRef, RegisterCtx, TextEvent, Update, UpdateCtx, Widget, WidgetId,
};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::Axis;
use xilem::masonry::kurbo::{Affine, BezPath, Size, Stroke};
use xilem::masonry::layout::{LenReq, Length};
use xilem::masonry::peniko::Color;

/// How the icon scales relative to its container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScaleMode {
    #[default]
    AspectFill,
    AspectFit,
}

/// An SVG icon defined by a BezPath.
#[derive(Debug, Clone)]
pub struct SvgIcon {
    path: BezPath,
    viewbox_width: f64,
    viewbox_height: f64,
    pub(crate) size: f64,
    pub color: Color,
    pub stroke_width: Option<f64>,
    pub scale_mode: ScaleMode,
    pub rotation_degrees: f64,
}

impl SvgIcon {
    pub fn new(path: BezPath, viewbox_width: f64, viewbox_height: f64) -> Self {
        Self {
            path,
            viewbox_width,
            viewbox_height,
            size: 24.0,
            color: Color::WHITE,
            stroke_width: None,
            scale_mode: ScaleMode::default(),
            rotation_degrees: 0.0,
        }
    }

    pub fn from_svg(path_data: &str, viewbox_width: f64, viewbox_height: f64) -> Self {
        let path = BezPath::from_svg(path_data).unwrap_or_default();
        Self::new(path, viewbox_width, viewbox_height)
    }

    pub fn size(mut self, size: f64) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = Some(width);
        self
    }

    pub fn scale_mode(mut self, mode: ScaleMode) -> Self {
        self.scale_mode = mode;
        self
    }

    /// Rotates the icon clockwise around its own center, in degrees.
    ///
    /// Lets one path serve multiple orientations — e.g. a right-pointing
    /// chevron rotated 90° becomes a downward-pointing one — instead of
    /// needing a separate path (or a separate font glyph) per direction.
    pub fn rotation_degrees(mut self, degrees: f64) -> Self {
        self.rotation_degrees = degrees;
        self
    }

    pub fn icon_size(&self) -> f64 {
        self.size
    }

    pub fn aspect_ratio(&self) -> f64 {
        self.viewbox_width / self.viewbox_height
    }

    pub fn width(&self) -> f64 {
        match self.scale_mode {
            ScaleMode::AspectFill => self.size * self.aspect_ratio(),
            ScaleMode::AspectFit => self.size,
        }
    }

    pub fn height(&self) -> f64 {
        self.size
    }

    fn scale(&self) -> f64 {
        match self.scale_mode {
            ScaleMode::AspectFill => self.size / self.viewbox_height,
            ScaleMode::AspectFit => self.size / self.viewbox_width.max(self.viewbox_height),
        }
    }

    pub fn scaled_path(&self) -> BezPath {
        let center = (self.viewbox_width / 2.0, self.viewbox_height / 2.0);
        let rotate = Affine::rotate_about(self.rotation_degrees.to_radians(), center);
        let transform = Affine::scale(self.scale()) * rotate;
        let mut scaled = BezPath::new();
        for el in self.path.elements() {
            scaled.push(transform * *el);
        }
        scaled
    }

    pub fn scaled_stroke_width(&self) -> Option<f64> {
        self.stroke_width.map(|w| w * self.scale())
    }
}

/// Widget that renders an SVG icon.
pub struct SvgIconWidget {
    pub(crate) icon: SvgIcon,
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
    ) {
    }

    fn on_text_event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &TextEvent,
    ) {
    }

    fn on_access_event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &AccessEvent,
    ) {
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &Update,
    ) {
    }

    fn register_children(&mut self, _ctx: &mut RegisterCtx<'_>) {}

    fn property_changed(&mut self, _ctx: &mut UpdateCtx<'_>, _property_type: TypeId) {}

    fn measure(
        &mut self,
        _ctx: &mut MeasureCtx<'_>,
        _props: &PropertiesRef<'_>,
        axis: Axis,
        _len_req: LenReq,
        _cross_length: Option<Length>,
    ) -> Length {
        match axis {
            Axis::Horizontal => Length::px(self.icon.width()),
            Axis::Vertical => Length::px(self.icon.height()),
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, _size: Size) {}

    fn paint(
        &mut self,
        _ctx: &mut PaintCtx<'_>,
        _props: &PropertiesRef<'_>,
        painter: &mut Painter<'_>,
    ) {
        let path = self.icon.scaled_path();
        if let Some(stroke_width) = self.icon.scaled_stroke_width() {
            painter
                .stroke(&path, &Stroke::new(stroke_width), self.icon.color)
                .draw();
        } else {
            painter.fill(&path, self.icon.color).draw();
        }
    }

    fn accessibility_role(&self) -> Role {
        Role::Image
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        node: &mut Node,
    ) {
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
