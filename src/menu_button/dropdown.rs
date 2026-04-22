//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Floating dropdown layer for pulldown menus.

use xilem::masonry::accesskit::{Node, Role};
use tracing::{Span, trace_span};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Rect, RoundedRect, Stroke};
use xilem::masonry::peniko::Color;

use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, ComposeCtx, EventCtx, Layer, LayerType,
    LayoutCtx, MeasureCtx, NewWidget, NoAction, PaintCtx, PointerButton, PointerButtonEvent,
    PointerEvent, PropertiesMut, PropertiesRef, RegisterCtx, TextEvent, Update, UpdateCtx,
    Widget, WidgetId, WidgetPod,
};
use xilem::masonry::kurbo::{Axis, Point, Size};
use xilem::masonry::layout::{LayoutSize, LenDef, LenReq, SizeDef};
use xilem::masonry::properties::Gap;

use super::widget::{MenuButton, MenuButtonPress, MenuItemData};

/// Default dropdown background color.
const BG_COLOR: Color = Color::from_rgba8(0x38, 0x36, 0x34, 0xF8);
/// Default dropdown border color.
const BORDER_COLOR: Color = Color::from_rgba8(0x50, 0x4E, 0x4A, 0xFF);

/// A floating dropdown layer spawned by a [`MenuButton`].
///
/// Implements [`Layer`] to capture pointer events and dismiss on
/// click-outside. When an item is clicked, it submits a [`MenuButtonPress`]
/// action on behalf of the creator widget and removes itself.
pub struct MenuDropdown {
    creator: WidgetId,
    children: Vec<WidgetPod<dyn Widget>>,
    /// Submenu data for children that are submenus (indexed by child position).
    submenu_data: Vec<Option<Vec<MenuItemData>>>,
    /// Index of the currently hovered submenu item.
    pub(crate) hovered_submenu_index: Option<usize>,
    /// Widget ID of the currently open submenu layer.
    pub(crate) submenu_layer_id: Option<WidgetId>,
    /// Stored child sizes/positions for submenu placement.
    child_rects: Vec<(Point, Size)>,
    bg_color: Color,
    border_color: Color,
}

impl MenuDropdown {
    /// Creates a new empty dropdown menu.
    pub fn new(creator: WidgetId) -> Self {
        Self {
            creator,
            children: Vec::new(),
            submenu_data: Vec::new(),
            hovered_submenu_index: None,
            submenu_layer_id: None,
            child_rects: Vec::new(),
            bg_color: BG_COLOR,
            border_color: BORDER_COLOR,
        }
    }

    /// Builder-style method to add a menu item widget.
    pub fn with_item(mut self, child: NewWidget<impl Widget + ?Sized>) -> Self {
        self.children.push(child.erased().to_pod());
        self.submenu_data.push(None);
        self
    }

    /// Builder-style method to add a submenu item widget with children.
    pub fn with_submenu_item(mut self, child: NewWidget<impl Widget + ?Sized>, children: Vec<MenuItemData>) -> Self {
        self.children.push(child.erased().to_pod());
        self.submenu_data.push(Some(children));
        self
    }

    /// Sets the background color.
    pub fn with_bg_color(mut self, color: Color) -> Self {
        self.bg_color = color;
        self
    }

    /// Sets the border color.
    pub fn with_border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    /// Checks if a child at the given index is a submenu.
    fn is_submenu(&self, index: usize) -> bool {
        self.submenu_data.get(index).map(|d| d.is_some()).unwrap_or(false)
    }

    /// Builds a child submenu dropdown from item data.
    fn build_submenu(creator: WidgetId, items: &[MenuItemData]) -> SubmenuDropdown {
        let mut menu = SubmenuDropdown::new(creator);
        for item in items {
            match item {
                MenuItemData::Separator => {
                    menu = menu.with_item(NewWidget::new(super::MenuSeparator::new()));
                }
                MenuItemData::Action { label, checked } => {
                    let mut item_widget = super::PulldownMenuItem::new(label.clone());
                    if let Some(is_checked) = checked {
                        item_widget = item_widget.with_checked(Some(*is_checked));
                    }
                    menu = menu.with_item(NewWidget::new(item_widget));
                }
                MenuItemData::Submenu { .. } => {
                    // One level only - ignore nested submenus
                }
            }
        }
        menu
    }
}


