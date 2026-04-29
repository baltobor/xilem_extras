//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Application model for the gallery example.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use chrono::NaiveDate;
use xilem_extras::{
    ExpansionState, SingleSelection, MultiSelection, SortOrder, SortDirection, ColumnWidths, ColumnDef, column,
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
    TreeView,
    List,
    SectionedList,
    Table,
    VirtualTable,
    Tabs,
    Menu,
    AppMenu,
    Calendar,
    Widgets,
    Chart,
    StockChart,
}

/// Application state.
pub struct AppModel {
    /// Current demo page.
    pub page: Page,

    // Tree demo state
    pub file_tree: FileNode,
    pub tree_expansion: ExpansionState<String>,
    pub tree_selection: SingleSelection<String>,
    /// Last node activated by Enter / double-click, surfaced in the demo
    /// label so the user can verify Activate is wired up.
    pub tree_activated: Option<String>,
    /// Id of the node being inline-renamed, or `None` if not editing.
    pub tree_editing: Option<String>,
    /// Buffer text shown in the rename `text_input`.
    pub tree_editing_text: String,

    // List demo state
    pub contacts: Vec<Contact>,
    pub list_selection: MultiSelection<u64>,
    pub sectioned_list_selection: MultiSelection<u64>,

    // Table demo state
    pub cyclists: Vec<Cyclist>,
    pub table_selection: MultiSelection<u64>,
    pub table_sort: SortOrder,
    pub table_column_widths: ColumnWidths,
    pub last_click_mods: String,

    // Virtual Table demo state (10,000 rows)
    pub virtual_cyclists: Vec<Cyclist>,
    pub virtual_table_selection: MultiSelection<u64>,
    pub virtual_table_sort: SortOrder,
    pub virtual_table_columns: Vec<ColumnDef>,
    pub virtual_table_column_widths: ColumnWidths,

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

    // Calendar demo state
    pub calendar_selected_date: Option<NaiveDate>,
    pub calendar_hour: u8,
    pub calendar_minute: u8,

    // Widgets demo state
    pub widgets_text_light: String,
    pub widgets_text_dark: String,
    pub widgets_checkbox_1: bool,
    pub widgets_checkbox_2: bool,
    pub widgets_show_sheet: bool,

    // Chart demo state (simple bar/line)
    pub chart_mode: usize,  // 0=Bar, 1=Line
    pub chart_show_values: bool,

    // Stock chart demo state
    pub stock_chart_mode: usize,
    pub stock_chart_hover: Option<(String, f64)>,

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

/// Find the node's `name` field by `path`. Used to seed the rename buffer.
fn find_node_name(node: &FileNode, target: &str) -> Option<String> {
    if node.path == target {
        return Some(node.name.clone());
    }
    for child in &node.children {
        if let Some(found) = find_node_name(child, target) {
            return Some(found);
        }
    }
    None
}

impl AppModel {
    pub fn new() -> Self {
        use crate::mock_data;

        Self {
            page: Page::default(),

            // Tree
            file_tree: mock_data::mock_file_tree(),
            tree_expansion: ExpansionState::with_expanded([
                ".".to_string(),
                "src".to_string(),
                "src/components".to_string(),
                "tests".to_string(),
            ]),
            tree_selection: SingleSelection::new(),
            tree_activated: None,
            tree_editing: None,
            tree_editing_text: String::new(),

            // List
            contacts: mock_data::mock_contacts(),
            list_selection: MultiSelection::new(),
            sectioned_list_selection: MultiSelection::new(),

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

            // Virtual Table (10,000 rows for performance testing)
            virtual_cyclists: mock_data::mock_cyclists_large(10_000),
            virtual_table_selection: MultiSelection::new(),
            virtual_table_sort: SortOrder::single("name", SortDirection::Ascending),
            virtual_table_columns: vec![
                column("name", "Name").flex(2.0).build(),
                column("route", "Route").flex(2.0).build(),
                column("distance_km", "Distance").fixed(100.0).build(),
                column("joy_level", "Joy").fixed(60.0).build(),
            ],
            virtual_table_column_widths: ColumnWidths::from_columns(&[
                ("name", 200.0),
                ("route", 200.0),
                ("distance_km", 100.0),
                ("joy_level", 60.0),
            ]),

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

            // Calendar
            calendar_selected_date: None,
            calendar_hour: 12,
            calendar_minute: 0,

            // Widgets
            widgets_text_light: String::new(),
            widgets_text_dark: String::new(),
            widgets_checkbox_1: false,
            widgets_checkbox_2: true,
            widgets_show_sheet: false,

            // Chart (simple bar/line)
            chart_mode: 0,
            chart_show_values: true,

            // Stock chart
            stock_chart_mode: 0,
            stock_chart_hover: None,

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

    /// Begin inline-renaming the node with this id. Seeds the edit buffer
    /// with the node's current name so the text_input shows the same string
    /// the user is about to change.
    pub fn start_editing_tree_node(&mut self, id: &str) {
        if let Some(name) = find_node_name(&self.file_tree, id) {
            self.tree_editing = Some(id.to_string());
            self.tree_editing_text = name;
        }
    }

    /// Apply the rename. The node's `name` becomes `new_name`; identity
    /// (`path`) is preserved so selection / expansion state still points at
    /// the same row.
    pub fn rename_tree_node(&mut self, id: &str, new_name: String) {
        fn rename_in(node: &mut FileNode, target: &str, new_name: &str) -> bool {
            if node.path == target {
                node.name = new_name.to_string();
                return true;
            }
            for child in &mut node.children {
                if rename_in(child, target, new_name) {
                    return true;
                }
            }
            false
        }
        rename_in(&mut self.file_tree, id, &new_name);
        self.tree_editing = None;
        self.tree_editing_text.clear();
    }

    /// Cancel any inline rename in progress.
    pub fn cancel_editing_tree_node(&mut self) {
        self.tree_editing = None;
        self.tree_editing_text.clear();
    }

    /// Remove the node identified by `path` from the file tree. No-op if the
    /// path is empty (the root) or not found.
    pub fn delete_tree_node(&mut self, id: &str) {
        if id.is_empty() {
            return;
        }
        fn remove_in(node: &mut FileNode, target: &str) -> bool {
            if let Some(pos) = node.children.iter().position(|c| c.path == target) {
                node.children.remove(pos);
                return true;
            }
            for child in &mut node.children {
                if remove_in(child, target) {
                    return true;
                }
            }
            false
        }
        remove_in(&mut self.file_tree, id);
        // Clear selection / activation if it pointed at the removed node.
        if self.tree_selection.selected().map(|s| s.as_str()) == Some(id) {
            self.tree_selection.set(None);
        }
        if self.tree_activated.as_deref() == Some(id) {
            self.tree_activated = None;
        }
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
