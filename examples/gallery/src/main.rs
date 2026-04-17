//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Gallery example demonstrating xilem_extras widgets.
//!
//! This example uses platform-specific menu handling:
//! - On macOS/Windows: Native muda menus (system menu bar)
//! - On Linux: xilem_extras menu_button widgets (in-window menu bar)

mod app_model;
mod mock_data;
mod tree_demo;
mod list_demo;
mod table_demo;
mod tabs_demo;
mod menu_demo;
mod app_menu_demo;

use masonry::layout::AsUnit;
use masonry::theme::default_property_set;
use xilem::masonry::peniko::Color;
use xilem::style::Style;
use xilem::core::fork;
use xilem::view::{CrossAxisAlignment, split, flex_col, label, task_raw};
use xilem::{window, EventLoop, WidgetView, WindowView, Xilem};
use std::sync::atomic::Ordering;
use std::time::Duration;

// External event loop support
use masonry_winit::app::{AppDriver, MasonryUserEvent, MasonryState};
use xilem::winit::application::ApplicationHandler;
use xilem::winit::event_loop::ActiveEventLoop;
use xilem::winit::window::WindowId as WinitWindowId;
use xilem::winit::event::{WindowEvent, DeviceEvent, DeviceId, StartCause};

use xilem_extras::row_button;
use app_model::{AppModel, Page};

// Material Symbols font
use xilem_material_icons::FONT_DATA;

// Linux-only: menu_button imports for fallback menu bar
#[cfg(target_os = "linux")]
use xilem_extras::{menu_button, menu_item, separator, submenu};
#[cfg(target_os = "linux")]
use xilem_extras::menu_button::DEFAULT_ITEM_HEIGHT;

// macOS/Windows: muda for native menus (re-exported from xilem_extras)
#[cfg(not(target_os = "linux"))]
use xilem_extras::app_menu::muda::{Menu, MenuItem, MenuEvent, PredefinedMenuItem, Submenu, CheckMenuItem, accelerator::Accelerator};

// Channel for menu commands (macOS/Windows)
#[cfg(not(target_os = "linux"))]
use std::sync::{mpsc, Arc, Mutex};

const TEXT_COLOR: Color = Color::from_rgb8(220, 218, 214);
const TEXT_SECONDARY: Color = Color::from_rgb8(160, 156, 150);
const BG_NAV: Color = Color::from_rgb8(45, 43, 40);
const BG_HOVER: Color = Color::from_rgb8(55, 52, 48);
const BG_ACTIVE: Color = Color::from_rgb8(65, 62, 58);

#[cfg(target_os = "linux")]
const MENU_BAR_BG: Color = Color::from_rgb8(38, 36, 33);

/// Menu bar height derived from default item height (Linux only).
#[cfg(target_os = "linux")]
const MENU_TEXT_SIZE: f32 = (DEFAULT_ITEM_HEIGHT * 0.43) as f32;
#[cfg(target_os = "linux")]
const MENU_PADDING_V: f64 = (DEFAULT_ITEM_HEIGHT - MENU_TEXT_SIZE as f64) / 2.0;
#[cfg(target_os = "linux")]
const MENU_PADDING_H: f64 = 10.0;