impl Widget for MenuDropdown {
    type Action = NoAction;

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        match event {
            PointerEvent::Up(PointerButtonEvent {
                button: None | Some(PointerButton::Primary),
                ..
            }) => {
                let self_id = ctx.widget_id();
                let clicked_id = ctx.target();

                let index = self
                    .children
                    .iter()
                    .position(|child| child.id() == clicked_id);

                if let Some(index) = index {
                    // Don't close for submenu items - they open submenus
                    if self.is_submenu(index) {
                        return;
                    }

                    *super::widget::ACTIVE_MENU_BUTTON.lock().unwrap() = None;
                    ctx.mutate_later(self.creator, move |mut menu_btn| {
                        let mut menu_btn = menu_btn.downcast::<MenuButton>();
                        menu_btn
                            .ctx
                            .submit_action::<MenuButtonPress>(MenuButtonPress { index });
                        menu_btn.ctx.remove_layer(self_id);
                        menu_btn.widget.menu_layer_id = None;
                    });
                }
            }
            PointerEvent::Move(state) => {
                // Check which child is hovered for submenu handling
                let local_pos = ctx.local_position(state.current.position);
                let mut new_hovered: Option<usize> = None;
                let mut hovering_any_child = false;

                for (i, (origin, size)) in self.child_rects.iter().enumerate() {
                    let rect = Rect::from_origin_size(*origin, *size);
                    if rect.contains(local_pos) {
                        hovering_any_child = true;
                        if self.is_submenu(i) {
                            new_hovered = Some(i);
                        }
                        break;
                    }
                }

                if new_hovered != self.hovered_submenu_index {
                    // Only close the old submenu if we're hovering a different item
                    // (not when pointer moves into empty space — user may be
                    // moving toward the submenu popup).
                    if hovering_any_child {
                        if let Some(old_layer_id) = self.submenu_layer_id.take() {
                            ctx.remove_layer(old_layer_id);
                        }
                    }

                    // Open new submenu if hovering over a submenu item
                    if let Some(idx) = new_hovered {
                        if let Some(Some(children)) = self.submenu_data.get(idx) {
                            if let Some((origin, _size)) = self.child_rects.get(idx) {
                                let submenu = Self::build_submenu(ctx.widget_id(), children);
                                let submenu_pos = ctx.window_origin() +
                                    Point::new(ctx.border_box_size().width + 4.0, origin.y).to_vec2();

                                ctx.create_layer(
                                    LayerType::Other,
                                    NewWidget::new(submenu),
                                    submenu_pos,
                                );
                            }
                        }
                    }

                    if hovering_any_child {
                        self.hovered_submenu_index = new_hovered;
                    }
                }
            }
            _ => {}
        }
    }

    fn on_text_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &TextEvent,
    ) {
        if let TextEvent::WindowFocusChange(false) = event {
            self.dismiss(ctx);
        }
    }

    fn on_access_event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &AccessEvent,
    ) {
    }

    fn update(&mut self, ctx: &mut UpdateCtx<'_>, _props: &mut PropertiesMut<'_>, event: &Update) {
        if let Update::WidgetAdded = event {
            // Register this dropdown's layer ID with the parent MenuButton
            let id = ctx.widget_id();
            ctx.mutate_later(self.creator, move |mut menu_btn| {
                let menu_btn = menu_btn.downcast::<MenuButton>();
                menu_btn.widget.menu_layer_id = Some(id);
            });
        }
    }

    fn register_children(&mut self, ctx: &mut RegisterCtx<'_>) {
        for child in &mut self.children {
            ctx.register_child(child);
        }
    }

    fn measure(
        &mut self,
        ctx: &mut MeasureCtx<'_>,
        props: &PropertiesRef<'_>,
        axis: Axis,
        len_req: LenReq,
        cross_length: Option<f64>,
    ) -> f64 {
        let scale = 1.0;
        let gap = props.get::<Gap>(ctx.property_cache());
        let gap_length = gap.gap.dp(scale);

        let (len_req, min_result) = match len_req {
            LenReq::MinContent | LenReq::MaxContent => (len_req, 0.),
            LenReq::FitContent(space) => (LenReq::MinContent, space),
        };

        let auto_length = len_req.into();
        let context_size = LayoutSize::maybe(axis.cross(), cross_length);

        let mut length: f64 = 0.;
        for child in &mut self.children {
            let child_length =
                ctx.compute_length(child, auto_length, context_size, axis, cross_length);
            match axis {
                Axis::Horizontal => length = length.max(child_length),
                Axis::Vertical => length += child_length,
            }
        }

        if axis == Axis::Vertical && !self.children.is_empty() {
            let gap_count = (self.children.len() - 1) as f64;
            length += gap_count * gap_length;
        }

        min_result.max(length)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, props: &PropertiesRef<'_>, size: Size) {
        let scale = 1.0;
        let gap = props.get::<Gap>(ctx.property_cache());
        let gap_length = gap.gap.dp(scale);

        let width_def = LenDef::FitContent(size.width);
        let height_def = LenDef::FitContent(size.height);
        let auto_size = SizeDef::new(width_def, height_def);
        let context_size = size.into();

        // Clear and rebuild child rects for submenu positioning
        self.child_rects.clear();

        let mut y_offset = 0.0;
        for child in &mut self.children {
            let child_size = ctx.compute_size(child, auto_size, context_size);
            ctx.run_layout(child, child_size);
            let origin = Point::new(0.0, y_offset);
            ctx.place_child(child, origin);
            self.child_rects.push((origin, child_size));
            y_offset += child_size.height + gap_length;
        }

        if !self.children.is_empty() {
            let first_child = self.children.first().unwrap();
            let (first_baseline, _) = ctx.child_aligned_baselines(first_child);
            let first_child_origin = ctx.child_origin(first_child);
            let first_baseline = first_child_origin.y + first_baseline;

            let last_child = self.children.last().unwrap();
            let (_, last_baseline) = ctx.child_aligned_baselines(last_child);
            let last_child_origin = ctx.child_origin(last_child);
            let last_baseline = last_child_origin.y + last_baseline;

            ctx.set_baselines(first_baseline, last_baseline);
        } else {
            ctx.clear_baselines();
        }
    }

    fn compose(&mut self, _ctx: &mut ComposeCtx<'_>) {}

    fn paint(&mut self, ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, painter: &mut Painter<'_>) {
        let size = ctx.border_box_size();
        let padding = 6.0;
        let rect = Rect::new(
            -padding,
            -padding / 2.0,
            size.width + padding,
            size.height + padding / 2.0,
        );
        let rounded = RoundedRect::from_rect(rect, 4.0);

        painter.fill(rounded, self.bg_color).draw();
        painter.stroke(rounded, &Stroke::new(1.0), self.border_color).draw();
    }

    fn accessibility_role(&self) -> Role {
        Role::Menu
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        _node: &mut Node,
    ) {
    }

    fn children_ids(&self) -> ChildrenIds {
        self.children.iter().map(|child| child.id()).collect()
    }

    fn as_layer(&mut self) -> Option<&mut dyn Layer> {
        Some(self)
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("MenuDropdown", id = id.trace())
    }
}

