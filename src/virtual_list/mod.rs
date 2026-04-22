//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Virtual list for efficient rendering of large datasets.
//!
//! Only renders visible items based on scroll position, making it suitable
//! for lists with thousands of items.
//!
//! # Example
//!
//! ```ignore
//! use xilem_extras::virtual_list;
//!
//! virtual_list(
//!     items.len(),
//!     28.0,  // row height
//!     |idx| &items[idx],
//!     |item, idx| create_row(item, idx),
//! )
//! ```

mod widget;
mod view;

pub use view::{virtual_list, virtual_list_styled, VirtualListStyle};
pub use widget::{VirtualListWidget, VirtualListState};
