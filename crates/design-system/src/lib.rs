//! Design tokens and atomic controls for KeePassDesktop.
//!
//! ## Why this crate exists
//!
//! It is the **only** crate that depends on `gpui-kit`. Everything else depends
//! on this crate's [`prelude`]. That keeps the GPUI base backend (currently
//! `gpui-pre`, see docs/architecture.md Q16) an implementation detail.
//!
//! The `extern crate` alias below is mandatory, not stylistic: `gpui-pre-macros`
//! expands to bare `gpui::` paths, so a crate-root alias named exactly `gpui` is
//! what resolves them. Never `use gpui_pre::…` or `use gpui_fast::…` directly.
//!
//! ## Token layers
//!
//! `primitive` (raw numbers) -> `semantic` (roles) -> `component` (controls).
//! Component code must never contain a raw colour or size literal; it references
//! `semantic` only. Spacing is owned here, not by `gpui-kit`: their
//! `Theme::apply_semantic_tokens` persists colours and radii but not a custom
//! spacing scale (gpui-kit docs), so we ship our own.
//!
//! Type tokens carry **two metrics sets** (CJK and Latin) because a 14px CJK
//! glyph and a 14px Latin glyph need different line heights and tracking.

extern crate gpui_kit as gpui;

pub mod controls;
pub mod icons;
pub mod prelude;
pub mod tokens;
