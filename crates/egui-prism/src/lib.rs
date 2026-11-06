//! Shared foundation for egui-prism components.
//!
//! Components and semantic themes are planned in subsequent quests. Applications
//! own their window, renderer, values, and resource lifecycles.
//!
//! The re-export keeps applications and the gallery on the same egui types:
//!
//! ```
//! use egui_prism::egui;
//!
//! let context = egui::Context::default();
//! context.request_repaint();
//! ```

/// The egui version used by this library.
pub use egui;
