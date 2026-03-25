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

### Row Button with Selection

```rust
use xilem_extras::{row_button, SelectionState, SelectionModifiers};

fn contact_row(contact: &Contact, selected: bool) -> impl WidgetView<AppModel> {
    let id = contact.id;
    let row = flex_row((
        label(contact.name.clone()),
        label(contact.email.clone()),
    ));

    row_button(row, move |model: &mut AppModel| {
        model.selection.select(id, SelectionModifiers::NONE);
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

## Technical Details

- **Compatibility**: Xilem main branch (git)
- **License**: Apache-2.0
- **Platform**: Tested on macOS

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
