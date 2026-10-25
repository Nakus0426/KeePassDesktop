//! Design-system gallery.
//!
//! Every control rendered against the real tokens, in every state, so the visual
//! result can be judged by eye. Not a product screen.
//!
//! It also doubles as the IME harness: the live field in the middle is the one to
//! type Chinese into with a real IME, because that interaction cannot be
//! automated (see docs/architecture.md §17).
//!
//! ## Component sourcing
//!
//! Per AGENTS.md Q33, behaviour comes from `gpui_base` and only presentation is
//! ours. The text field wraps `gpui_base`'s input engine rather than
//! reimplementing it.

use design_system::prelude::*;
use design_system::tokens::{DARK, radius, space, typography};

pub struct Gallery {
    live_text: Entity<InputState>,
    live_password: Entity<InputState>,
    static_empty: Entity<InputState>,
    static_filled: Entity<InputState>,
    static_password: Entity<InputState>,
    static_focused: Entity<InputState>,
    static_invalid: Entity<InputState>,
    static_disabled: Entity<InputState>,
}

impl Gallery {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Every state is built once here. Creating an `Entity` inside `render`
        // would allocate a fresh one per frame and discard the caret.
        let live_text =
            cx.new(|cx| text_input(window, cx).placeholder("在这里输入中文，测试输入法"));
        let live_password =
            cx.new(|cx| text_input(window, cx).placeholder("密码字段").masked(true));
        let static_empty = cx.new(|cx| text_input(window, cx).placeholder("请输入名称"));
        let static_filled = cx.new(|cx| text_input(window, cx));
        let static_password = cx.new(|cx| text_input(window, cx).masked(true));
        let static_focused = cx.new(|cx| text_input(window, cx));
        let static_invalid = cx.new(|cx| text_input(window, cx));
        let static_disabled = cx.new(|cx| text_input(window, cx));

        static_filled.update(cx, |s, cx| s.set_value("已填写的值", window, cx));
        static_password.update(cx, |s, cx| s.set_value("hunter2", window, cx));
        static_focused.update(cx, |s, cx| s.set_value("聚焦态", window, cx));
        static_invalid.update(cx, |s, cx| s.set_value("校验失败", window, cx));
        static_disabled.update(cx, |s, cx| s.set_value("禁用", window, cx));

        Self {
            live_text,
            live_password,
            static_empty,
            static_filled,
            static_password,
            static_focused,
            static_invalid,
            static_disabled,
        }
    }

    /// Returns a concrete `Div`: callers chain `.child()` on it, which an opaque
    /// return type would hide.
    fn section(&self, title: &'static str) -> Div {
        let token = typography::control::LABEL;
        div().flex().flex_col().gap(px(space::STACK)).child(
            div()
                .text_size(px(token.line_height()))
                .text_color(rgb(DARK.text_secondary))
                .child(title),
        )
    }

    fn row_of(&self, children: Vec<AnyElement>) -> Div {
        let mut row = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(space::INLINE));
        for child in children {
            row = row.child(child);
        }
        row
    }
}

impl Render for Gallery {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let title_token = typography::TITLE_M;

