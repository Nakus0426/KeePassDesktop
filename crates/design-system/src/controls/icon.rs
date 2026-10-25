//! Icon：包装 GPUI 的 `svg()` 元素。
//!
//! ## 上游提供什么
//!
//! `gpui::Svg`（`gpui-pre` 的 `src/elements/svg.rs`）已经实现了 SVG 的加载、
//! usvg 解析、按 `(path, size)` 缓存光栅结果、以及"取 alpha 通道再乘
//! `text_color`"的上色。我们只决定用哪个文件、多大、什么颜色。
//!
//! ## 为什么不自研、也不启用 `gpui-component`
//!
//! `gpui-base` 0.8.0 的 `pub use` 清单里没有 Icon（Q33 第 1 级落空）。
//! `gpui-component::Icon` 就是 `svg()` + `Sizable` + 主题前景色的薄封装，用它要
//! 打开 `component` feature、把整个 gpui-component 拉进依赖图，收益为零。
//! 因此这是 Q33 第 4 级（自研），范围限于"选文件 + 定尺寸 + 定颜色"。
//!
//! ## 使用时注意
//!
//! 颜色默认取 `DARK.text_primary`，与 [`crate::controls::button::Button`] 的默认
//! 调色板一致。放在别的色语境里必须显式 `.color(..)`：SVG 只认 `text_color`，
//! 不会跟随父容器的文字颜色自动变化。

use gpui::{App, IntoElement, RenderOnce, Styled, Window, px, rgb, svg};

use crate::icons::IconName;
use crate::tokens::{DARK, icon};

/// 图标尺寸档位。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconSize {
    /// 行内文字旁、按钮内。
    Small,
    /// 默认：工具条、列表行。
    Medium,
    /// 空状态、对话框标题。
    Large,
}

impl IconSize {
    const fn px(self) -> f32 {
        match self {
            Self::Small => icon::SM,
            Self::Medium => icon::MD,
            Self::Large => icon::LG,
        }
    }
}

/// 一个固定尺寸、单色的 Material Symbols 图标。
#[derive(IntoElement)]
pub struct Icon {
    name: IconName,
    size: IconSize,
    color: u32,
    filled: bool,
}

impl Icon {
    pub fn new(name: IconName) -> Self {
        Self {
            name,
            size: IconSize::Medium,
            color: DARK.text_primary,
            filled: false,
        }
    }

    pub fn small(mut self) -> Self {
        self.size = IconSize::Small;
        self
    }

    pub fn large(mut self) -> Self {
        self.size = IconSize::Large;
        self
    }

    /// 用填充态。Material 用它表达"已选中 / 已激活"，与常规态成对存在。
    pub fn filled(mut self) -> Self {
        self.filled = true;
        self
    }

    pub fn color(mut self, color: u32) -> Self {
        self.color = color;
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let side = self.size.px();
        svg()
            .path(self.name.path(self.filled))
            .flex_shrink_0()
            .size(px(side))
            .text_color(rgb(self.color))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::primitive;

    #[test]
    fn icon_sizes_sit_on_the_four_px_grid() {
        for size in [IconSize::Small, IconSize::Medium, IconSize::Large] {
            assert_eq!(size.px() % 4.0, 0.0, "{size:?} 不在 4px 栅格上");
        }
    }

    #[test]
    fn icon_sizes_are_strictly_increasing() {
        assert!(IconSize::Small.px() < IconSize::Medium.px());
        assert!(IconSize::Medium.px() < IconSize::Large.px());
    }

    #[test]
    fn default_icon_matches_body_text_height() {
        // 20px 图标配 BODY_M 的 14px 字号：图标略大于字，视觉重量才持平。
        let body = crate::tokens::typography::BODY_M;
        assert!(IconSize::Medium.px() > body.cjk.size);
        assert!(IconSize::Medium.px() < body.line_height());
    }

    #[test]
    fn small_icon_equals_the_smallest_spacing_unit() {
        assert_eq!(IconSize::Small.px(), primitive::SPACE_4);
    }

    #[test]
    fn new_icon_defaults_to_primary_text_and_regular_fill() {
        let icon = Icon::new(IconName::Lock);
        assert_eq!(icon.color, DARK.text_primary);
        assert!(!icon.filled);
        assert_eq!(icon.size, IconSize::Medium);
    }
}