impl Layer for MenuDropdown {
    fn capture_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        let dismiss = match event {
            PointerEvent::Down(PointerButtonEvent { state, .. }) => {
                let local_pos = ctx.local_position(state.position);
                !ctx.border_box_size().to_rect().contains(local_pos)
            }
            PointerEvent::Cancel(..) => true,
            _ => false,
        };

        if dismiss {
            self.dismiss(ctx);
        }
    }
}

impl MenuDropdown {
    /// Dismiss this dropdown and any open submenu, and clear global tracking.
    fn dismiss(&mut self, ctx: &mut EventCtx<'_>) {
        if let Some(submenu_id) = self.submenu_layer_id.take() {
            ctx.remove_layer(submenu_id);
        }
        ctx.remove_layer(ctx.widget_id());
        *super::widget::ACTIVE_MENU_BUTTON.lock().unwrap() = None;
        ctx.mutate_later(self.creator, move |mut menu_btn| {
            let menu_btn = menu_btn.downcast::<MenuButton>();
            menu_btn.widget.menu_layer_id = None;
        });
    }
}

// ============================================================================
// SubmenuDropdown - a simple dropdown for submenu content (one level only)
// ============================================================================

/// A child dropdown shown when hovering over a submenu item.
/// Simpler than MenuDropdown - no recursive submenu support.
pub struct SubmenuDropdown {
    creator: WidgetId,
    children: Vec<WidgetPod<dyn Widget>>,
    bg_color: Color,
    border_color: Color,
}

impl SubmenuDropdown {
    pub fn new(creator: WidgetId) -> Self {
        Self {
            creator,
            children: Vec::new(),
            bg_color: BG_COLOR,
            border_color: BORDER_COLOR,
        }
    }

    pub fn with_item(mut self, child: NewWidget<impl Widget + ?Sized>) -> Self {
        self.children.push(child.erased().to_pod());
        self
    }
}