/// Build the application menu bar using xilem_extras menu_button.
/// This is the fallback for Linux only.
#[cfg(target_os = "linux")]
fn build_menu_bar(model: &mut AppModel) -> impl WidgetView<AppModel> + use<> {
    use xilem::style::Padding;

    let menu_label = |text: &str| {
        label(text.to_string())
            .text_size(MENU_TEXT_SIZE)
            .color(TEXT_COLOR)
            .padding(Padding {
                top: MENU_PADDING_V,
                bottom: MENU_PADDING_V,
                left: MENU_PADDING_H,
                right: MENU_PADDING_H,
            })
    };

    flex_row((
        // Examples menu - Navigate to different demos
        menu_button(
            menu_label("Examples"),
            (
                menu_item("Tree View", |model: &mut AppModel| {
                    model.page = Page::Tree;
                }),
                menu_item("List View", |model: &mut AppModel| {
                    model.page = Page::List;
                }),
                menu_item("Table View", |model: &mut AppModel| {
                    model.page = Page::Table;
                }),
                menu_item("Tabs", |model: &mut AppModel| {
                    model.page = Page::Tabs;
                }),
                separator(),
                menu_item("Pulldown Menus", |model: &mut AppModel| {
                    model.page = Page::Menu;
                }),
                menu_item("App Menu Bar", |model: &mut AppModel| {
                    model.page = Page::AppMenu;
                }),
            ),
        ),
        // Edit menu - Standard editing operations
        menu_button(
            menu_label("Edit"),
            (
                menu_item("Undo", |model: &mut AppModel| {
                    model.menu_last_action = "Edit > Undo".to_string();
                }),
                menu_item("Redo", |model: &mut AppModel| {
                    model.menu_last_action = "Edit > Redo".to_string();
                }),
                separator(),
                menu_item("Cut", |model: &mut AppModel| {
                    model.menu_last_action = "Edit > Cut".to_string();
                }),
                menu_item("Copy", |model: &mut AppModel| {
                    model.menu_last_action = "Edit > Copy".to_string();
                }),
                menu_item("Paste", |model: &mut AppModel| {
                    model.menu_last_action = "Edit > Paste".to_string();
                }),
            ),
        ),
        // View menu - Display settings
        menu_button(
            menu_label("View"),
            (
                menu_item("Dark Mode", |model: &mut AppModel| {
                    model.dark_mode = !model.dark_mode;
                    model.menu_last_action = format!("Dark Mode: {}", model.dark_mode);
                }).checked(model.dark_mode),
                menu_item("Show Toolbar", |model: &mut AppModel| {
                    model.show_toolbar = !model.show_toolbar;
                    model.menu_last_action = format!("Show Toolbar: {}", model.show_toolbar);
                }).checked(model.show_toolbar),
                separator(),
                submenu("Zoom", (
                    menu_item("Zoom In", |model: &mut AppModel| {
                        model.menu_last_action = "View > Zoom In".to_string();
                    }),
                    menu_item("Zoom Out", |model: &mut AppModel| {
                        model.menu_last_action = "View > Zoom Out".to_string();
                    }),
                    menu_item("Reset Zoom", |model: &mut AppModel| {
                        model.menu_last_action = "View > Reset Zoom".to_string();
                    }),
                )),
            ),
        ),
        // Help menu
        menu_button(
            menu_label("Help"),
            (
                menu_item("Documentation", |model: &mut AppModel| {
                    model.menu_last_action = "Help > Documentation".to_string();
                }),
                separator(),
                menu_item("About xilem_extras", |model: &mut AppModel| {
                    model.menu_last_action = "Help > About".to_string();
                }),
            ),
        ),
    ))
    .gap(0.px())
    .height((DEFAULT_ITEM_HEIGHT as i32).px())
    .background_color(MENU_BAR_BG)
}

fn nav_button(text: &str, page: Page, current: Page) -> impl WidgetView<AppModel> + use<'_> {
    let is_active = page == current;
    let bg = if is_active { BG_ACTIVE } else { BG_NAV };

    row_button(
        label(text.to_string())
            .text_size(13.0)
            .color(TEXT_COLOR)
            .padding(8.0),
        move |model: &mut AppModel| {
            model.page = page;
        },
    )
    .hover_bg(BG_HOVER)
    .background_color(bg)
}

