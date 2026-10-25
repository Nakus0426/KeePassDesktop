# Material Symbols 图标抓取脚本。
#
# 为什么 pin commit 而不是跟 master：图标是进二进制的资产，master 改名或改
# 路径会让下一次抓取悄悄换掉图形，而 git diff 看不出来（SVG 全是数字串）。
# 换版本时同时改这里的 $Commit 和 crates/design-system/assets/icons/NOTICE.md。
#
# 用法（仓库根目录）：pwsh tools/icons/fetch.ps1

$ErrorActionPreference = 'Stop'

$Commit = '10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7'
$BaseUrl = "https://raw.githubusercontent.com/google/material-design-icons/$Commit"
$Style = 'materialsymbolssharp'
$Dest = Join-Path $PSScriptRoot '..\..\crates\design-system\assets\icons\sharp'

# 只放当前界面真正用到的图标。加图标 = 在这里加名字 + 在 icons.rs 加一个分支。
$Names = @(
    'lock'
    'key'
    'visibility'
    'visibility_off'
    'content_copy'
    'search'
    'add'
    'settings'
    'folder'
    'attach_file'
    'refresh'
    'check'
)

New-Item -ItemType Directory -Force -Path $Dest | Out-Null

$written = 0
foreach ($name in $Names) {
    foreach ($variant in @(@{ Suffix = ''; File = "$name.svg" }, @{ Suffix = '_fill1'; File = "$name-fill.svg" })) {
        $url = "$BaseUrl/symbols/web/$name/$Style/$name$($variant.Suffix)_24px.svg"
        $svg = (Invoke-WebRequest -Uri $url -TimeoutSec 30).Content
        if (-not $svg.StartsWith('<svg')) {
            throw "not an SVG document: $url"
        }
        # 统一去掉 width/height，只留 viewBox：尺寸由 GPUI 的 svg().size() 决定，
        # 保留固定宽高会让 resvg 在非整数缩放下多一层取整。
        $svg = [regex]::Replace($svg, '\s(?:width|height)="[^"]*"', '')
        if ($svg -notmatch 'viewBox="0 -960 960 960"') {
            throw "unexpected viewBox (Material Symbols uses 0 -960 960 960): $url"
        }
        Set-Content -Path (Join-Path $Dest $variant.File) -Value $svg -NoNewline -Encoding utf8
        $written++
    }
}

# Apache-2.0 §4 要求随分发附带许可证全文。
$license = (Invoke-WebRequest -Uri "$BaseUrl/LICENSE" -TimeoutSec 30).Content
Set-Content -Path (Join-Path $Dest '..\LICENSE-APACHE-2.0.txt') -Value $license -NoNewline -Encoding utf8

Write-Output "wrote $written SVG files to $Dest (commit $Commit)"
