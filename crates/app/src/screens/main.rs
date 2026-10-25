//! Main screen: three columns (group sidebar, entry list, detail).
//!
//! The reference client uses a bottom tab bar plus a bottom sheet for detail,
//! which is a phone shape. A desktop window gets the three-column layout.

use design_system::prelude::*;
use design_system::tokens::{DARK, typography};

pub struct MainScreen {
    pub selected_group: Option<String>,
    pub selected_entry: Option<String>,
}

impl MainScreen {
    pub fn new() -> Self {
        Self {
            selected_group: None,
            selected_entry: None,
        }
    }
}

impl Default for MainScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for MainScreen {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            .bg(rgb(DARK.surface.page))
            .child(self.sidebar())
            .child(self.list_column())
            .child(self.detail_column())
    }
}

impl MainScreen {
    fn sidebar(&self) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w(px(240.0))
            .h_full()
            .p(px(12.0))
            .gap(px(4.0))
            .bg(rgb(DARK.surface.page))
            .child(self.sidebar_item("全部条目", true))
            .child(self.sidebar_item("收藏", false))
            .child(self.sidebar_item("最近使用", false))
    }

    fn sidebar_item(&self, label: &'static str, selected: bool) -> impl IntoElement {
        div()
            .h(px(36.0))
            .px(px(12.0))
            .flex()
            .items_center()
            .rounded(px(8.0))
            .bg(rgb(if selected {
                DARK.surface.card
            } else {
                DARK.surface.page
            }))
            .text_size(px(typography::BODY_M.cjk.size))
            .text_color(rgb(if selected {
                DARK.text_primary
            } else {
                DARK.text_secondary
            }))
            .child(label)
    }

    fn list_column(&self) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .flex_1()
            .h_full()
            .bg(rgb(DARK.surface.card))
            .p(px(12.0))
            .child(
                div()
                    .text_size(px(typography::CAPTION.cjk.size))
                    .text_color(rgb(DARK.text_secondary))
                    .child("尚无条目"),
            )
    }

    fn detail_column(&self) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w(px(360.0))
            .h_full()
            .bg(rgb(DARK.surface.raised))
            .p(px(16.0))
            .child(
                div()
                    .text_size(px(typography::CAPTION.cjk.size))
                    .text_color(rgb(DARK.text_secondary))
                    .child("选择一个条目"),
            )
    }
}
