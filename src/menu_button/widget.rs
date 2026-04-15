//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Menu button widget that opens a dropdown menu layer when clicked.

use std::any::TypeId;

use xilem::masonry::accesskit::{self, Node, Role};
use tracing::{Span, trace_span};
use xilem::masonry::imaging::Painter;

use xilem::masonry::core::keyboard::{Key, NamedKey};
use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, EventCtx, LayerType, LayoutCtx, MeasureCtx, NewWidget,
    PaintCtx, PointerButtonEvent, PointerEvent, PropertiesMut, PropertiesRef, RegisterCtx,
    TextEvent, Update, UpdateCtx, Widget, WidgetId, WidgetMut, WidgetPod,
};
use xilem::masonry::kurbo::{Axis, Size, Vec2};
use xilem::masonry::layout::{LayoutSize, LenReq, SizeDef};

use super::MenuDropdown;

/// Action emitted when a menu item inside a [`MenuButton`]'s dropdown is clicked.
#[derive(PartialEq, Debug)]
pub struct MenuButtonPress {
    /// Index of the selected menu item.
    pub index: usize,
}

/// Data for a menu item, including support for submenus.
#[derive(Clone, Debug)]
pub enum MenuItemData {
    /// A clickable action item with a label.
    Action { label: String, checked: Option<bool> },
    /// A visual separator.
    Separator,
    /// A submenu with nested items.
    Submenu { label: String, children: Vec<MenuItemData> },
}

impl MenuItemData {
    /// Creates an action item.
    pub fn action(label: impl Into<String>) -> Self {
        Self::Action { label: label.into(), checked: None }
    }

    /// Creates an action item with checked state.
    pub fn action_checked(label: impl Into<String>, checked: bool) -> Self {
        Self::Action { label: label.into(), checked: Some(checked) }
    }

    /// Creates a separator.
    pub fn separator() -> Self {
        Self::Separator
    }

    /// Creates a submenu.
    pub fn submenu(label: impl Into<String>, children: Vec<MenuItemData>) -> Self {
        Self::Submenu { label: label.into(), children }
    }

    /// Returns the display label, or None for separators.
    pub fn label(&self) -> Option<&str> {
        match self {
            Self::Action { label, .. } => Some(label),
            Self::Submenu { label, .. } => Some(label),
            Self::Separator => None,
        }
    }

    /// Returns true if this is a submenu.
    pub fn is_submenu(&self) -> bool {
        matches!(self, Self::Submenu { .. })
    }
}

/// A button widget that opens a dropdown menu layer when clicked.
///
/// This follows the same pattern as masonry's `Selector` widget: on click it
/// spawns a [`MenuDropdown`] layer positioned below itself. The dropdown
/// dismisses on click-outside (via the [`Layer`] trait) or when an item is
/// selected.
pub struct MenuButton {
    /// The label shown on the button (e.g. "File", "View").
    child: WidgetPod<dyn Widget>,
    /// Menu item data including submenus.
    items: Vec<MenuItemData>,
    /// Tracked layer id so we can toggle/remove it.
    pub(crate) menu_layer_id: Option<WidgetId>,
}

impl MenuButton {
    /// Creates a new menu button.
    ///
    /// - `child`: the always-visible label widget (e.g. a `Label`).
    /// - `items`: the list of dropdown menu item labels.
    pub fn new(child: NewWidget<impl Widget + ?Sized>, items: Vec<String>) -> Self {
        // Convert legacy string-based items to MenuItemData
        let items = items.into_iter().map(|label| {
            if label == "---" {
                MenuItemData::Separator
            } else {
                MenuItemData::Action { label, checked: None }
            }
        }).collect();

        Self {
            child: child.erased().to_pod(),
            items,
            menu_layer_id: None,
        }
    }

    /// Creates a new menu button with full menu item data (including submenus).
    pub fn new_with_data(child: NewWidget<impl Widget + ?Sized>, items: Vec<MenuItemData>) -> Self {
        Self {
            child: child.erased().to_pod(),
            items,
            menu_layer_id: None,
        }
    }

    fn toggle_layer(&mut self, ctx: &mut EventCtx<'_>) {
        if let Some(id) = self.menu_layer_id {
            ctx.remove_layer(id);
            self.menu_layer_id = None;
            return;
        }

        let menu = Self::build_dropdown(ctx.widget_id(), &self.items);

        ctx.create_layer(
            LayerType::Other,
            NewWidget::new(menu),
            ctx.window_origin() + Vec2::new(0., ctx.border_box_size().height),
        );
    }

    /// Builds a MenuDropdown from item data.
    fn build_dropdown(creator: WidgetId, items: &[MenuItemData]) -> MenuDropdown {
        let mut menu = MenuDropdown::new(creator);
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
                MenuItemData::Submenu { label, children } => {
                    let submenu_widget = super::PulldownSubmenuItem::new(label.clone());
                    menu = menu.with_submenu_item(
                        NewWidget::new(submenu_widget),
                        children.clone(),
                    );
                }
            }
        }
        menu
    }
}

impl MenuButton {
    /// Returns a mutable reference to the child label widget.
    pub fn child_mut<'t>(this: &'t mut WidgetMut<'_, Self>) -> WidgetMut<'t, dyn Widget> {
        this.ctx.get_mut(&mut this.widget.child)
    }

    /// Replaces the dropdown item labels (legacy API).
    pub fn set_items(this: &mut WidgetMut<'_, Self>, items: Vec<String>) {
        this.widget.items = items.into_iter().map(|label| {
            if label == "---" {
                MenuItemData::Separator
            } else {
                MenuItemData::Action { label, checked: None }
            }
        }).collect();
    }

    /// Replaces the dropdown items with full menu data.
    pub fn set_items_data(this: &mut WidgetMut<'_, Self>, items: Vec<MenuItemData>) {
        this.widget.items = items;
    }
}

impl Widget for MenuButton {
    type Action = MenuButtonPress;

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        match event {
            PointerEvent::Down(..) => {
                if self.menu_layer_id.is_none() {
                    ctx.capture_pointer();
                }
            }
            PointerEvent::Up(PointerButtonEvent { .. }) => {
                if ctx.is_active() && ctx.is_hovered() {
                    self.toggle_layer(ctx);
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
                    self.toggle_layer(ctx);
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
            self.toggle_layer(ctx);
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
        ctx.register_child(&mut self.child);
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

        ctx.compute_length(&mut self.child, auto_length, context_size, axis, cross_length)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, size: Size) {
        let child_size = ctx.compute_size(&mut self.child, SizeDef::fit(size), size.into());
        ctx.run_layout(&mut self.child, child_size);

        let child_origin = ((size - child_size).to_vec2() * 0.5).to_point();
        ctx.place_child(&mut self.child, child_origin);
        ctx.derive_baselines(&self.child);
    }

    fn paint(&mut self, _ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, _painter: &mut Painter<'_>) {}

    fn accessibility_role(&self) -> Role {
        Role::Button
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        node: &mut Node,
    ) {
        node.add_action(accesskit::Action::Click);
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::from_slice(&[self.child.id()])
    }

    fn accepts_focus(&self) -> bool {
        true
    }

    fn accepts_text_input(&self) -> bool {
        false
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("MenuButton", id = id.trace())
    }
}
