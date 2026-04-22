//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Application model for the gallery example.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use xilem::masonry::kurbo::Point;
use xilem_extras::{
    ExpansionState, SingleSelection, MultiSelection, SortOrder, SortDirection, ColumnWidths,
};

use xilem_extras::SimpleTab;

use crate::mock_data::{FileNode, Contact, Cyclist};
use crate::tabs_demo::{DemoTab, create_demo_tabs};

// Channel for menu commands (macOS/Windows only)
#[cfg(not(target_os = "linux"))]
use std::sync::{mpsc, Mutex};

/// Menu commands sent from the native menu bar to the application model.
#[cfg(not(target_os = "linux"))]
#[derive(Debug, Clone)]
pub enum MenuCommand {
    GotoPage(Page),
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    ToggleDarkMode,
    ToggleToolbar,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    Documentation,
    About,
}

/// Current demo page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    Tree,
    List,
    Table,
    Tabs,
    Menu,
    AppMenu,
}

/// Application state.
pub struct AppModel {
    /// Current demo page.
    pub page: Page,

    // Tree demo state
    pub file_tree: FileNode,
    pub tree_expansion: ExpansionState<String>,
    pub tree_selection: SingleSelection<String>,
    pub context_menu_position: Option<Point>,

    // List demo state
    pub contacts: Vec<Contact>,
    pub list_selection: MultiSelection<u64>,

    // Table demo state
    pub cyclists: Vec<Cyclist>,
    pub table_selection: MultiSelection<u64>,
    pub table_sort: SortOrder,
    pub table_column_widths: ColumnWidths,
    pub last_click_mods: String,

    // Tabs demo state
    pub demo_tabs: Vec<DemoTab>,
    pub demo_active_tab: usize,
    pub demo_show_tab_nav: bool,

    // NavTabBar demo state
    pub nav_tabs: Vec<SimpleTab>,
    pub nav_active_tab: usize,
    pub nav_show_arrows: bool,

    // Menu demo state
    pub menu_last_action: String,
    pub dropdown_selected_index: usize,
    pub dark_mode: bool,
    pub show_toolbar: bool,

    // Menu command channel (macOS/Windows only)
    #[cfg(not(target_os = "linux"))]
    pub menu_command_rx: Option<Arc<Mutex<mpsc::Receiver<MenuCommand>>>>,

    // Window ID for xilem window management
    pub main_window_id: masonry_winit::app::WindowId,

    // App lifecycle
    pub app_running: bool,

    /// Background activity flag - when true, the ticker task triggers re-renders.
    pub bg_active: Arc<AtomicBool>,
}

impl AppModel {
    pub fn new() -> Self {
        use crate::mock_data;

        Self {
            page: Page::default(),

            // Tree
            file_tree: mock_data::mock_file_tree(),
            tree_expansion: ExpansionState::with_expanded(["src".to_string()]),
            tree_selection: SingleSelection::new(),
            context_menu_position: None,

            // List
            contacts: mock_data::mock_contacts(),
            list_selection: MultiSelection::new(),

            // Table
            cyclists: mock_data::mock_cyclists(),
            table_selection: MultiSelection::new(),
            table_sort: SortOrder::single("name", SortDirection::Ascending),
            table_column_widths: ColumnWidths::from_columns(&[
                ("name", 120.0),
                ("route", 120.0),
                ("distance_km", 80.0),
                ("joy_level", 60.0),
            ]),
            last_click_mods: "(click a row)".to_string(),

            // Tabs
            demo_tabs: create_demo_tabs(),
            demo_active_tab: 0,
            demo_show_tab_nav: true,

            // NavTabBar
            nav_tabs: vec![
                SimpleTab::new("Overview"),
                SimpleTab::new("Details"),
                SimpleTab::new("Settings"),
                SimpleTab::new("History"),
            ],
            nav_active_tab: 0,
            nav_show_arrows: false,

            // Menu
            menu_last_action: "(none)".to_string(),
            dropdown_selected_index: 0,
            dark_mode: true,
            show_toolbar: true,

            // Menu command channel (set by main)
            #[cfg(not(target_os = "linux"))]
            menu_command_rx: None,

            // Window ID
            main_window_id: masonry_winit::app::WindowId::next(),

            // App lifecycle
            app_running: true,
            bg_active: Arc::new(AtomicBool::new(false)),
        }
    }

    // Tree actions
    pub fn toggle_tree_node(&mut self, id: &str) {
        self.tree_expansion.toggle(&id.to_string());
    }

    pub fn select_tree_node(&mut self, id: String) {
        self.tree_selection.set(Some(id));
    }

    // Tabs actions
    pub fn close_demo_tab(&mut self, index: usize) {
        if index < self.demo_tabs.len() {
            self.demo_tabs.remove(index);
            if self.demo_active_tab >= self.demo_tabs.len() && !self.demo_tabs.is_empty() {
                self.demo_active_tab = self.demo_tabs.len() - 1;
            }
        }
    }

    /// Poll and process menu commands from the native menu bar (macOS/Windows).
    #[cfg(not(target_os = "linux"))]
    pub fn poll_menu_commands(&mut self) {
        let commands: Vec<MenuCommand> = {
            let Some(rx_arc) = &self.menu_command_rx else {
                return;
            };
            let Ok(rx) = rx_arc.lock() else {
                return;
            };
            rx.try_iter().collect()
        };

        for cmd in commands {
            match cmd {
                MenuCommand::GotoPage(page) => self.page = page,
                MenuCommand::Undo => self.menu_last_action = "Edit > Undo".to_string(),
                MenuCommand::Redo => self.menu_last_action = "Edit > Redo".to_string(),
                MenuCommand::Cut => self.menu_last_action = "Edit > Cut".to_string(),
                MenuCommand::Copy => self.menu_last_action = "Edit > Copy".to_string(),
                MenuCommand::Paste => self.menu_last_action = "Edit > Paste".to_string(),
                MenuCommand::ToggleDarkMode => {
                    self.dark_mode = !self.dark_mode;
                    self.menu_last_action = format!("Dark Mode: {}", self.dark_mode);
                }
                MenuCommand::ToggleToolbar => {
                    self.show_toolbar = !self.show_toolbar;
                    self.menu_last_action = format!("Show Toolbar: {}", self.show_toolbar);
                }
                MenuCommand::ZoomIn => self.menu_last_action = "View > Zoom In".to_string(),
                MenuCommand::ZoomOut => self.menu_last_action = "View > Zoom Out".to_string(),
                MenuCommand::ZoomReset => self.menu_last_action = "View > Reset Zoom".to_string(),
                MenuCommand::Documentation => self.menu_last_action = "Help > Documentation".to_string(),
                MenuCommand::About => self.menu_last_action = "Help > About".to_string(),
            }
        }
    }
}

impl Default for AppModel {
    fn default() -> Self {
        Self::new()
    }
}

impl xilem::AppState for AppModel {
    fn keep_running(&self) -> bool {
        self.app_running
    }
}
