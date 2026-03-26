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

Or directly:

```bash
cargo run --example gallery
```

<img width="654" height="496" alt="xilem_extras_gallery" src="https://github.com/user-attachments/assets/70053df6-ca28-4d39-9a97-93f77bd3f15d" />

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
use xilem_extras::menu_button;

// Use "---" for separators
menu_button(
    label("File"),
    vec![
        "New".to_string(),
        "Open...".to_string(),
        "---".to_string(),
        "Save".to_string(),
        "Exit".to_string(),
    ],
    |model: &mut AppModel, index: usize| {
        match index {
            0 => model.new_file(),
            1 => model.open_file(),
            // index 2 is separator
            3 => model.save_file(),
            4 => model.exit(),
            _ => {}
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

This software is provided as-is, without warranty. Use at your own risk.