fn app_logic(model: &mut AppModel) -> impl WidgetView<AppModel> + use<> {
    // Poll and process native menu commands (macOS/Windows)
    #[cfg(not(target_os = "linux"))]
    model.poll_menu_commands();

    let current_page = model.page;

    // Main content area with sidebar
    let main_content = split(
        // Navigation sidebar
        flex_col((
            label("xilem_extras")
                .text_size(14.0)
                .weight(xilem::FontWeight::BOLD)
                .color(TEXT_COLOR),
            label("Gallery")
                .text_size(12.0)
                .color(TEXT_SECONDARY),
            nav_button("Tree", Page::Tree, current_page),
            nav_button("List", Page::List, current_page),
            nav_button("Table", Page::Table, current_page),
            nav_button("Tabs", Page::Tabs, current_page),
            nav_button("Menu", Page::Menu, current_page),
            nav_button("App Menu", Page::AppMenu, current_page),
        ))
        .cross_axis_alignment(CrossAxisAlignment::Stretch)
        .gap(4.px())
        .padding(12.0)
        .background_color(BG_NAV),

        // Demo content
        match model.page {
            Page::Tree => tree_demo::tree_demo(model).boxed(),
            Page::List => list_demo::list_demo(model).boxed(),
            Page::Table => table_demo::table_demo(model).boxed(),
            Page::Tabs => tabs_demo::tabs_demo(model).boxed(),
            Page::Menu => menu_demo::menu_demo(model).boxed(),
            Page::AppMenu => app_menu_demo::app_menu_demo(model).boxed(),
        },
    )
    .split_point_from_start(160.px())
    .min_lengths(120.px(), 200.px())
    .bar_thickness(1.px())
    .solid_bar(true);

    // On Linux: include the xilem menu bar at the top
    // On macOS/Windows: native menu bar is used, no in-window menu needed
    #[cfg(target_os = "linux")]
    {
        flex_col((
            build_menu_bar(model),
            main_content,
        ))
        .cross_axis_alignment(CrossAxisAlignment::Stretch)
    }

    #[cfg(not(target_os = "linux"))]
    {
        // Wrap in a background ticker that triggers re-renders when bg_active is set
        let bg_flag = model.bg_active.clone();
        fork(
            main_content,
            task_raw::<(), _, _, _, _, _>(
                move |proxy, _state: &mut AppModel| {
                    let flag = bg_flag.clone();
                    async move {
                        let mut interval = tokio::time::interval(Duration::from_millis(100));
                        loop {
                            interval.tick().await;
                            if flag.load(Ordering::Relaxed) {
                                flag.store(false, Ordering::Relaxed);
                                if proxy.message(()).is_err() {
                                    break;
                                }
                            }
                        }
                    }
                },
                |_model: &mut AppModel, ()| {
                    // The message triggers a re-render; poll_menu_commands runs at top of app_logic
                },
            ),
        )
    }
}

/// Menu action IDs for muda (macOS/Windows)
#[cfg(not(target_os = "linux"))]
mod menu_ids {
    pub const TREE: &str = "tree";
    pub const LIST: &str = "list";
    pub const TABLE: &str = "table";
    pub const TABS: &str = "tabs";
    pub const MENU: &str = "menu";
    pub const APP_MENU: &str = "app_menu";
    pub const UNDO: &str = "undo";
    pub const REDO: &str = "redo";
    pub const CUT: &str = "cut";
    pub const COPY: &str = "copy";
    pub const PASTE: &str = "paste";
    pub const DARK_MODE: &str = "dark_mode";
    pub const SHOW_TOOLBAR: &str = "show_toolbar";
    pub const ZOOM_IN: &str = "zoom_in";
    pub const ZOOM_OUT: &str = "zoom_out";
    pub const ZOOM_RESET: &str = "zoom_reset";
    pub const DOCUMENTATION: &str = "documentation";
    pub const ABOUT: &str = "about";
}


