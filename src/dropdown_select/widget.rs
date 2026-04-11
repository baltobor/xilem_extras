//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Dropdown select widget that opens a dropdown list when clicked.

use std::any::TypeId;
use std::sync::Arc;

use xilem::masonry::accesskit::{self, Node, Role};
use tracing::{Span, trace_span};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Rect, RoundedRect, Stroke};
use xilem::masonry::peniko::Color;

use xilem::masonry::core::keyboard::{Key, NamedKey};
use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, EventCtx, LayerType, LayoutCtx, MeasureCtx, NewWidget,
    PaintCtx, PointerButtonEvent, PointerEvent, PropertiesMut, PropertiesRef, RegisterCtx,
    TextEvent, Update, UpdateCtx, Widget, WidgetId, WidgetMut, WidgetPod,
};
use xilem::masonry::kurbo::{Axis, Point, Size, Vec2};
use xilem::masonry::layout::{LayoutSize, LenReq, SizeDef};
use xilem::masonry::core::StyleProperty;
use xilem::masonry::widgets::Label;

use super::SelectDropdown;

/// Default button background color.
const BG_COLOR: Color = Color::from_rgba8(0x38, 0x36, 0x34, 0xFF);
/// Hover background color.
const BG_HOVER: Color = Color::from_rgba8(0x48, 0x46, 0x44, 0xFF);
/// Border color.
const BORDER_COLOR: Color = Color::from_rgba8(0x58, 0x56, 0x54, 0xFF);
const TEXT_SIZE: f32 = 13.0;
const PADDING_H: f64 = 10.0;
const PADDING_V: f64 = 6.0;

/// Action emitted when a dropdown selection changes.
#[derive(PartialEq, Debug, Clone)]
pub struct DropdownSelectAction {
    /// The selected option value.
    pub value: String,
    /// Index of the selected option.
    pub index: usize,
}

/// A dropdown select widget that displays the current selection and opens
/// a dropdown list when clicked.
pub struct DropdownSelect {
    /// Label showing the current selection.
    label: WidgetPod<Label>,
    /// Available options.
    options: Vec<String>,
    /// Currently selected index.
    selected_index: usize,
    /// Tracked layer id for the dropdown.
    pub(crate) dropdown_layer_id: Option<WidgetId>,
    /// Widget size.
    size: Size,
    /// Background color.
    bg_color: Color,
    /// Hover background color.
    hover_bg: Color,
    /// Border color.
    border_color: Color,
}

impl DropdownSelect {
    /// Creates a new dropdown select widget.
    ///
    /// - `options`: the list of selectable option labels.
    /// - `selected_index`: index of the initially selected option.
    pub fn new(options: Vec<String>, selected_index: usize) -> Self {
        let selected_text = options.get(selected_index)
            .cloned()
            .unwrap_or_default();

        let label = Label::new(selected_text)
            .with_style(StyleProperty::FontSize(TEXT_SIZE));

        Self {
            label: WidgetPod::new(label),
            options,
            selected_index,
            dropdown_layer_id: None,
            size: Size::ZERO,
            bg_color: BG_COLOR,
            hover_bg: BG_HOVER,
            border_color: BORDER_COLOR,
        }
    }

    fn toggle_dropdown(&mut self, ctx: &mut EventCtx<'_>) {
        if let Some(id) = self.dropdown_layer_id {
            ctx.remove_layer(id);
            self.dropdown_layer_id = None;
            return;
        }

        let mut dropdown = SelectDropdown::new(ctx.widget_id());
        for (i, label) in self.options.iter().enumerate() {
            let is_selected = i == self.selected_index;
            let item = super::SelectOptionItem::new(label.clone(), is_selected);
            dropdown = dropdown
                .with_option(NewWidget::new(item))
                .with_option_label(label.clone());
        }

        // Estimate dropdown height: ~28px per option + padding
        let estimated_dropdown_h = self.options.len() as f64 * 28.0 + 8.0;
        let widget_origin = ctx.window_origin();
        let widget_h = ctx.border_box_size().height;

        // Open upward if the widget is in the lower portion of the screen
        // (where the dropdown would likely go off-screen)
        let y_offset = if widget_origin.y > 300.0 && estimated_dropdown_h > 100.0 {
            -estimated_dropdown_h
        } else {
            widget_h
        };

        ctx.create_layer(
            LayerType::Other,
            NewWidget::new(dropdown),
            widget_origin + Vec2::new(0., y_offset),
        );
    }
}

