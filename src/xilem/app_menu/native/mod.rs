//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Platform-specific menu bar backends.
//!
//! - macOS/Windows: Uses muda for native menu bars (requires "app-menu" feature)
//! - Linux: Falls back to xilem_extras menu_button widgets

#[cfg(all(feature = "app-menu", not(target_os = "linux")))]
mod muda_backend;

#[cfg(target_os = "linux")]
mod fallback;

#[cfg(all(feature = "app-menu", not(target_os = "linux")))]
pub use muda_backend::{MudaMenuBar, build_muda_menu};

#[cfg(target_os = "linux")]
pub use fallback::FallbackMenuBar;
