//! Material Symbols 图标资源目录。
//!
//! ## 为什么是 SVG，不是图标字体
//!
//! `gpui-pre` 0.3.9 只支持 `FontFeatures`（4 字符 OpenType tag），**没有**
//! `font-variation-settings`。Material Symbols 的 FILL / GRAD / opsz / wght 四个轴
//! 一个都设不了，字体路线只能固定一个静态实例；变量字体本体还有 10 MB。SVG 走
//! `Svg::path`，GPUI 光栅时只取 alpha 通道、再用 `text_color` 上色，所以浅色与
//! 深色主题共用同一份资源。
//!
//! ## 变体约定
//!
//! 只用 sharp / wght400 / 24px。`fill1` 是 Material 表达"已选中"的官方做法，与
//! 常规态成对提供。字重与尺寸不预置：`viewBox` 固定为 `0 -960 960 960`，缩放到
//! 任何尺寸都由渲染器完成。
//!
//! 上游对纯几何图标不给 fill1 差异：`add` / `check` / `search` / `attach_file` /
//! `refresh` 的 `_fill1_` 与常规态字节相同，`filled()` 对它们是视觉空操作。这是
//! 上游事实，不是抓取错误；`filled_variants_known_identical` 断言现状，改上游后
//! 这个断言会红，提醒重新确认。
//!
//! ## 资源来源
//!
//! 由 `tools/icons/fetch.ps1` 按固定 commit 抓取，**不要手工编辑**这些 SVG。
//! 加一个图标 = 脚本里加名字 + 本文件加一条 `match` 分支 + `BUNDLED` 加一行；
//! `include_bytes!` 保证漏文件时编译期就失败。

use std::borrow::Cow;

/// 一个 Material Symbols 图标。填充态由 [`IconName::path`] 的 `filled` 参数选择。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IconName {
    Lock,
    Key,
    Visibility,
    VisibilityOff,
    ContentCopy,
    Search,
    Add,
    Settings,
    Folder,
    AttachFile,
    Refresh,
    Check,
}

impl IconName {
    /// 每个图标都有的两份资源路径：常规态与填充态。
    const fn files(self) -> (&'static str, &'static str) {
        match self {
            Self::Lock => ("icons/sharp/lock.svg", "icons/sharp/lock-fill.svg"),
            Self::Key => ("icons/sharp/key.svg", "icons/sharp/key-fill.svg"),
            Self::Visibility => (
                "icons/sharp/visibility.svg",
                "icons/sharp/visibility-fill.svg",
            ),
            Self::VisibilityOff => (
                "icons/sharp/visibility_off.svg",
                "icons/sharp/visibility_off-fill.svg",
            ),
            Self::ContentCopy => (
                "icons/sharp/content_copy.svg",
                "icons/sharp/content_copy-fill.svg",
            ),
            Self::Search => ("icons/sharp/search.svg", "icons/sharp/search-fill.svg"),
            Self::Add => ("icons/sharp/add.svg", "icons/sharp/add-fill.svg"),
            Self::Settings => ("icons/sharp/settings.svg", "icons/sharp/settings-fill.svg"),
            Self::Folder => ("icons/sharp/folder.svg", "icons/sharp/folder-fill.svg"),
            Self::AttachFile => (
                "icons/sharp/attach_file.svg",
                "icons/sharp/attach_file-fill.svg",
            ),
            Self::Refresh => ("icons/sharp/refresh.svg", "icons/sharp/refresh-fill.svg"),
            Self::Check => ("icons/sharp/check.svg", "icons/sharp/check-fill.svg"),
        }
    }

    /// 资产源里的键。用 `new_static` 而非 `From<&str>`：长路径超过 `SmolStr` 的
    /// 22 字节内联上限，后者每帧都会堆分配一次。
    pub fn path(self, filled: bool) -> gpui::SharedString {
        let (regular, filled_path) = self.files();
        gpui::SharedString::new_static(if filled { filled_path } else { regular })
    }

    /// 全部图标，供测试与资源完整性检查遍历。
    pub const ALL: &'static [Self] = &[
        Self::Lock,
        Self::Key,
        Self::Visibility,
        Self::VisibilityOff,
        Self::ContentCopy,
        Self::Search,
        Self::Add,
        Self::Settings,
        Self::Folder,
        Self::AttachFile,
        Self::Refresh,
        Self::Check,
    ];
}

/// `application().with_assets(IconAssets)` 注册的资产源。
///
/// 字节以 `include_bytes!` 静态内联，`load` 返回借用而不是拷贝：`Svg::path` 的
/// 光栅结果按 `(path, size)` 缓存在 sprite atlas 里，只有首次渲染会走到这里。
pub struct IconAssets;