impl DropdownSelect {
    /// Returns a mutable reference to the label widget.
    pub fn label_mut<'t>(this: &'t mut WidgetMut<'_, Self>) -> WidgetMut<'t, Label> {
        this.ctx.get_mut(&mut this.widget.label)
    }

    /// Updates the options list.
    pub fn set_options(this: &mut WidgetMut<'_, Self>, options: Vec<String>) {
        this.widget.options = options;
    }

    /// Updates the selected index and label text.
    pub fn set_selected_index(this: &mut WidgetMut<'_, Self>, index: usize) {
        this.widget.selected_index = index;
        let text: Arc<str> = this.widget.options.get(index)
            .cloned()
            .unwrap_or_default()
            .into();
        Label::set_text(&mut Self::label_mut(this), text);
    }
}

impl Widget for DropdownSelect {
    type Action = DropdownSelectAction;

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        match event {
            PointerEvent::Down(..) => {
                if self.dropdown_layer_id.is_none() {
                    ctx.capture_pointer();
                }
            }
            PointerEvent::Up(PointerButtonEvent { .. }) => {
                if ctx.is_active() && ctx.is_hovered() {
                    self.toggle_dropdown(ctx);
                }
            }
            _ => (),
        }
    }

    fn on_text_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &TextEvent,
    ) {
        match event {
            TextEvent::Keyboard(event) if event.state.is_up() => {
                if matches!(&event.key, Key::Character(c) if c == " ")
                    || event.key == Key::Named(NamedKey::Enter)
                {
                    self.toggle_dropdown(ctx);
                }
            }
            _ => (),
        }
    }

    fn on_access_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &AccessEvent,
    ) {
        if event.action == accesskit::Action::Click {
            self.toggle_dropdown(ctx);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx<'_>, _props: &mut PropertiesMut<'_>, event: &Update) {
        match event {
            Update::HoveredChanged(_)
            | Update::ActiveChanged(_)
            | Update::FocusChanged(_)
            | Update::DisabledChanged(_) => {
                ctx.request_paint_only();
            }
            _ => {}
        }
    }

    fn register_children(&mut self, ctx: &mut RegisterCtx<'_>) {
        ctx.register_child(&mut self.label);
    }

    fn property_changed(&mut self, _ctx: &mut UpdateCtx<'_>, _property_type: TypeId) {}

    fn measure(
        &mut self,
        ctx: &mut MeasureCtx<'_>,
        _props: &PropertiesRef<'_>,
        axis: Axis,
        len_req: LenReq,
        cross_length: Option<f64>,
    ) -> f64 {
        let auto_length = len_req.into();
        let context_size = LayoutSize::maybe(axis.cross(), cross_length);

        let label_len = ctx.compute_length(&mut self.label, auto_length, context_size, axis, cross_length);

        match axis {
            Axis::Horizontal => label_len + 2.0 * PADDING_H,
            Axis::Vertical => label_len + 2.0 * PADDING_V,
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, size: Size) {
        self.size = size;

        let inner_height = size.height - 2.0 * PADDING_V;
        let label_width = size.width - 2.0 * PADDING_H;
        let label_size = ctx.compute_size(
            &mut self.label,
            SizeDef::fit(Size::new(label_width.max(0.0), inner_height)),
            size.into(),
        );
        ctx.run_layout(&mut self.label, label_size);

        let label_y = ((size.height - label_size.height) * 0.5).max(0.0);
        ctx.place_child(&mut self.label, Point::new(PADDING_H, label_y));

        ctx.derive_baselines(&self.label);
    }

    fn paint(&mut self, ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, painter: &mut Painter<'_>) {
        let rect = Rect::from_origin_size(Point::ZERO, self.size);
        let rounded = RoundedRect::from_rect(rect, 4.0);

        let bg = if ctx.is_hovered() || ctx.is_active() {
            self.hover_bg
        } else {
            self.bg_color
        };

        painter.fill(rounded, bg).draw();
        painter.stroke(rounded, &Stroke::new(1.0), self.border_color).draw();
    }

    fn accessibility_role(&self) -> Role {
        Role::ComboBox
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        node: &mut Node,
    ) {
        node.add_action(accesskit::Action::Click);
        node.set_expanded(self.dropdown_layer_id.is_some());
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::from_slice(&[self.label.id()])
    }

    fn accepts_focus(&self) -> bool {
        true
    }

    fn accepts_text_input(&self) -> bool {
        false
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("DropdownSelect", id = id.trace())
    }
}
