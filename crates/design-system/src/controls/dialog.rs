//! Dialog: a styled wrapper over `gpui_base`'s dialog host.
//!
//! ## What upstream provides
//!
//! `gpui_base::Dialog` is an *unstyled modal host* that owns the hard parts:
//! a focus trap, Escape and backdrop dismissal, `open` state with an
//! `on_open_change` callback, layering above other content, and callback
//! ordering. Its parts are separate unstyled slots (`DialogTitle`,
//! `DialogDescription`, `DialogClose`), which is exactly the seam a custom design
//! system needs.
//!
//! Per AGENTS.md Q33 our `Dialog` therefore only supplies the backdrop colour,
//! the panel surface, and token-based typography. Focus trapping is **not**
//! reimplemented here — the earlier hand-rolled panel had no trap at all, which
//! is the failure this wrapper fixes.

use gpui::{App, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px, rgb};

use crate::tokens::{DARK, Palette, radius, space, typography};

pub use gpui_kit::base::{Dialog as UpstreamDialog, DialogClose, DialogDescription, DialogTitle};

/// Narrow enough for a 1366-wide laptop, wide enough for a 20-char CJK body.
pub const MIN_WIDTH: f32 = 320.0;
pub const MAX_WIDTH: f32 = 520.0;

/// Clamp a requested width into the supported range.
pub const fn clamp_width(requested: f32) -> f32 {
    if requested < MIN_WIDTH {
        MIN_WIDTH
    } else if requested > MAX_WIDTH {
        MAX_WIDTH
    } else {
        requested
    }
}

/// Paint a token-styled modal around the upstream host.
#[derive(IntoElement)]
pub struct Dialog {
    id: &'static str,
    title: SharedString,
    body: Option<SharedString>,
    width: f32,
    palette: Palette,
    open: bool,
}

impl Dialog {
    pub fn new(id: &'static str, title: impl Into<SharedString>) -> Self {
        Self {
            id,
            title: title.into(),
            body: None,
            width: clamp_width(420.0),
            palette: DARK,
            // Closed by default: a modal that starts open covers the whole screen
            // and swallows every click. Callers open it explicitly.
            open: false,
        }
    }

    pub fn body(mut self, body: impl Into<SharedString>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = clamp_width(width);
        self
    }

    /// Whether the dialog is shown. Upstream owns the open/close lifecycle; this
    /// only forwards the current value for a static preview.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
}

impl RenderOnce for Dialog {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let title_token = typography::control::DIALOG_TITLE;
        let body_token = typography::control::BODY;

        // Backdrop: a dimming layer. The scrim token is the only one allowed to
        // carry alpha.
        let backdrop = div()
            .absolute()
            .inset_0()
            .bg(rgb(self.palette.scrim).opacity(0.55));

        let body = self.body.clone();
        let panel = div()
            .id(self.id)
            .w(px(self.width))
            .flex()
            .flex_col()
            .gap(px(space::STACK))
            .p(px(space::SECTION))
            .rounded(px(radius::CARD))
            .bg(rgb(self.palette.surface.raised))
            .child(
                DialogTitle::new()
                    .text_size(px(title_token.line_height()))
                    .text_color(rgb(self.palette.text_primary))
                    .child(self.title.clone()),
            )
            .child(
                DialogDescription::new()
                    .text_size(px(body_token.line_height()))
                    .text_color(rgb(self.palette.text_secondary))
                    .child(body.unwrap_or_else(|| "".into())),
            );

        UpstreamDialog::new(cx)
            .open(self.open)
            .backdrop(backdrop)
            .popup(panel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_is_clamped_both_ways() {
        assert_eq!(clamp_width(0.0), MIN_WIDTH);
        assert_eq!(clamp_width(100.0), MIN_WIDTH);
        assert_eq!(clamp_width(400.0), 400.0);
        assert_eq!(clamp_width(10_000.0), MAX_WIDTH);
    }

    #[test]
    fn default_width_is_inside_the_range() {
        let w = clamp_width(420.0);
        assert!((MIN_WIDTH..=MAX_WIDTH).contains(&w));
    }

    #[test]
    fn min_width_fits_a_twenty_char_cjk_body() {
        // At the body size, 20 CJK glyphs must fit one line, otherwise the body
        // wraps to one character per row on a narrow window.
        let glyph = typography::control::BODY.cjk.size;
        let needed = glyph * 20.0;
        assert!(
            MIN_WIDTH >= needed,
            "MIN_WIDTH {MIN_WIDTH} cannot hold {needed}px of CJK text"
        );
    }

    #[test]
    fn dialog_sits_above_page_and_card() {
        // The panel must be distinguishable from both, or it looks like content.
        assert_ne!(DARK.surface.raised, DARK.surface.page);
        assert_ne!(DARK.surface.raised, DARK.surface.card);
    }
}
