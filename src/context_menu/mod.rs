//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Context menu (right-click popup menu) support.

mod dropdown;
mod view;
mod widget;

pub use dropdown::ContextMenuDropdown;
pub use view::{ContextMenuView, context_menu};
pub use widget::{ContextMenuAction, ContextMenuWidget};
