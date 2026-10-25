# Material Symbols 资产来源

| 项 | 值 |
|---|---|
| 上游 | https://github.com/google/material-design-icons |
| commit | `10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7` |
| 样式 | `symbols/web/<name>/materialsymbolssharp/`（sharp, wght400, 24px） |
| 许可 | Apache-2.0，全文见同级 `LICENSE-APACHE-2.0.txt` |
| 生成 | `pwsh tools/icons/fetch.ps1` |

`sharp/*.svg` 是脚本产物，不要手工编辑。改动方式：编辑脚本里的图标清单，
重跑脚本，再更新 `crates/design-system/src/icons.rs`。

`fetch.ps1` 落盘时去掉了根节点的 `width` / `height`，只保留
`viewBox="0 -960 960 960"`：尺寸由 `Icon` 的 token 决定，固定宽高会让 resvg
在非整数缩放下多一层取整。
