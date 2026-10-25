//! # app
//!
//! Assembly layer: the session state machine, navigation, and screens.
//!
//! This is the only crate that depends on both halves of the product. Screens
//! hold `vault_model` types and render with `design_system`; they never name
//! `gpui_kit` or `keepass_core` internals. The GPUI entry point arrives through
//! `design_system::prelude::application`, so `gpui-kit` is not a direct
//! dependency here at all — see docs/architecture.md Q27.

pub mod screens;
pub mod state;

pub use state::{AppState, Overlay, Route, Theme};
