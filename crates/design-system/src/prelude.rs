//! The single import surface for the rest of the product.
//!
//! Every crate except this one writes `use design_system::prelude::*;` and never
//! names `gpui_kit` itself. If the GPUI base backend is swapped, only this crate
//! changes.
//!
//! This deliberately does not follow the gpui-kit guide's "`use gpui_kit::*`
//! everywhere" advice: the product owns its design system, so the framework
//! surface is funnelled through one crate. See docs/architecture.md Q27.

pub use crate::controls::button::{Button, ButtonKind, ButtonSize};
pub use crate::controls::dialog::Dialog;
pub use crate::controls::icon::{Icon, IconSize};
pub use crate::controls::list_row::{ListRow, row_height};
pub use crate::controls::text_input::{Input, InputState, TextField, text_input};
pub use crate::controls::{
    ControlState, StateInput, derive_state, stroke_color, stroke_visible, surface_fill, text_color,
};
pub use crate::icons::{IconAssets, IconName};
pub use crate::tokens::{DARK, LIGHT, Palette, SurfaceTier, TypeToken, radius, space, typography};

// GPUI surface the product actually uses, under our own names.
pub use gpui::{
    AnyElement, App, Application, Bounds, ClickEvent, Context, Div, ElementId, Entity,
    EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement, Render, RenderOnce,
    SharedString, Styled, TitlebarOptions, Window, WindowBounds, WindowOptions, div, px, rgb, size,
};

pub use gpui::prelude::*;

/// The GPUI application entry point, with the icon assets registered.
///
/// Wrapped rather than re-exported for two reasons: the app crate must never
/// import `gpui_kit` itself (Q27), and `Svg::path` only resolves through an
/// `AssetSource` registered here. Skipping this makes every icon render blank
/// with no error, because `paint_svg` treats a missing asset as "draw nothing".
pub fn application() -> gpui::Application {
    gpui_kit::application().with_assets(crate::icons::IconAssets)
}

/// Must be called once, before anything else, in `application().run(..)`.
///
/// Re-exported rather than called here so the app crate never imports
/// `gpui_kit`.
pub fn init(cx: &mut App) {
    gpui_kit::init(cx);
}

/// Open an application window with the framework's `Root` mounted.
///
/// `Root` is what makes dialogs, sheets, notifications and tooltips work; a
/// plain `cx.open_window` does not mount it, so always use this.
pub fn open_window<V: Render>(
    options: WindowOptions,
    cx: &mut App,
    build: impl FnOnce(&mut Window, &mut App) -> Entity<V>,
) -> anyhow_like::Result<(gpui::AnyWindowHandle, Entity<V>)> {
    Ok(gpui_kit::open_window(options, cx, build)?)
}

/// Minimal local alias so this crate does not depend on `anyhow` just to name
/// the framework's error type in one signature.
mod anyhow_like {
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
}
