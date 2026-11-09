//! Shared foundation for egui-prism components.
//!
//! Themes are in [`theme`], buttons in [`button`], editors in [`text_input`], and checkboxes in [`checkbox`]. Applications
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

pub mod theme;

pub mod button;
pub mod text_input;

pub mod checkbox;

pub mod compositing;
pub mod slider;
