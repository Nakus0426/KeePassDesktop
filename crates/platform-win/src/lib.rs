//! Windows implementations of `platform_api`.
//!
//! v1 targets Windows only (docs/architecture.md Q2/Q6). Nothing outside this
//! crate may call Win32 directly.

pub mod clipboard;
pub mod paths;
pub mod session;
pub mod single_instance;
