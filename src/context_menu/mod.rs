//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Context menu (right-click popup menu) support.

mod widget;
mod view;
mod dropdown;

pub use widget::{ContextMenuWidget, ContextMenuAction};
pub use view::{context_menu, ContextMenuView};
pub use dropdown::ContextMenuDropdown;