        // The gallery is taller than the window. `id` is required: GPUI only
        // keeps scroll state for an element with a stable identity. `h_full`
        // (not `size_full`) so the height is bounded while content exceeds it.
        div()
            .id("gallery-scroll")
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .overflow_y_scroll()
            .bg(rgb(DARK.surface.page))
            .p(px(space::PAGE))
            .gap(px(space::SECTION))
            .child(
                div()
                    .text_size(px(title_token.line_height()))
                    .text_color(rgb(DARK.text_primary))
                    .child("设计系统"),
            )
            .child(
                self.section("按钮")
                    .child(self.row_of(vec![
                        Button::new("g-primary", "主要操作")
                            .primary()
                            .into_any_element(),
                        Button::new("g-secondary", "次要操作").into_any_element(),
                        Button::new("g-ghost", "轻量").ghost().into_any_element(),
                        Button::new("g-danger", "删除").danger().into_any_element(),
                    ]))
                    .child(self.row_of(vec![
                        Button::new("g-hover", "悬停")
                            .from_inputs(StateInput {
                                hovered: true,
                                ..Default::default()
                            })
                            .into_any_element(),
                        Button::new("g-pressed", "按下")
                            .from_inputs(StateInput {
                                pressed: true,
                                ..Default::default()
                            })
                            .into_any_element(),
                        Button::new("g-disabled", "禁用")
                            .from_inputs(StateInput {
                                disabled: true,
                                ..Default::default()
                            })
                            .into_any_element(),
                        Button::new("g-small", "小号").small().into_any_element(),
                        Button::new("g-large", "大号").large().into_any_element(),
                    ])),
            )
            .child(
                self.section("输入框（静态状态）")
                    .child(self.row_of(vec![
                        TextField::new("g-text", &self.static_empty, DARK).into_any_element(),
                        TextField::new("g-filled", &self.static_filled, DARK).into_any_element(),
                        TextField::new("g-password", &self.static_password, DARK)
                            .into_any_element(),
                    ]))
                    .child(self.row_of(vec![
                        TextField::new("g-focused", &self.static_focused, DARK)
                            .from_inputs(StateInput {
                                focused: true,
                                ..Default::default()
                            })
                            .into_any_element(),
                        TextField::new("g-invalid", &self.static_invalid, DARK)
                            .invalid(true)
                            .helper("主密码至少 12 位")
                            .into_any_element(),
                        TextField::new("g-disabled-field", &self.static_disabled, DARK)
                            .from_inputs(StateInput {
                                disabled: true,
                                ..Default::default()
                            })
                            .into_any_element(),
                    ])),
            )
            .child(
                self.section("输入框（可编辑，IME 用这一组）").child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(space::INLINE))
                        .w(px(520.0))
                        .child(
                            TextField::new("g-live-text", &self.live_text, DARK)
                                .width(520.0)
                                .helper("候选框位置、Esc 取消、上屏后复制"),
                        )
                        .child(
                            TextField::new("g-live-password", &self.live_password, DARK)
                                .width(520.0)
                                .helper("掩码由上游引擎处理"),
                        ),
                ),
            )
            .child(
                self.section("列表行").child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(space::INLINE))
                        .w(px(420.0))
                        .child(
                            ListRow::new("g-row-1", "GitHub")
                                .subtitle("alice@example.com")
                                .trailing("TOTP"),
                        )
                        .child(
                            ListRow::new("g-row-2", "银行账户")
                                .subtitle("6222 **** 1234")
                                .selected(true),
                        )
                        .child(
                            ListRow::new("g-row-3", "服务器")
                                .subtitle("root@10.0.0.1")
                                .from_inputs(StateInput {
                                    hovered: true,
                                    ..Default::default()
                                }),
                        ),
                ),
            )
            .child(
                self.section("对话框").child(
                    // Kept closed: a modal covers the whole gallery and swallows
                    // every click, so the preview shows the trigger instead. The
                    // upstream host's open/close lifecycle is exercised by the
                    // real screens once they exist.
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(space::INLINE))
                        .child(
                            div()
                                .text_size(px(typography::CAPTION.line_height()))
                                .text_color(rgb(DARK.text_secondary))
                                .child("对话框默认关闭；样式见 Dialog 的 backdrop + panel"),
                        )
                        .child(
                            Dialog::new("g-dialog", "删除这个条目？")
                                .body("条目将被移入回收站，可以还原。")
                                .width(420.0),
                        ),
                ),
            )
            .child(
                self.section("图标（Material Symbols sharp）")
                    .child(self.icon_row(IconSize::Medium))
                    .child(self.icon_row(IconSize::Small))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(space::STACK))
                            .child(Icon::new(IconName::Lock))
                            .child(
                                div()
                                    .text_size(px(typography::BODY_M.line_height()))
                                    .text_color(rgb(DARK.text_primary))
                                    .child("图标与正文同行时基线看起来是否齐"),
                            )
                            .child(Icon::new(IconName::Visibility).color(DARK.text_secondary))
                            .child(Icon::new(IconName::Check).color(DARK.accent))
                            .child(Icon::new(IconName::Add).color(DARK.danger)),
                    ),
            )
            .child(
                self.section("色板").child(
                    div()
                        .flex()
                        .flex_row()
                        .gap(px(space::STACK))
                        .child(self.swatch("页面", DARK.surface.page))
                        .child(self.swatch("卡片", DARK.surface.card))
                        .child(self.swatch("抬起", DARK.surface.raised))
                        .child(self.swatch("强调", DARK.accent))
                        .child(self.swatch("危险", DARK.danger)),
                ),
            )
    }
}

impl Gallery {
    /// One row per size: regular above, filled below, so the pair can be compared.
    fn icon_row(&self, size: IconSize) -> Div {
        let mut row = div().flex().flex_row().items_center().gap(px(space::STACK));
        for name in IconName::ALL {
            row = row
                .child(Self::sized(Icon::new(*name), size))
                .child(Self::sized(Icon::new(*name).filled(), size));
        }
        row
    }

    fn sized(icon: Icon, size: IconSize) -> Icon {
        match size {
            IconSize::Small => icon.small(),
            IconSize::Medium => icon,
            IconSize::Large => icon.large(),
        }
    }

    fn swatch(&self, label: &'static str, color: u32) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(px(space::INLINE))
            .child(
                div()
                    .w(px(64.0))
                    .h(px(40.0))
                    .rounded(px(radius::SURFACE))
                    .bg(rgb(color))
                    .border_1()
                    .border_color(rgb(DARK.surface.raised)),
            )
            .child(
                div()
                    .text_size(px(typography::CAPTION.line_height()))
                    .text_color(rgb(DARK.text_secondary))
                    .child(label),
            )
    }
}
