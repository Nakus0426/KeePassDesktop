//! Welcome screen: create a vault or open an existing one.

use design_system::prelude::*;
use design_system::tokens::{DARK, space, typography};

/// What the user asked for. Emitted upward so the screen owns no routing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WelcomeIntent {
    OpenVault,
    CreateVault,
    /// Development-only entry to the design-system gallery.
    ShowGallery,
}

pub struct WelcomeScreen {
    /// Reserved for the key-file flow; the field exists so the screen shape is
    /// stable when that lands (v1 opens password-only vaults).
    #[allow(dead_code)]
    has_key_file: bool,
}

impl WelcomeScreen {
    pub fn new() -> Self {
        Self {
            has_key_file: false,
        }
    }
}

impl Default for WelcomeScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for WelcomeScreen {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body = typography::BODY_M;
        let title = typography::TITLE_M;

        div()
            .flex()
            .flex_col()
            .size_full()
            .items_center()
            .justify_center()
            .bg(rgb(DARK.surface.page))
            .gap(px(space::SECTION))
            .child(
                div()
                    .text_size(px(title.line_height()))
                    .text_color(rgb(DARK.text_primary))
                    .child("KeePass Desktop"),
            )
            .child(
                div()
                    .text_size(px(body.line_height()))
                    .text_color(rgb(DARK.text_secondary))
                    .child("新建数据库，或打开已有的 .kdbx 文件"),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(space::CONTROL_X))
                    .child(
                        Button::new("welcome-open", "打开数据库")
                            .primary()
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.emit(WelcomeIntent::OpenVault);
                            })),
                    )
                    .child(
                        Button::new("welcome-create", "新建数据库").on_click(cx.listener(
                            |_, _, _, cx| {
                                cx.emit(WelcomeIntent::CreateVault);
                            },
                        )),
                    ),
            )
            .child(
                Button::new("welcome-gallery", "设计系统预览")
                    .ghost()
                    .small()
                    .on_click(cx.listener(|_, _, _, cx| {
                        cx.emit(WelcomeIntent::ShowGallery);
                    })),
            )
    }
}

impl EventEmitter<WelcomeIntent> for WelcomeScreen {}
