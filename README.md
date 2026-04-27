# xilem_extras

High-level widget library for [Xilem](https://github.com/linebender/xilem) providing Tree, List, Table, and Menu widgets with SwiftUI-inspired APIs.

***This crate is highly experimental and not for productive use. The API may break anytime. use it at your own risk.***

## Overview

This library extends Xilem with common UI patterns for building desktop applications. It provides composable, trait-based widgets that integrate seamlessly with Xilem's reactive architecture.

## Available Widgets

- **Tree View** - Hierarchical data with expand/collapse, selection, and keyboard navigation
- **List View** - Selectable lists with sections and disclosure groups
- **Table View** - Sortable data grids with resizable columns and multi-column sorting
- **Menu Button** - Pulldown menus with separators (macOS-style)
- **Dropdown Select** - Value selection dropdowns
- **Row Button** - Full-width clickable rows with hover states
- **Tabs** - Tab bar with closeable tabs

## Quick Start

```bash
just run
```

This runs the gallery with all features enabled. [Just](https://github.com/casey/just) is a command runner. Install it with:

```bash
cargo install just
```

Or directly without just:

```bash
cargo run --example gallery --features "rust-logos app-menu"
```

The `rust-logos` feature provides Rust file icons, and `app-menu` enables native menu bar support (muda on macOS/Windows).

<img width="1012" height="744" alt="xilem_extras_gallery" src="https://github.com/user-attachments/assets/4eab7538-43f5-4cbd-a527-8182c0e78ddb" />


## Core Traits

The library uses trait-based design for flexibility:

```rust
// Stable identity for collections
pub trait Identifiable {
    type Id: Clone + Eq + Hash + Send + 'static;
    fn id(&self) -> Self::Id;
}

// Hierarchical data
pub trait TreeNode: Identifiable + Sized {
    fn children(&self) -> &[Self];
    fn is_expandable(&self) -> bool;
    fn label(&self) -> &str;
}

// Selection strategy
pub trait SelectionState<Id: Clone + Eq + Hash> {
    fn is_selected(&self, id: &Id) -> bool;
    fn select(&mut self, id: Id, modifiers: SelectionModifiers);
    fn clear(&mut self);
}
```

## Selection Types

- `SingleSelection<Id>` - One item at a time, toggle on re-click
- `MultiSelection<Id>` - Cmd/Ctrl+click toggle, Shift+click range

## Sort Descriptors

Table sorting uses Rust's iterator pattern:

```rust
let sorted = sort_order.sorted(&cyclists);
let filtered = sort_order.filter_sorted(&cyclists, |c| c.joy_level >= 8);
```

## Examples

### Tree Group

```rust
use xilem_extras::{tree_group, TreeAction, ExpansionState, SingleSelection};

tree_group(
    &model.file_tree,           // root node (implements TreeNode)
    &model.expansion,           // ExpansionState<NodeId>
    Some(&model.selection),     // Option<&impl SelectionState<NodeId>>
    |node, depth, is_expanded, is_selected| {
        // Build row view for each node
        flex_row((
            if node.is_expandable() {
                disclosure(is_expanded).boxed()
            } else {
                sized_box(()).width(16.0).boxed()
            },
            label(node.label()),
        ))
        .padding_left(depth as f64 * 16.0)
        .background_color(if is_selected { BG_SELECTED } else { Color::TRANSPARENT })
        .boxed()
    },
    |state, node_id, action| {
        match action {
            TreeAction::Toggle => state.expansion.toggle(node_id),
            TreeAction::Select => state.selection.set(Some(node_id.clone())),
            TreeAction::DoubleClick => state.open_file(node_id),
        }
    },
)
```

### Row Button with Multi-Selection

Use `row_button_with_modifiers` to capture Cmd/Ctrl+Click for multi-selection:

```rust
use xilem_extras::{row_button_with_modifiers, SelectionState, SelectionModifiers};

fn contact_row(contact: &Contact, selected: bool) -> impl WidgetView<AppModel> {
    let id = contact.id;
    let row = flex_row((
        label(contact.name.clone()),
        label(contact.email.clone()),
    ));

    row_button_with_modifiers(row, move |model: &mut AppModel, modifiers| {
        // Convert platform modifiers to SelectionModifiers
        // (Cmd on macOS, Ctrl on Windows/Linux)
        let sel_mods = SelectionModifiers::from_modifiers(modifiers);
        model.selection.select(id, sel_mods);
    })
    .hover_bg(BG_HOVER)
    .background_color(if selected { BG_SELECTED } else { Color::TRANSPARENT })
}
```

### Menu Button

```rust
use xilem_extras::{menu_button, menu_item, separator};

menu_button(
    label("File"),
    (
        menu_item("New", |model: &mut AppModel| model.new_file()),
        menu_item("Open...", |model| model.open_file()),
        separator(),
        menu_item("Save", |model| model.save_file()),
        menu_item("Exit", |model| model.exit()),
    ),
)
```

### Context Menu

```rust
use xilem_extras::{context_menu, menu_item, separator};

context_menu(
    label("Right-click me"),
    (
        menu_item("Cut", |model: &mut AppModel| model.cut()),
        menu_item("Copy", |model| model.copy()),
        separator(),
        menu_item("Paste", |model| model.paste()),
    ),
)
```

### App Menu Bar (xilem_muda)

The `app_menu` module provides a unified API for application menu bars:
- **macOS/Windows**: Native menus via the muda crate (requires `app-menu` feature)
- **Linux**: Fallback using `menu_button` widgets

**Builder API** (for defining menu structure):

```rust
use xilem_extras::{MenuBarBuilder, Key, CMD, SHIFT};

// Define menu structure with the fluent builder API
let menu_def = MenuBarBuilder::new()
    .menu("File", |m| m
        .item("New", |s: &mut AppState| s.new_file())
            .shortcut(CMD + Key::N)
        .item("Open...", |s| s.open_file())
            .shortcut(CMD + Key::O)
        .separator()
        .submenu("Recent", |m| m
            .item("project1.rs", |s| s.open_recent(0))
            .item("project2.rs", |s| s.open_recent(1))
        )
        .separator()
        .item("Quit", |s| s.quit())
            .shortcut(CMD + Key::Q)
    )
    .menu("Edit", |m| m
        .item("Undo", |s| s.undo())
            .shortcut(CMD + Key::Z)
            .enabled(|s| s.can_undo())
        .item("Redo", |s| s.redo())
            .shortcut(CMD + SHIFT + Key::Z)
            .enabled(|s| s.can_redo())
    );
```

**Xilem Widget Rendering** (for Linux and cross-platform fallback):

```rust
use xilem_extras::{menu_button, menu_item, separator, submenu};
use xilem::view::{flex_row, label};

// Build application menu bar with menu_button widgets
fn build_menu_bar(model: &mut AppModel) -> impl WidgetView<AppModel> {
    flex_row((
        menu_button(
            label("File"),
            (
                menu_item("New", |m: &mut AppModel| m.new_file()),
                menu_item("Open...", |m| m.open_file()),
                separator(),
                submenu("Recent", (
                    menu_item("project1.rs", |m| m.open_recent(0)),
                    menu_item("project2.rs", |m| m.open_recent(1)),
                )),
                separator(),
                menu_item("Quit", |m| m.quit()),
            ),
        ),
        menu_button(
            label("Edit"),
            (
                menu_item("Undo", |m: &mut AppModel| m.undo()),
                menu_item("Redo", |m| m.redo()),
            ),
        ),
    ))
    .background_color(MENU_BAR_BG)
}
```

**Shortcut DSL**:

```rust
use xilem_extras::{Key, CMD, SHIFT, ALT, CTRL};

// Platform-aware shortcuts
CMD + Key::S           // Cmd+S on macOS, Ctrl+S on Windows/Linux
CMD + SHIFT + Key::Z   // Cmd+Shift+Z / Ctrl+Shift+Z
ALT + Key::F4          // Alt+F4 (always Alt)
CTRL + Key::C          // Ctrl+C (always Ctrl, even on macOS)
```

### List

```rust
use xilem_extras::{list_styled, ListAction, ListStyle};

list_styled(
    &model.contacts,
    &model.selection,
    ListStyle::new().hover_bg(BG_HOVER),
    |contact, is_selected| {
        // Build row view - clone data, don't borrow
        let name = contact.name.clone();
        flex_row((label(name),))
            .background_color(if is_selected { BG_SELECTED } else { Color::TRANSPARENT })
    },
    |state, action| {
        match action {
            ListAction::Select(id, mods) => state.selection.select(id, mods),
            ListAction::Activate(id) => state.open_contact(&id),
        }
    },
)
```

### Table

<img width="1012" height="744" alt="xilem_extras_gallery" src="https://github.com/user-attachments/assets/0404e161-7ef6-447d-b855-1c92d1741262" />


```rust
use xilem_extras::{table, table_cell, column, TableAction, Alignment};

table(
    &model.employees,
    &[
        column("name", "Name").flex(2.0).build(),
        column("department", "Department").flex(1.5).build(),
        column("salary", "Salary").fixed(100.0).align(Alignment::End).build(),
    ],
    &model.column_widths,
    &model.selection,
    &model.sort_order,
    // Row builder receives column widths for proper cell sizing
    |state, idx, is_selected, is_striped, widths| {
        let employee = &state.employees[idx];
        flex_row((
            // table_cell clips content to prevent overflow when columns are resized
            table_cell(label(employee.name.clone()).padding(4.0), widths[0]),
            table_cell(label(employee.department.clone()).padding(4.0), widths[1]),
            table_cell(label(format!("${:.0}", employee.salary)).padding(4.0), widths[2]),
        ))
        .background_color(if is_selected { BG_SELECTED } else { Color::TRANSPARENT })
    },
    |state, action| {
        match action {
            TableAction::Sort(col, dir) => state.sort_order = SortOrder::single(&col, dir),
            TableAction::Select(id, mods) => state.selection.select(id, mods),
            TableAction::Activate(id) => state.edit_employee(&id),
            TableAction::ColumnResized(key, width) => state.column_widths.set(&key, width),
        }
    },
)
```

## Compatibility

- **Compatibility**: Xilem main branch (git)
- **Platforms**: Tested on macOS Tahoe 26.3.1

## Just Commands

```bash
just build   # Build the library
just run     # Run the widget gallery
just doc     # Generate documentation
just test    # Run tests
just lint    # Run clippy
```

## Author

Jacek Wisniowski

## Attribution

Built on [Xilem](https://github.com/linebender/xilem) by the Linebender team.

## Licence

This crate is licensed under the Apache License 2.0.

### Rust Logo (Optional Feature)

Rust logo SVG icons are available via the `rust-logos` feature:

```toml
[dependencies]
xilem_extras = { version = "0.0.2", features = ["rust-logos"] }
```

This provides `rust_logo()`, `rust_gear()`, `rust_logo_complete()`, and `ferris()`.
The Rust logo and Ferris mascot are from [rust-lang/rust-artwork](https://github.com/rust-lang/rust-artwork) 
and are licensed under [CC-BY (Creative Commons Attribution)](https://creativecommons.org/licenses/by/4.0/). 
If you enable this feature, you must comply with CC-BY requirements.

This software is provided as-is, without warranty. Use at your own risk.
