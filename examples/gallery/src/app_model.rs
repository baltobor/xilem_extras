//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Application model for the gallery example.

use xilem_extras::{
    ExpansionState, SingleSelection, MultiSelection, SortOrder, SortDirection, ColumnWidths,
};

use crate::mock_data::{FileNode, Contact, Cyclist};
use crate::tabs_demo::{DemoTab, create_demo_tabs};

/// Current demo page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    Tree,
    List,
    Table,
    Tabs,
    Menu,
}

/// Application state.
pub struct AppModel {
    /// Current demo page.
    pub page: Page,

    // Tree demo state
    pub file_tree: FileNode,
    pub tree_expansion: ExpansionState<String>,
    pub tree_selection: SingleSelection<String>,

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

    // Menu demo state
    pub menu_last_action: String,
    pub dropdown_selected_index: usize,
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

            // Menu
            menu_last_action: "(none)".to_string(),
            dropdown_selected_index: 0,
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

}

impl Default for AppModel {
    fn default() -> Self {
        Self::new()
    }
}