impl Widget for SubmenuDropdown {
    type Action = NoAction;

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        if let PointerEvent::Up(PointerButtonEvent {
            button: None | Some(PointerButton::Primary),
            ..
        }) = event
        {
            let self_id = ctx.widget_id();
            let clicked_id = ctx.target();

            let index = self
                .children
                .iter()
                .position(|child| child.id() == clicked_id);

            if let Some(_index) = index {
                // TODO: Submit action through parent MenuDropdown
                // For now, just close the submenu
                ctx.remove_layer(self_id);
            }
        }
    }

    fn on_text_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &TextEvent,
    ) {
        if let TextEvent::WindowFocusChange(false) = event {
            ctx.remove_layer(ctx.widget_id());
        }
    }

    fn on_access_event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &AccessEvent,
    ) {
    }

    fn update(&mut self, ctx: &mut UpdateCtx<'_>, _props: &mut PropertiesMut<'_>, event: &Update) {
        if let Update::WidgetAdded = event {
            // Register this submenu's ID with the parent MenuDropdown so it can close us
            let id = ctx.widget_id();
            ctx.mutate_later(self.creator, move |mut parent| {
                let parent = parent.downcast::<MenuDropdown>();
                parent.widget.submenu_layer_id = Some(id);
            });
        }
    }

    fn register_children(&mut self, ctx: &mut RegisterCtx<'_>) {
        for child in &mut self.children {
            ctx.register_child(child);
        }
    }

    fn measure(
        &mut self,
        ctx: &mut MeasureCtx<'_>,
        props: &PropertiesRef<'_>,
        axis: Axis,
        len_req: LenReq,
        cross_length: Option<f64>,
    ) -> f64 {
        let scale = 1.0;
        let gap = props.get::<Gap>(ctx.property_cache());
        let gap_length = gap.gap.dp(scale);

        let (len_req, min_result) = match len_req {
            LenReq::MinContent | LenReq::MaxContent => (len_req, 0.),
            LenReq::FitContent(space) => (LenReq::MinContent, space),
        };

        let auto_length = len_req.into();
        let context_size = LayoutSize::maybe(axis.cross(), cross_length);

        let mut length: f64 = 0.;
        for child in &mut self.children {
            let child_length =
                ctx.compute_length(child, auto_length, context_size, axis, cross_length);
            match axis {
                Axis::Horizontal => length = length.max(child_length),
                Axis::Vertical => length += child_length,
            }
        }

        if axis == Axis::Vertical && !self.children.is_empty() {
            let gap_count = (self.children.len() - 1) as f64;
            length += gap_count * gap_length;
        }

        min_result.max(length)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, props: &PropertiesRef<'_>, size: Size) {
        let scale = 1.0;
        let gap = props.get::<Gap>(ctx.property_cache());
        let gap_length = gap.gap.dp(scale);

        let width_def = LenDef::FitContent(size.width);
        let height_def = LenDef::FitContent(size.height);
        let auto_size = SizeDef::new(width_def, height_def);
        let context_size = size.into();

        let mut y_offset = 0.0;
        for child in &mut self.children {
            let child_size = ctx.compute_size(child, auto_size, context_size);
            ctx.run_layout(child, child_size);
            ctx.place_child(child, Point::new(0.0, y_offset));
            y_offset += child_size.height + gap_length;
        }

        if !self.children.is_empty() {
            let first_child = self.children.first().unwrap();
            let (first_baseline, _) = ctx.child_aligned_baselines(first_child);
            let first_child_origin = ctx.child_origin(first_child);
            let first_baseline = first_child_origin.y + first_baseline;

            let last_child = self.children.last().unwrap();
            let (_, last_baseline) = ctx.child_aligned_baselines(last_child);
            let last_child_origin = ctx.child_origin(last_child);
            let last_baseline = last_child_origin.y + last_baseline;

            ctx.set_baselines(first_baseline, last_baseline);
        } else {
            ctx.clear_baselines();
        }
    }

    fn compose(&mut self, _ctx: &mut ComposeCtx<'_>) {}

    fn paint(&mut self, ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, painter: &mut Painter<'_>) {
        let size = ctx.border_box_size();
        let padding = 6.0;
        let rect = Rect::new(
            -padding,
            -padding / 2.0,
            size.width + padding,
            size.height + padding / 2.0,
        );
        let rounded = RoundedRect::from_rect(rect, 4.0);

        painter.fill(rounded, self.bg_color).draw();
        painter.stroke(rounded, &Stroke::new(1.0), self.border_color).draw();
    }

    fn accessibility_role(&self) -> Role {
        Role::Menu
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        _node: &mut Node,
    ) {
    }

    fn children_ids(&self) -> ChildrenIds {
        self.children.iter().map(|child| child.id()).collect()
    }

    fn as_layer(&mut self) -> Option<&mut dyn Layer> {
        Some(self)
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("SubmenuDropdown", id = id.trace())
    }
}

impl Layer for SubmenuDropdown {
    fn capture_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        match event {
            PointerEvent::Cancel(..) => {
                ctx.remove_layer(ctx.widget_id());
            }
            PointerEvent::Down(PointerButtonEvent { state, .. }) => {
                // Dismiss if clicking outside the submenu area
                let local_pos = ctx.local_position(state.position);
                if !ctx.border_box_size().to_rect().contains(local_pos) {
                    ctx.remove_layer(ctx.widget_id());
                    // Clear submenu tracking in parent
                    ctx.mutate_later(self.creator, move |mut parent| {
                        let parent = parent.downcast::<MenuDropdown>();
                        parent.widget.submenu_layer_id = None;
                        parent.widget.hovered_submenu_index = None;
                    });
                }
            }
            _ => {}
        }
    }
}