/// 资源键到字节的映射。与 [`IconName::files`] 的路径必须一一对应，
/// `every_path_is_bundled` 测试守着这条。
const BUNDLED: &[(&str, &[u8])] = &[
    (
        "icons/sharp/lock.svg",
        include_bytes!("../assets/icons/sharp/lock.svg"),
    ),
    (
        "icons/sharp/lock-fill.svg",
        include_bytes!("../assets/icons/sharp/lock-fill.svg"),
    ),
    (
        "icons/sharp/key.svg",
        include_bytes!("../assets/icons/sharp/key.svg"),
    ),
    (
        "icons/sharp/key-fill.svg",
        include_bytes!("../assets/icons/sharp/key-fill.svg"),
    ),
    (
        "icons/sharp/visibility.svg",
        include_bytes!("../assets/icons/sharp/visibility.svg"),
    ),
    (
        "icons/sharp/visibility-fill.svg",
        include_bytes!("../assets/icons/sharp/visibility-fill.svg"),
    ),
    (
        "icons/sharp/visibility_off.svg",
        include_bytes!("../assets/icons/sharp/visibility_off.svg"),
    ),
    (
        "icons/sharp/visibility_off-fill.svg",
        include_bytes!("../assets/icons/sharp/visibility_off-fill.svg"),
    ),
    (
        "icons/sharp/content_copy.svg",
        include_bytes!("../assets/icons/sharp/content_copy.svg"),
    ),
    (
        "icons/sharp/content_copy-fill.svg",
        include_bytes!("../assets/icons/sharp/content_copy-fill.svg"),
    ),
    (
        "icons/sharp/search.svg",
        include_bytes!("../assets/icons/sharp/search.svg"),
    ),
    (
        "icons/sharp/search-fill.svg",
        include_bytes!("../assets/icons/sharp/search-fill.svg"),
    ),
    (
        "icons/sharp/add.svg",
        include_bytes!("../assets/icons/sharp/add.svg"),
    ),
    (
        "icons/sharp/add-fill.svg",
        include_bytes!("../assets/icons/sharp/add-fill.svg"),
    ),
    (
        "icons/sharp/settings.svg",
        include_bytes!("../assets/icons/sharp/settings.svg"),
    ),
    (
        "icons/sharp/settings-fill.svg",
        include_bytes!("../assets/icons/sharp/settings-fill.svg"),
    ),
    (
        "icons/sharp/folder.svg",
        include_bytes!("../assets/icons/sharp/folder.svg"),
    ),
    (
        "icons/sharp/folder-fill.svg",
        include_bytes!("../assets/icons/sharp/folder-fill.svg"),
    ),
    (
        "icons/sharp/attach_file.svg",
        include_bytes!("../assets/icons/sharp/attach_file.svg"),
    ),
    (
        "icons/sharp/attach_file-fill.svg",
        include_bytes!("../assets/icons/sharp/attach_file-fill.svg"),
    ),
    (
        "icons/sharp/refresh.svg",
        include_bytes!("../assets/icons/sharp/refresh.svg"),
    ),
    (
        "icons/sharp/refresh-fill.svg",
        include_bytes!("../assets/icons/sharp/refresh-fill.svg"),
    ),
    (
        "icons/sharp/check.svg",
        include_bytes!("../assets/icons/sharp/check.svg"),
    ),
    (
        "icons/sharp/check-fill.svg",
        include_bytes!("../assets/icons/sharp/check-fill.svg"),
    ),
];

impl gpui::AssetSource for IconAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        let hit = BUNDLED.iter().find(|(key, _)| *key == path);
        Ok(hit.map(|(_, bytes)| Cow::Borrowed(*bytes)))
    }

    fn list(&self, _path: &str) -> gpui::Result<Vec<gpui::SharedString>> {
        // 产品从不按目录列举资产（图标只经 IconName 引用），返回空表即可。
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::AssetSource;

    /// Material Symbols sharp 的固定坐标系。变了说明抓错文件（比如抓成别的图标集）。
    const VIEW_BOX: &str = "viewBox=\"0 -960 960 960\"";

    #[test]
    fn every_path_is_bundled_and_is_an_svg() {
        for name in IconName::ALL {
            for filled in [false, true] {
                let path = name.path(filled);
                let bytes = IconAssets
                    .load(&path)
                    .expect("asset lookup must not fail")
                    .unwrap_or_else(|| panic!("{path} 没有打包进资产源"));
                let text = std::str::from_utf8(&bytes).expect("SVG 必须是 UTF-8");
                assert!(
                    text.starts_with("<svg"),
                    "{path} 不是 SVG 文档，开头是 {:?}",
                    &text[..text.len().min(20)]
                );
                assert!(text.contains(VIEW_BOX), "{path} 的 viewBox 不符合约定");
            }
        }
    }

    #[test]
    fn paths_are_unique_per_variant() {
        let mut seen = std::collections::BTreeSet::new();
        for name in IconName::ALL {
            for filled in [false, true] {
                let path = name.path(filled);
                assert!(seen.insert(path.to_string()), "资源路径重复：{path}");
            }
        }
        assert_eq!(seen.len(), IconName::ALL.len() * 2);
    }

    #[test]
    fn filled_variant_uses_the_fill_suffix() {
        for name in IconName::ALL {
            assert!(
                name.path(true).ends_with("-fill.svg"),
                "{name:?} 的填充态路径命名不对"
            );
            assert!(!name.path(false).ends_with("-fill.svg"));
        }
    }

    /// 上游对纯几何图形不给 fill1 差异，`filled()` 对它们是视觉空操作。
    ///
    /// 这是上游事实，不是抓取漏文件。写成断言：哪天上游给这些图标补了填充态，
    /// 这里会红，提醒我们把它们从名单里拿掉。
    #[test]
    fn filled_variants_known_identical() {
        let identical: &[IconName] = &[
            IconName::Add,
            IconName::Check,
            IconName::Search,
            IconName::AttachFile,
            IconName::Refresh,
        ];

        for name in IconName::ALL {
            let regular = IconAssets.load(&name.path(false)).expect("lookup");
            let filled = IconAssets.load(&name.path(true)).expect("lookup");
            let same = regular == filled;
            if identical.contains(name) {
                assert!(same, "{name:?} 的 fill1 已不再等同常规态，把它从名单里移除");
            } else {
                assert!(!same, "{name:?} 的 fill1 与常规态相同，名单漏了它");
            }
        }
    }
}