/// Build native muda menu bar for macOS/Windows
#[cfg(not(target_os = "linux"))]
fn build_native_menu() -> Menu {
    use menu_ids::*;

    let menu = Menu::new();

    // macOS App menu (About, Services, Hide, Quit) - required for Cmd+Q and window close
    #[cfg(target_os = "macos")]
    {
        let app_menu = Submenu::new("Gallery", true);
        let _ = app_menu.append_items(&[
            &PredefinedMenuItem::about(None, None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::services(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::hide(None),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::show_all(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::quit(None),
        ]);
        let _ = menu.append(&app_menu);
    }

    // Examples menu - Navigate to different demos
    let examples_menu = Submenu::new("Examples", true);
    examples_menu.append(&MenuItem::with_id(TREE, "Tree View", true, None::<Accelerator>)).ok();
    examples_menu.append(&MenuItem::with_id(LIST, "List View", true, None::<Accelerator>)).ok();
    examples_menu.append(&MenuItem::with_id(TABLE, "Table View", true, None::<Accelerator>)).ok();
    examples_menu.append(&MenuItem::with_id(TABS, "Tabs", true, None::<Accelerator>)).ok();
    examples_menu.append(&PredefinedMenuItem::separator()).ok();
    examples_menu.append(&MenuItem::with_id(MENU, "Pulldown Menus", true, None::<Accelerator>)).ok();
    examples_menu.append(&MenuItem::with_id(APP_MENU, "App Menu Bar", true, None::<Accelerator>)).ok();
    menu.append(&examples_menu).ok();

    // Edit menu
    let edit_menu = Submenu::new("Edit", true);
    edit_menu.append(&MenuItem::with_id(UNDO, "Undo", true, "CmdOrCtrl+Z".parse::<Accelerator>().ok())).ok();
    edit_menu.append(&MenuItem::with_id(REDO, "Redo", true, "CmdOrCtrl+Shift+Z".parse::<Accelerator>().ok())).ok();
    edit_menu.append(&PredefinedMenuItem::separator()).ok();
    edit_menu.append(&MenuItem::with_id(CUT, "Cut", true, "CmdOrCtrl+X".parse::<Accelerator>().ok())).ok();
    edit_menu.append(&MenuItem::with_id(COPY, "Copy", true, "CmdOrCtrl+C".parse::<Accelerator>().ok())).ok();
    edit_menu.append(&MenuItem::with_id(PASTE, "Paste", true, "CmdOrCtrl+V".parse::<Accelerator>().ok())).ok();
    menu.append(&edit_menu).ok();

    // View menu
    let view_menu = Submenu::new("View", true);
    view_menu.append(&CheckMenuItem::with_id(DARK_MODE, "Dark Mode", true, true, None::<Accelerator>)).ok();
    view_menu.append(&CheckMenuItem::with_id(SHOW_TOOLBAR, "Show Toolbar", true, true, None::<Accelerator>)).ok();
    view_menu.append(&PredefinedMenuItem::separator()).ok();
    let zoom_menu = Submenu::new("Zoom", true);
    zoom_menu.append(&MenuItem::with_id(ZOOM_IN, "Zoom In", true, "CmdOrCtrl+=".parse::<Accelerator>().ok())).ok();
    zoom_menu.append(&MenuItem::with_id(ZOOM_OUT, "Zoom Out", true, "CmdOrCtrl+-".parse::<Accelerator>().ok())).ok();
    zoom_menu.append(&MenuItem::with_id(ZOOM_RESET, "Reset Zoom", true, "CmdOrCtrl+0".parse::<Accelerator>().ok())).ok();
    view_menu.append(&zoom_menu).ok();
    menu.append(&view_menu).ok();

    // Help menu
    let help_menu = Submenu::new("Help", true);
    help_menu.append(&MenuItem::with_id(DOCUMENTATION, "Documentation", true, None::<Accelerator>)).ok();
    help_menu.append(&PredefinedMenuItem::separator()).ok();
    help_menu.append(&MenuItem::with_id(ABOUT, "About xilem_extras", true, None::<Accelerator>)).ok();
    menu.append(&help_menu).ok();

    menu
}

/// Application wrapper for external event loop with muda integration
struct GalleryApp {
    masonry_state: MasonryState<'static>,
    app_driver: Box<dyn AppDriver>,
    #[cfg(not(target_os = "linux"))]
    _menu_bar: Menu,
    #[cfg(not(target_os = "linux"))]
    menu_command_tx: mpsc::Sender<app_model::MenuCommand>,
    /// Background activity flag - triggers re-renders when set.
    bg_active: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl ApplicationHandler<MasonryUserEvent> for GalleryApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.masonry_state
            .handle_resumed(event_loop, &mut *self.app_driver);
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        self.masonry_state.handle_suspended(event_loop);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Poll muda menu events and send commands through the channel (macOS/Windows)
        #[cfg(not(target_os = "linux"))]
        {
            use menu_ids::*;
            use app_model::MenuCommand;

            while let Ok(event) = MenuEvent::receiver().try_recv() {
                let cmd = match event.id().0.as_str() {
                    TREE => Some(MenuCommand::GotoPage(Page::Tree)),
                    LIST => Some(MenuCommand::GotoPage(Page::List)),
                    TABLE => Some(MenuCommand::GotoPage(Page::Table)),
                    TABS => Some(MenuCommand::GotoPage(Page::Tabs)),
                    MENU => Some(MenuCommand::GotoPage(Page::Menu)),
                    APP_MENU => Some(MenuCommand::GotoPage(Page::AppMenu)),
                    UNDO => Some(MenuCommand::Undo),
                    REDO => Some(MenuCommand::Redo),
                    CUT => Some(MenuCommand::Cut),
                    COPY => Some(MenuCommand::Copy),
                    PASTE => Some(MenuCommand::Paste),
                    DARK_MODE => Some(MenuCommand::ToggleDarkMode),
                    SHOW_TOOLBAR => Some(MenuCommand::ToggleToolbar),
                    ZOOM_IN => Some(MenuCommand::ZoomIn),
                    ZOOM_OUT => Some(MenuCommand::ZoomOut),
                    ZOOM_RESET => Some(MenuCommand::ZoomReset),
                    DOCUMENTATION => Some(MenuCommand::Documentation),
                    ABOUT => Some(MenuCommand::About),
                    _ => None,
                };
                if let Some(cmd) = cmd {
                    let _ = self.menu_command_tx.send(cmd);
                    // Set bg_active to trigger re-render via background ticker
                    self.bg_active.store(true, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }

        self.masonry_state.handle_about_to_wait(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WinitWindowId,
        event: WindowEvent,
    ) {
        self.masonry_state.handle_window_event(
            event_loop,
            window_id,
            event,
            self.app_driver.as_mut(),
        );
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: MasonryUserEvent) {
        self.masonry_state
            .handle_user_event(event_loop, event, self.app_driver.as_mut());
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        self.masonry_state.handle_device_event(
            event_loop,
            device_id,
            event,
            self.app_driver.as_mut(),
        );
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        self.masonry_state.handle_new_events(event_loop, cause);
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        self.masonry_state.handle_exiting(event_loop);
    }

    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
        self.masonry_state.handle_memory_warning(event_loop);
    }
}

/// Window logic function for Xilem::new (returns iterator of WindowView)
fn gallery_window_logic(model: &mut AppModel) -> impl Iterator<Item = WindowView<AppModel>> + use<> {
    let window_size = xilem::winit::dpi::LogicalSize::new(900.0, 600.0);
    let main_window = window(
        model.main_window_id,
        "xilem_extras Gallery",
        app_logic(model),
    )
    .with_options(move |o| {
        o.with_initial_inner_size(window_size)
            .on_close(|model: &mut AppModel| {
                model.app_running = false;
            })
    });

    std::iter::once(main_window)
}

fn main() {
    // Build event loop - disable winit's default menu on macOS so muda owns it
    let mut event_loop_builder = EventLoop::with_user_event();
    #[cfg(target_os = "macos")]
    {
        use xilem::winit::platform::macos::EventLoopBuilderExtMacOS;
        event_loop_builder.with_default_menu(false);
    }
    let event_loop = event_loop_builder.build().expect("Failed to build event loop");

    // Create proxy for event integration
    let proxy = event_loop.create_proxy();

    // Create menu command channel (macOS/Windows)
    #[cfg(not(target_os = "linux"))]
    let (menu_command_tx, menu_command_rx) = mpsc::channel::<app_model::MenuCommand>();

    // Build native menu bar for macOS/Windows
    #[cfg(not(target_os = "linux"))]
    let native_menu = build_native_menu();

    // Set up native menu on macOS
    #[cfg(target_os = "macos")]
    {
        native_menu.init_for_nsapp();
    }

    // Create app model with menu command channel
    let mut model = AppModel::new();
    #[cfg(not(target_os = "linux"))]
    {
        model.menu_command_rx = Some(Arc::new(Mutex::new(menu_command_rx)));
    }

    // Clone bg_active before passing model to Xilem
    let bg_active = model.bg_active.clone();

    // Create xilem app using Xilem::new with window logic (like rust_conductor)
    let xilem = Xilem::new(model, gallery_window_logic)
        .with_font(FONT_DATA.to_vec());

    // Get driver and windows from xilem
    let (driver, windows) =
        xilem.into_driver_and_windows(move |event| proxy.send_event(event).map_err(|err| err.0));

    // Create masonry state
    let masonry_state = MasonryState::new(
        event_loop.create_proxy(),
        windows,
        default_property_set(),
    );

    // Create and run the app
    let mut app = GalleryApp {
        masonry_state,
        app_driver: Box::new(driver),
        #[cfg(not(target_os = "linux"))]
        _menu_bar: native_menu,
        #[cfg(not(target_os = "linux"))]
        menu_command_tx,
        bg_active,
    };

    event_loop.run_app(&mut app).unwrap();
}
