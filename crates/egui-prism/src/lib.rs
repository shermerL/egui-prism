//! Shared foundation for egui-prism components.
//!
//! Themes are in [`theme`], badges in [`badge`], buttons in [`button`], editors in [`text_input`],
//! checkboxes in [`checkbox`], selects in [`select`], and dialogs in [`dialog`].
//! Applications own their
//! window, renderer, values, and resource lifecycles.
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

mod shadow;
mod transition;

pub mod theme;

pub mod badge;
pub mod button;
pub mod card;
pub mod text_input;

pub mod checkbox;
pub mod switch;

pub mod compositing;
pub mod dialog;
pub mod select;
pub mod slider;
pub mod tooltip;
