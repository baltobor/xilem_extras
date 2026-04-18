//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Automatic rendering of `MenuBarBuilder` as xilem pulldown menu widgets.
//!
//! On Linux (or any platform without native muda support), this module provides
//! [`pulldown_menu_bar`] which converts a [`MenuBarBuilder`] into a horizontal
//! row of pulldown [`menu_button`](crate::menu_button) views — automatically.
//!
//! This means you can define your menu once and have it work on all platforms:
//! - macOS/Windows: `build_muda_menu()` for native menus
//! - Linux: `pulldown_menu_bar()` for xilem widget menus
//!
//! # Example
//!
//! ```ignore
//! use xilem_extras::app_menu::{pulldown_menu_bar, MenuBarBuilder, PulldownMenuBarStyle};
//!
//! fn define_menus() -> MenuBarBuilder<AppModel, ()> {
//!     MenuBarBuilder::new()
//!         .menu("File", |m| m
//!             .item("New", |s: &mut AppModel| s.new_file())
//!             .separator()
//!             .item("Quit", |s| s.quit())
//!         )
//!         .menu("Edit", |m| m
//!             .item("Undo", |s| s.undo())
//!             .item("Redo", |s| s.redo())
//!         )
//! }
//!
//! fn linux_menu_bar(model: &mut AppModel) -> impl WidgetView<AppModel> {
//!     pulldown_menu_bar(
//!         define_menus(),
//!         handle_action,      // fn(&mut AppModel, MyAction)
//!         PulldownMenuBarStyle::default(),
//!     )
//! }
//! ```

use std::sync::Arc;

use xilem::masonry::peniko::Color;
use xilem::masonry::layout::Length;
use xilem::view::{flex_row, label};
use xilem::style::Style as _;
use xilem::WidgetView;

use crate::menu_button::DEFAULT_ITEM_HEIGHT;
use crate::menu_items::{BoxedMenuEntry, MenuItem, SeparatorEntry, Submenu};
use crate::menu_button;

use super::builder::{MenuBarBuilder, MenuItemBuilder};
use super::action_trait::MenuActionTrait;

/// Default menu bar background color.
pub const MENU_BAR_BG: Color = Color::from_rgb8(45, 43, 40);
/// Default menu text color.
pub const MENU_TEXT_COLOR: Color = Color::from_rgb8(220, 218, 214);
/// Font size for menu labels.
pub const MENU_TEXT_SIZE: f32 = (DEFAULT_ITEM_HEIGHT * 0.43) as f32;

/// Styling options for the pulldown menu bar.
#[derive(Debug, Clone)]
pub struct PulldownMenuBarStyle {
    /// Text color for menu labels.
    pub text_color: Color,
    /// Font size for menu labels.
    pub text_size: f32,
    /// Background color for the menu bar.
    pub background_color: Color,
    /// Gap between menu buttons in pixels.
    pub gap: f64,
    /// Padding around the menu bar.
    pub padding: f64,
}

impl Default for PulldownMenuBarStyle {
    fn default() -> Self {
        Self {
            text_color: MENU_TEXT_COLOR,
            text_size: MENU_TEXT_SIZE,
            background_color: MENU_BAR_BG,
            gap: 8.0,
            padding: 6.0,
        }
    }
}

/// Converts a [`MenuBarBuilder`] into a horizontal row of pulldown menu buttons.
///
/// This is the Linux equivalent of `build_muda_menu()` — it takes the same
/// `MenuBarBuilder` definition and renders it as xilem widget-based menus.
///
/// The `handler` parameter dispatches `ActionEnum` items (those added via
/// `.action(MyAction::Variant)`). When such an item is clicked, the handler
/// is called with the downcast action value. Closure-based items (`.item(...)`)
/// are dispatched directly without the handler.
///
/// # Example
///
/// ```ignore
/// let menus = MenuBarBuilder::new()
///     .menu("File", |m| m
///         .action(MenuAction::New)
///         .action(MenuAction::Open)
///         .separator()
///         .item("Quit", |s: &mut AppModel| s.quit())
///     );
///
/// let menu_bar = pulldown_menu_bar(menus, handle_menu_action, PulldownMenuBarStyle::default());
/// ```
pub fn pulldown_menu_bar<State, Action, A>(
    builder: MenuBarBuilder<State, Action>,
    handler: fn(&mut State, A),
    style: PulldownMenuBarStyle,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: Default + 'static,
    A: MenuActionTrait,
{
    let tc = style.text_color;
    let ts = style.text_size;
    let bg = style.background_color;
    let gap = style.gap;
    let padding = style.padding;

    let buttons: Vec<_> = builder.menus.into_iter().map(|menu| {
        let items = convert_menu_items::<State, Action, A>(menu.items, handler);
        menu_button(
            label(menu.title).text_size(ts).color(tc),
            items,
        )
    }).collect();

    flex_row(buttons)
        .gap(Length::px(gap))
        .padding(padding)
        .background_color(bg)
}

/// Converts a `Vec<MenuItemBuilder>` into `Vec<BoxedMenuEntry>` for use
/// with pulldown menus.
///
/// - `Action` items → `MenuItem` with their closure
/// - `ActionEnum` items → `MenuItem` that calls `handler(state, action_value)`
/// - `Separator` → `SeparatorEntry`
/// - `Submenu` → `Submenu` wrapping recursively converted children
fn convert_menu_items<State, Action, A>(
    items: Vec<MenuItemBuilder<State, Action>>,
    handler: fn(&mut State, A),
) -> Vec<BoxedMenuEntry<State, Action>>
where
    State: 'static,
    Action: Default + 'static,
    A: MenuActionTrait,
{
    let mut entries: Vec<BoxedMenuEntry<State, Action>> = Vec::new();

    for item in items {
        match item {
            MenuItemBuilder::Action { label, action, .. } => {
                let action = Arc::clone(&action);
                entries.push(Box::new(MenuItem::new(label, move |state: &mut State| {
                    action(state)
                })));
            }
            MenuItemBuilder::ActionEnum { label, action_value, .. } => {
                // Downcast the stored action value and wire up the handler
                if let Some(action_val) = action_value.downcast_ref::<A>() {
                    let action_val = *action_val;
                    entries.push(Box::new(MenuItem::new(label, move |state: &mut State| {
                        handler(state, action_val);
                        Action::default()
                    })));
                } else {
                    // Type mismatch — create a labeled no-op item
                    entries.push(Box::new(MenuItem::new(label, move |_state: &mut State| {
                        Action::default()
                    })));
                }
            }
            MenuItemBuilder::Separator => {
                entries.push(Box::new(SeparatorEntry::<State, Action>::default()));
            }
            MenuItemBuilder::Submenu { label, items: sub_items } => {
                let children = convert_menu_items::<State, Action, A>(sub_items, handler);
                entries.push(Box::new(Submenu::new(label, children)));
            }
            MenuItemBuilder::Dynamic { .. } => {
                // Dynamic items require state to evaluate — skipped in static conversion.
            }
        }
    }

    entries
}
