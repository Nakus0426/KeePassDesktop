//! Unlock screen.
//!
//! The master password field is a real editable input (backed by `gpui_base`'s
//! engine), so typing a Chinese passphrase with a real IME exercises the path
//! that matters. `Vault` is still a trait with no implementation, so the screen
//! says so instead of pretending to unlock.

use design_system::prelude::*;
use design_system::tokens::{DARK, space, typography};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnlockIntent {
    /// The user asked to unlock. Wired once `keepass-core` is bound to a file.
    Submit,
}

pub struct UnlockScreen {
    password: Entity<InputState>,
    use_key_file: bool,
}

impl UnlockScreen {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let password = cx.new(|cx| {
            let state = text_input(window, cx);
            state.placeholder("主密码").masked(true)
        });

        Self {
            password,
            use_key_file: false,
        }
    }

    pub fn password_state(&self) -> &Entity<InputState> {
        &self.password
    }
}

impl Render for UnlockScreen {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let title = typography::TITLE_M;
        let body = typography::BODY_M;
        let caption = typography::CAPTION;

        let focus = self.password.read(cx).focus_handle(cx).clone();

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
                    .child("解锁数据库"),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(space::STACK))
                    .w(px(360.0))
                    .child(
                        TextField::new("unlock-password", &self.password, DARK)
                            .width(360.0)
                            .helper("解锁后 5 分钟无操作自动锁定"),
                    )
                    .child(
                        div()
                            .text_size(px(caption.line_height()))
                            .text_color(rgb(DARK.text_secondary))
                            .child(if self.use_key_file {
                                "密码 + 密钥文件"
                            } else {
                                "仅密码"
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(space::CONTROL_X))
                            .child(Button::new("unlock-submit", "解锁").primary().on_click(
                                move |_, window, cx| {
                                    // No vault backend yet: move the caret into
                                    // the field rather than doing nothing.
                                    window.focus(&focus, cx);
                                },
                            ))
                            .child(
                                Button::new("unlock-keyfile", "使用密钥文件")
                                    .ghost()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.use_key_file = !this.use_key_file;
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            .child(
                div()
                    .text_size(px(body.line_height()))
                    .text_color(rgb(DARK.text_disabled))
                    .child("尚未接入库文件，解锁按钮暂不可用"),
            )
    }
}

impl EventEmitter<UnlockIntent> for UnlockScreen {}
