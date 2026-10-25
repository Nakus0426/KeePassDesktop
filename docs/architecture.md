# KeePassDesktop 技术架构

> 状态：草案 v1　日期：2026-10-24　基线：`main@54b1ace`
> 参考实现：`E:\WorkSpace\KeePassHomo@d91b5f6`（鸿蒙端）
> 本文档中的"已验证"指在本机实际编译或运行过，其余为调研结论。

---

## 1. 已确认的决策

| 编号 | 决策 | 值 |
|---|---|---|
| Q1 | v1 功能线 | 读写核心：解锁、条目/分组 CRUD、密码生成器、TOTP、搜索、附件 |
| Q2/Q6 | 平台 | **仅 Windows**。macOS / Linux 不实现，但接口预留 |
| Q3 | 库模型 | 数据结构按多库设计，v1 只开放单库交互；同步交给外部工具 |
| Q4/Q20/Q21 | 仓库结构 | cargo workspace，6 个 crate |
| Q5 | i18n | 编译期字串表，只发 `zh-CN` |
| Q7/Q24 | 工具链 | rustup + stable-msvc，装在 `E:\Tools` |
| Q8 | 安全底线 | 全部接受（zeroize、30s 剪贴板、5min 自动锁、禁日志明文、禁 core dump） |
| Q9 | 设计 | 无设计稿，先做 token 层 + 原子控件验证页 |
| Q10 | 参考边界 | 只参考功能意图与交互流程，实现全部重做 |
| Q11/Q22/Q23 | 互认 | 读写鸿蒙端私有 key，**单向兼容**（Q23=A）；CJK/Latin 双度量建模 |
| Q12 | TOTP 格式 | 只写 `otpauth://` 到 `otp` 字段；老格式只读 |
| Q13 | 调研报告 | 保留为架构输入记录 |
| Q14 | UI 基座 | `gpui-base` 为主，`gpui-component` 按需引入（feature 隔离） |
| Q15 | 无障碍 | 第一周 spike，结论出来后决定是否换栈 |
| Q16 | GPUI 后端 | **`gpui-pre` 0.3.9**，不启用 `gpui-fast` |
| Q17 | KDBX | `keepass` 0.17.0，包在自有 trait 后 |
| Q18 | 链 | stable-msvc（无 nightly） |
| Q19 | 测试向量 | `keepass-core` 第一批代码 |
| Q25 | CI | GitHub Actions，只跑 `keepass-core` + clippy |
| Q26 | `cx.notify()` 验证 | headless 断言覆盖解锁/显示密码/自动锁 |
| Q27 | 宏路径 | `design-system` 封装 `prelude`，其它 crate 不接触 `gpui_kit` |
| Q28 | 私有 key | 全部四个都写（`Favorite`/`Archived`/`FieldTypes`/`KPEX_LayoutType`） |
| Q29 | 防覆盖 | 保存前哈希比对，不一致拒绝写入 |
| Q30 | 再认证 | 多操作覆盖 + 头部 HMAC 校验 + 限速 |
| Q31 | 模板 | 完全对齐 6 个模板，含 `SshKeys` JSON 结构 |
| Q32 | 产品定位 | **面向普通用户的 GUI 桌面应用**。默认 GUI subsystem，不出现命令行；全部功能走界面（§7.1） |

### 1.1 本机已实测

| 项 | 结果 |
|---|---|
| rustc / cargo | 1.99.1（`E:\Tools\rustup` / `E:\Tools\cargo`） |
| MSVC 工具链 | VS BuildTools 2022 17.14.42，`E:\Tools\VS`，MSVC 14.44.35207 |
| Windows SDK | 10.0.26100.0 |
| `rustc` 链接 | 通过 |
| `gpui-base` 0.8.0 编译 | 通过，1m21s，465 个包 |
| `gpui-kit` 0.8.0 编译 | 通过，1m27s |
| GPUI 窗口启动 | **通过**。515×537 窗口，灰底白字渲染 "你好, KeePass!"，CJK 无需额外字体包 |
| `keepass` 0.17.0 + `save_kdbx4` + `totp` | 通过，19s |

### 1.2 网络约束（重要）

`index.crates.io` 与 `static.crates.io` 在本机**连接超时**（PowerShell `Invoke-WebRequest` 可通，`curl` 与 `cargo` 均超时，IPv4/IPv6 都一样）。

解决：`E:\Tools\cargo\config.toml` 使用 rsproxy.cn 镜像。

```toml
[source.crates-io]
replace-with = "rsproxy-sparse"

[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"

[net]
retry = 5
git-fetch-with-cli = true

[build]
target-dir = "E:\\Tools\\cargo\\target"
```

配套：`C:\Users\Administrator\.cargo` 与 `.rustup` 是指向 `E:\Tools` 的目录联接，使工具链不依赖环境变量。

**风险**：CI 上不能用 rsproxy（GitHub Actions 直连 crates.io 正常），所以镜像配置必须留在本机 `CARGO_HOME`，**不能提交到仓库**。

---

## 2. 工作区结构

```
crates/
  keepass-core/     # KDBX 读写、密钥、磁盘事务、错误分类
  vault-model/      # 领域模型与用例，不依赖任何 UI 或存储细节
  design-system/    # design token + 原子控件 + gpui-kit 唯一入口
  platform-api/     # 平台抽象 trait：剪贴板、锁屏、单实例、托盘
  platform-win/     # Windows 实现
  app/              # 组装：状态机、路由、屏幕
```

### 2.1 依赖方向（单向，不可逆）

```
        app
       /   \
      v     v
vault-model  design-system
      |            |
      v            v
keepass-core   (gpui-kit)
      ^
      |
platform-api  <-- platform-win --> 由 app 注入
```

规则：

1. `keepass-core` **不依赖** `vault-model`、`design-system`、`gpui-kit`。它是叶子。
2. `vault-model` 只认 `keepass-core` 暴露的 trait，不认 `keepass` crate 本身的类型。
3. `design-system` 只依赖 `gpui-kit`，不依赖任何业务 crate。
4. `app` 是唯一同时依赖两侧的 crate。
5. `platform-api` 不依赖任何其它内部 crate。

CI 层面用 `cargo-deny` 或简单的 `cargo tree` 断言来强制执行第 1、3 条。

### 2.2 为什么不拆 `platform-mac` / `platform-linux`

Q21=A 已定：只建 `platform-win`。理由：空 crate 会让 CI 长期报错或需要 `--target` 排除，维护成本高于收益。届时新增平台是 `platform-<os>` crate + 在 `app` 里换构造，不改接口。

---

## 3. `keepass-core` 设计

### 3.1 trait 边界

```rust
/// 打开的库。持有解密后的状态，但**不**把明文暴露给 UI。
pub trait Vault {
    fn metadata(&self) -> &VaultMeta;
    fn groups(&self) -> &[GroupNode];
    fn entries(&self) -> &[EntrySummary];

    /// 受保护字段必须显式请求，且返回值在调用方 drop 时擦除。
    fn reveal(&self, entry: EntryId, field: FieldId) -> CoreResult<SecretString>;

    fn apply(&mut self, tx: VaultTransaction) -> CoreResult<ChangeSet>;
    fn rollback(&mut self, cs: ChangeSet) -> CoreResult<()>;
}
```

要点：

- **`EntrySummary` 不含明文**。参考实现的反面教材是 `mapEntry` 直接调 `ProtectedValue.getText()`，把明文拷进 `VaultDocument` 再交给 UI。我们不做。
- `reveal` 是唯一的明文出口，签名上就强制调用方声明意图。
- `apply` 收 `VaultTransaction`，返回 `ChangeSet` 用于撤销与脏标记。

### 3.2 磁盘事务（Q29 强制）

```
open:    读取文件 → sha256 + mtime + size → 记录为 OpenedIdentity
save:    重新 stat + sha256 当前文件
         ├─ 不匹配 → CoreError::FileChangedExternally，中止，不写
         └─ 匹配 → 写到 <name>.kdbx.tmp
                   → 用当前凭据完整 unlock tmp（验证可读）
                   → 原文件复制为 <name>.kdbx.bak
                   → rename tmp 覆盖原文件
                   → 再 unlock 一次验证
                   → 失败则用 bak 还原
                   → 删除 bak 与 tmp
```

参考实现 `FileVaultRepository.writeSafely` 已验证这套顺序有效，但它自己文档承认**不保证掉电原子性**。我们额外做两件事：

1. `FileChangedExternally` 检查（参考实现没有，这是数据丢失的主要来源）。
2. 写 tmp 前 `sync_all()`，rename 后再 `sync_all()` 父目录。

### 3.3 再认证（Q30）

需求：导出附件、导出 SSH 私钥、显示受保护字段、复制密码、改主密码、删除条目，都要二次确认。

- **校验方式**：只解密 KDBX 头部并验证 header HMAC，不做整库解密。参考实现用的是完整 `Kdbx.load`（含 KDF），开销过大。
- **待验证**：`keepass` 0.17.0 是否暴露"只校验凭据"的 API。若没有，我们在 `keepass-core` 里自己实现——KDBX 4 的头部结构（主 seed、transform seed、加密参数、header SHA-256 与 HMAC-SHA256）是公开格式，可行。**这一项列为 v1 的待办验证项。**
- **限速**：连续失败 3 次锁 30 秒；累计失败 10 次强制重新解锁整个库。
- 参考实现**完全没有**限速与尝试计数。

### 3.4 错误分类

```rust
pub enum CoreError {
    NotAKdbxFile,              // 签名不匹配
    UnsupportedVersion(String),
    Corrupt(&'static str),     // 结构损坏 / HMAC 失败
    BadCredential,             // 密码或密钥文件错误
    FileChangedExternally,     // Q29
    PermissionDenied(PathBuf),
    Io(std::io::Error),
}
```

UI 层按 variant 给不同的恢复动作，不显示裸错误串。日志只记 variant 与 code，不记路径以外的内容。

---

## 4. 互操作规范（Q23/Q28/Q31）

### 4.1 写入的私有标记（全部条目级 `customData`）

| 键 | 值 | 我们 |
|---|---|---|
| `KeePassHomo.Favorite` | `"true"` / `"false"` | 写 |
| `KeePassHomo.Archived` | `"true"` / `"false"` | 写 |
| `KeePassHomo.FieldTypes` | JSON `{"字段名":"url\|email\|phone\|address\|date\|password\|custom"}` | 写 |
| `KeePassHomo.Validation` | 时间戳 | **永不写**（鸿蒙端死代码） |
| `KPEX_LayoutType` | `Login`/`SecureNote`/`CreditCard`/`SshKey`/`Attachment`/`Password` | 写，仅创建时 |

`FieldTypes` 规则：只存非 `text` 的字段；映射为空时删除整个键。前缀常量 `KeePassHomo.`。

**注意**：数据库级与分组级 `customData` 鸿蒙端从不写，我们也不写。第三方 key（如 `KPXC_*`）必须原样保留，不清理。

### 4.2 私有字段

`SshKeys`——受保护字段，JSON：

```json
{"version":1,"keys":[{"name":"","algorithm":"","bits":0,"addedAt":0,"privateKey":"","publicKey":""}]}
```

空列表不写入。

### 4.3 模板字段名（必须逐字一致）

| 模板 | 字段 |
|---|---|
| `Login` | `UserName`, `Password`(protected), `URL`, `Notes`(multiline) |
| `SecureNote` | `Notes`(multiline) |
| `CreditCard` | `CardNumber`(protected), `Issuer`, `CardType`, `CardHolderName`, `Expiry` + `Notes` |
| `SshKey` | `SshKeys`(protected JSON) + `Notes` |
| `Attachment` | 无字段，走 KDBX binaries + `Notes` |
| `Password` | `Password`(protected) + `Notes` |

`CardType` 选项：银联 / Visa / Mastercard / American Express / JCB / Discover。

字段名一错，"互认"就断在数据层。这条是硬约束，写进 `keepass-core` 的测试。

### 4.4 自定义图标

完全标准 KDBX：字节写入 `meta.customIcons`（随机 UUID 键），条目用 `entry.customIcon` 引用，**`entry.icon` 强制为 69**（`CUSTOM_ICON_ID`），因为 KeePass 在 IconID < 69 时忽略 `customIcon`。字节相同则复用同一 UUID。

这条不是可选的：不照做，鸿蒙端与 KeePass 都读不到图标。

### 4.5 我们**不**照搬的三件事

1. **`writeFields` 的字段删除行为**。鸿蒙端保存时会删掉模型里没有的每个字段，且整份重写（last-writer-wins）。我们的保存路径必须保留它不认识的字段。
2. **SSH 私钥导出无门禁**。鸿蒙端 `SshKeyExport.exportSshPrivateKey` 直接写文件。我们强制再认证。
3. **搜索的简陋实现**。鸿蒙端是纯子串 + `toLocaleLowerCase`，无归一化、无拼音、无排序。我们做：Unicode 归一化 + 拼音首字母 + 字段权重排序 + TOTP/URL 特殊匹配。

---

## 5. `vault-model` 设计

### 5.1 领域类型

```rust
pub struct VaultRef { path: PathBuf, display_name: String, last_opened: SystemTime }

pub enum FieldKind { Text, Protected, Url, Email, Phone, Address, Date, Password, SshKeys }

pub struct EntrySummary {
    pub id: EntryId,
    pub title: String,          // 非敏感，可展示
    pub username: String,
    pub url: String,
    pub tags: Vec<String>,
    pub template: Option<Template>,
    pub favorite: bool,
    pub archived: bool,
    pub has_totp: bool,
    pub attachment_count: usize,
    pub group_path: String,
    // 无 password 字段。明文只能经 Vault::reveal 拿。
}
```

### 5.2 用例层

用例是纯函数或对 `Vault` trait 的操作，可在无 UI 情况下测试：

- `unlock`, `lock`, `reauthenticate`
- `list`, `search(Query) -> Vec<EntrySummary>`
- `create_entry(template)`, `update_entry`, `delete_entry`（进回收站）
- `create_group`, `rename_group`, `delete_group`
- `generate_password(RandomParams | MemorableParams | PinParams)`
- `totp_code(entry, now) -> SecretString`
- `save` / `discard`

### 5.3 会话与自动锁

```rust
pub struct Session {
    vault: Box<dyn Vault>,
    identity: OpenedIdentity,       // Q29 用
    opened_at: Instant,
    last_activity: Cell<Instant>,   // 由 UI 事件推进
}
```

自动锁 5 分钟（可配），且监听 Windows 会话锁屏事件——**锁屏必须立即锁定库**，不等待计时器。

### 5.4 密码生成器

三种模式，参数对齐鸿蒙端以保证用户习惯连续：

- Random：长度 8–20，四类字符开关，排除易混字符，最少数字/符号数
- Memorable：3–8 词，分隔符，大小写，含数字
- PIN：4–12 位，避免连号与重复

熵计算与强度评分统一用 `vault-model::strength`，算法参考鸿蒙端 `PasswordStrengthCalculator`（纯启发式，可直接移植）。

### 5.5 剪贴板（30 秒自动清除）

`platform-api::Clipboard` 提供 `set_secret(SecretString, ttl)`。实现要点：

- 记录写入序号；到期只有当前序号仍匹配才清除，避免清掉用户后复制的内容。
- 参考实现用鸿蒙 pasteboard 的 `changeCount` 做同样的事，思路正确，可移植。
- 应用退出时尽力清除。

---

## 6. `design-system`

### 6.1 唯一入口

`design-system` 是 `gpui-kit` 的**唯一**消费者。其它 crate 只 `use design_system::prelude::*`。

crate 根一行：

```rust
extern crate gpui_kit as gpui;
```

原因：`gpui-pre-macros` 展开的是裸 `gpui::` 路径（实测 193 处），必须靠 `extern crate ... as gpui` 解析。**不要**直接 import `gpui_pre::` 或 `gpui_fast::`。

### 6.2 Token 模型（Q22：CJK/Latin 双度量）

这是本设计系统与普通 web 设计系统的核心差异：中文字形与拉丁字形在相同字号下的视觉字重、行高、基线都不同。token 必须按两套度量建模。

```rust
pub struct TypeScale {
    pub cjk: Metrics,     // 用于含中文的文本
    pub latin: Metrics,   // 用于纯拉丁文本
}

pub struct Metrics {
    pub size: f32,        // px
    pub line_height: f32, // px，不是倍数——中文字面高度与拉丁 x-height 差异大
    pub letter_spacing: f32,
    pub weight: FontWeight,
}
```

Token 层级：

| 层 | 内容 |
|---|---|
| `primitive` | 原始色板、字号、间距、圆角、动效曲线的数值 |
| `semantic` | `surface.page` / `surface.card` / `text.primary` / `border.focus` / `state.danger` |
| `component` | 具体控件引用 semantic，禁止直接引用 primitive |

**禁用规则**：`component` 层不得出现字面数值，只能引用 `semantic`。这条用 clippy lint 或代码审查保证。

间距用 4px 基准（4/8/12/16/24/32/48），不采用 `gpui-kit` 的语义主题间距——其文档明确说 `Theme::apply_semantic_tokens` **不会**持久化自定义间距刻度，所以间距 token 由我们完全拥有。

### 6.3 无边框分层（移植鸿蒙端已验证的规则）

`docs/arkui-borderless-ui.md` 六条规则的核心，直接映射到桌面：

1. **静息态无描边**。卡片、输入框容器、头像一律不画 border。
2. **层级靠背景明度**，三级固定：`surface.page` / `surface.card` / `surface.card-raised`。容器与页面不能用同一级。
3. **状态描边保留，只变色**：静息透明、聚焦 `border.focus`、校验失败 `border.danger`。宽度恒定，避免布局跳动。
4. **分隔线与卡片描边不受此规则约束**。
5. 卡片自身扁平无阴影；阴影是短白名单，只用于浮层。
6. 圆角固定梯度，不做 50% 以外的特例（头像例外）。

### 6.4 原子控件清单（v1 spike 范围）

先做 4 个，其余在 spike 通过后再铺。**实现方式见 §18**：三个包装 `gpui_base`，一个自绘。

| 控件 | 实现 | 要验证的点 |
|---|---|---|
| `Button` | 包装 `gpui_base::Button` | 状态描边、配色、无障碍角色（上游给） |
| `TextField` | 包装 `gpui_base::input::Input` | IME、CJK 候选框、撤销、掩码（上游给） |
| `ListRow` | 自绘 | 行高契约、虚拟化兼容性 |
| `Dialog` | 包装 `gpui_base::Dialog` | 焦点陷阱（上游给）、遮罩、层级 |

**控件本身的行为不该由我们实现**（Q33）。这 4 个控件的作用是验证**视觉层能否覆盖到上游**，不是验证我们能否写出输入法。

### 6.5 与 `gpui-component` 的隔离

```toml
[dependencies]
gpui-kit = { workspace = true, default-features = false }

[features]
default = []
component-fallback = ["gpui-kit/component"]
```

`gpui-kit` 的 feature 名是 `component`（不是 `gpui-component`），且 `default` 会同时带上 `component` 与 `assets`，所以必须显式 `default-features = false`。

只在 `design-system` 内部 `#[cfg(feature = "component-fallback")]` 引用。对外 API 不出现任何 `gpui_component::` 类型。

**当前未启用**：`gpui_base` 已覆盖全部 v1 需求，`component-fallback` 是预留的逃生口，不是默认路径。

---

## 7. `platform-api` / `platform-win`

trait 只有四个，v1 全部要实现：

```rust
pub trait Clipboard { fn set_secret(&self, s: SecretString, ttl: Duration); fn clear(&self); }
pub trait SessionEvents { fn on_lock(&self, cb: Box<dyn Fn() + Send + Sync>); }   // 锁屏
pub trait SingleInstance { fn acquire(&self) -> bool; fn focus_existing(&self); }
pub trait AppPaths { fn config_dir(&self) -> PathBuf; fn data_dir(&self) -> PathBuf; }
```

Windows 实现：

- 剪贴板：`windows` crate 的 `OpenClipboard` / `SetClipboardData(CF_UNICODETEXT)`；`AddClipboardFormatListener` 可用来增强"用户是否覆盖过"判断。
- 锁屏：`WTSRegisterSessionNotification` 监听 `WTS_SESSION_LOCK`。
- 单实例：命名 Mutex + `WM_COPYDATA` 唤起。
- 路径：`%APPDATA%\KeePassDesktop` / `%LOCALAPPDATA%\KeePassDesktop`。

托盘与 autotype 推迟到 v2，但 trait 位置先留。

### 7.1 进程形态：普通 GUI 应用，不是命令行工具

这是产品定位问题，不是技术细节。**面向普通用户，双击即用，不允许出现黑色命令行窗口。**

Rust 在 Windows 上默认产出 `console` subsystem 的二进制，双击时会同时开一个控制台窗口。必须显式声明：

```rust
#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")]
```

`crates/app/Cargo.toml` 的 feature：

```toml
[features]
default = []          # 默认 GUI subsystem
console = []          # 开发时显式打开，看 stdout/stderr
```

**默认必须是 GUI**。开发时想看输出才用 `cargo run -p app --features console`。

#### 代价与补偿

GUI subsystem 会吞掉 stdout/stderr，panic 不再打印到任何地方。静默崩溃比留下日志更糟，所以：

- 无条件安装 panic hook，写 `%LOCALAPPDATA%\KeePassDesktop\panic.log`。
- 只写 panic payload 与位置，**绝不写库数据**（S3/S7）。
- 该文件是用户求助时唯一能提供的诊断信息。

#### 延伸约束（适用于所有后续功能）

只要目标是"普通用户能用的桌面应用"，以下都不能出现：

| 禁止 | 替代 |
|---|---|
| 控制台窗口 | GUI subsystem（本节） |
| 命令行参数作为唯一入口 | 全部功能走 GUI；参数只做可选的"用本程序打开这个文件" |
| 密码作为命令行参数 | 永不接受（S7） |
| 配置文件手写 | 设置页；手改仍支持作为逃生门，但不是唯一路径 |
| 报错要求用户看终端 | 错误在界面内显示恢复动作（§3.4） |
| 依赖系统预装运行时 | 依赖全部静态链接，发布物自带运行所需（Rust 默认静态链接 MSVC 运行时需确认 `/MD` vs `/MT`） |

#### 已实测

| 项 | 结果 |
|---|---|
| PE Subsystem | 2（`IMAGE_SUBSYSTEM_WINDOWS_GUI`） |
| `MainWindowTitle` | `KeePass Desktop` |
| 启动后子进程 | 无 |

---

## 8. `app`

### 8.1 状态机

```
Boot ──> Welcome（无最近库）──────┐
      └> Unlock（有最近库）───────┤
                                  v
                               Unlocked ──> Locked（超时/锁屏/手动）
```

`Unlocked` 之后是主界面：分组侧栏 + 条目列表 + 详情面板（桌面三栏，替代鸿蒙端的 tab + bottom sheet）。

路由不照抄鸿蒙端（它的详情是 `bindSheet` 而不是路由，桌面不合适）。

### 8.2 界面清单（v1）

1. Welcome：新建库 / 打开库
2. Unlock：密码 + 可选密钥文件
3. 主界面：侧栏（分组树）+ 列表 + 详情
4. 条目编辑：模板选择 → 字段表单 → 附件
5. 密码生成器（独立面板，可从任何密码字段唤起）
6. 搜索（覆盖式，`Ctrl+F` 唤起）
7. 设置：自动锁时长、剪贴板时长、主题、语言（只有中文）

鸿蒙端**没有**设置页与安全中心（占位符）。我们必须有，这是桌面用户的基本预期。

### 8.3 快捷键

`Ctrl+F` 搜索、`Ctrl+N` 新建条目、`Ctrl+S` 保存、`Ctrl+L` 手动锁定、`Ctrl+G` 生成密码、`Ctrl+C`（在密码字段上）复制密码。

不使用 `Ctrl+Shift+*` 系列，避免与中文输入法切换冲突。

---

## 9. 安全不变量（可测试）

每条都对应一个测试：

| 编号 | 不变量 |
|---|---|
| S1 | 库锁定时，内存中不存在任何明文密码（用 `zeroize` 的 `Zeroizing` 装箱，测试断言 drop 后缓冲区为零） |
| S2 | 保存前哈希不匹配必须拒绝写入，且原文件字节不改变 |
| S3 | 日志记录器在编译期拒绝 `SecretString` / `Zeroizing<Vec<u8>>`（类型层面不可 `Display`） |
| S4 | 剪贴板 30 秒后内容被清除，且不会清除用户后续复制的内容 |
| S5 | 锁屏事件触发后 1 秒内库处于锁定态 |
| S6 | 二次验证连续失败 3 次后进入 30 秒冷却 |
| S7 | CLI 参数与崩溃信息中不含密码（`main` 不接受密码参数） |

S3 通过让 `SecretString` 不实现 `Display`/`Debug`（`secrecy` 默认如此）+ clippy 禁止 `{:?}` 到 `tracing` 宏来保证。

---

## 10. 测试策略

### 10.1 `keepass-core`：测试向量（Q19，第一批代码）

`crates/keepass-core/tests/fixtures/`：

| 文件 | 用途 |
|---|---|
| `v3.1-aes.kdbx` | KDBX 3.1 读取 |
| `v4.0-aes-argon2id.kdbx` | 主流组合 |
| `v4.1-chacha20-argon2id.kdbx` | 4.1 + ChaCha20 |
| `v4.1-attachments.kdbx` | 附件池 |
| `v4.1-history.kdbx` | 历史记录 |
| `v4.1-custom-icons.kdbx` | IconID ≥ 69 |
| `v4.1-homo-markers.kdbx` | 四个私有 key + `SshKeys` |
| `corrupt-*` | 签名/版本/HMAC 篡改，断言错误分类 |

生成方式：手工用 KeePassXC 生成后提交入库（二进制，体积受控）。每个文件配一份 `expected.json` 描述应读出的字段。

四类测试：

1. **读**：解出预期字段。
2. **往返**：读 → 写 → 再读 → 逐字段比对。
3. **错误分类**：篡改字节 → 断言具体 variant。
4. **未知字段保留**：注入一个未知 `customData` key，写入后必须仍在。

第 4 类专门防鸿蒙端那种字段删除行为。

### 10.2 `design-system`：headless 断言（Q26）

用 `gpui_kit::test` + `TestWindowExt`：

- 解锁成功 → 断言主界面已渲染
- 点击"显示密码" → 断言画面更新（防 retained mode 静默旧帧）
- 自动锁触发 → 断言切回解锁页

### 10.3 CI（Q25）

```yaml
- cargo fmt --check
- cargo clippy -p keepass-core -p vault-model -- -D warnings
- cargo test -p keepass-core -p vault-model
```

`design-system` 与 `app` 不进 CI：GPUI 在无 GPU 的 runner 上编译代价高、运行不稳定。UI 层靠本地测试。

---

## 11. 风险与 gate

| # | 风险 | 严重度 | 缓解 |
|---|---|---|---|
| R1 | **Windows 屏幕阅读器完全不发声**（Zed #41138，JAWS/NVDA 在 Zed 0.209.6 下无声） | 高 | Q15 spike：用 NVDA 实测解锁页。若确认不可用，评估换 egui（accesskit 已是硬依赖）。**未解决** |
| R2 | **CJK IME 缺陷**（Zed 28 个 IME issue，含 shipped input 例子在中文合成后复制会 panic） | 中 | **用上游 `gpui_base::input`，不自己实现**（Q33）。基础输入、粘贴、撤销、掩码已人工验收通过；候选框位置与合成后复制待验 |
| R3 | `gpui-pre` 跟随 Zed，minor 版本就 breaking，单一发布者 | 中 | `Cargo.lock` 入库，CI `--locked`，禁整体 `cargo update`，预算每 3 周 1 天 |
| R4 | `keepass` 0.17.0 发布仅 5 天，下载量 820，4.1 写入是 experimental | 中 | 包在 trait 后；4.1 往返必须用真实 KeePassXC 文件验证，不信它自己的测试。**G2 已过** |
| R5 | 头部 HMAC 校验能力未知 | 低 | 已确认上游无此 API；当前用整库重解密 + 限速，正确性优先 |
| R6 | 与鸿蒙端并发编辑导致覆盖 | **高** | Q29 哈希门禁 + 测试 S2。**已实现并测试** |
| R7 | CJK/Latin 双度量建模工作量 | 低 | 已在 token 层实现，非事后补 |
| R8 | 本机不能用 crates.io，CI 能用 | 低 | 镜像配置留本机，不入库 |

**Gate 顺序**（不通过不进下一阶段）：

```
G1 工具链 + 窗口              ✅ 已通过
G2 keepass-core 读写回环       ✅ 已通过（25 测试，§14）
G3 a11y spike 结论（NVDA）     ⬜ 未开始，需人工
G4 IME spike 结论              🟡 基础能力已通，细节待人工
G5 design token + 4 个控件     ✅ 已通过（§16）
G6 三栏主界面骨架              ⬜ 未开始
```

---

## 12. 里程碑

| 阶段 | 内容 | 出口条件 |
|---|---|---|
| M0 ✅ | 工具链、窗口验证 | G1 通过 |
| M1 ✅ | `keepass-core`：trait、事务、错误分类、测试向量 | G2 通过 |
| M2 ✅ | `vault-model`：领域模型、用例、生成器、强度 | 用例测试全绿 |
| M3 | a11y + IME 双 spike | G3、G4 有明确结论 |
| M4 ✅ | `design-system`：token + 4 原子控件 | G5 通过 |
| M5 | 解锁 → 列表 → 详情的垂直切片 | 能真实打开并浏览库 |
| M6 | 条目/分组 CRUD + 模板 + 附件 | 能完整编辑并保存 |
| M7 | TOTP + 生成器 + 搜索 | 功能线 C 完整 |
| M8 | 设置页、自动锁、剪贴板、再认证、打包 | 可分发 |

### 下一步是什么

- **G3 / G4**：需人工。输入基础能力已验证可用（§18），剩余是 NVDA 读屏与 IME 候选框细节。
- **M5**：`Vault` trait 还没有实现挂到 UI。第一步是把 `KeepassVault` 接到解锁流程——`keepass-core` 的 G2 已过（25 个测试），缺的是 `vault-model` 与 `app` 之间的接线。

---

## 13. 待办验证项

### 13.1 已解决（G2 实测，2026-10-24）

**1. `keepass` 只校验凭据的 API —— 不存在。**

`verify_credential` 目前只能整库重解密（`Database::parse` 一次）。正确但有代价：Argon2id 每次数百毫秒。限速逻辑（`Session::record_reauth`）因此变得必要，而不是可选。

若将来要优化：KDBX 4 头部 HMAC 是公开结构，可在 `keepass-core` 内自己解析头部。**v1 不优化**，因为整库解密在正确性上更保守。

**2. KDBX 4.1 写入 —— 可写，但只能写 4.1。**

`dump_kdbx4` 的源码是：

```rust
if !matches!(db.config.version, DatabaseVersion::KDB4(1)) {
    return Err(DatabaseSaveError::UnsupportedVersion);
}
```

即**只接受 `KDB4(1)`**。KDBX 3.1 和 4.0 都无法写出。所以首次保存会强制升级到 4.1。

这是用户可见的后果：**升级后旧客户端（不支持 4.1 的版本）打不开这个库**。`save_with_transaction` 在升级前把版本改成 4.1，并在文档注释里记录。UI 层未来必须在首次保存时给出明确提示。

**3. 上游 bug：`customData` 值 `"true"` 读不出来。**

`keepass` 0.17 的 `CustomDataValue` 反序列化**先试 base64，再试文本**。`"true"` 恰好是合法 base64（解码为 3 字节 `[182,187,158]`），于是被当成 `Binary`，原文丢失且不可逆（那 3 字节不是合法 UTF-8）。

影响：`KeePassHomo.Favorite` / `.Archived` 存的正是字面 `"true"`/`"false"`。所以**鸿蒙端标记的收藏，我们读不到**（`"false"` 侥幸能读，因为它不是有效的同形 base64）。

已做的处置：

- fixture 拆成两份：`v4-homo-markers.kdbx` 用非 base64 值验证读取路径；`v4-homo-favorite-true.kdbx` 保留真实值。
- 测试 `favorite_true_is_unreadable_upstream` **故意断言这个错误行为**。上游修好后它会失败，那个失败就是移除 workaround 的信号。
- 写入不受影响：我们写入的 `"true"` 是正确的，KeePassXC 能读。

若升级到阻塞程度，缓解方案是在 `keepass-core` 内自己解析条目 XML 的 `customData`，不信任 `Entry::custom_data`。v1 不做。

### 13.2 仍待验证

1. Windows 屏幕阅读器在 GPUI 下是否完全无声（R1）——需人工，无自动化手段。
2. `gpui-kit` 的 `AccessKit` 在 Windows 上实际暴露了哪些 role（R1）——同上。
3. CJK IME 的候选框位置、Esc 取消、上屏后复制（R2）——基础输入已通过人工验收，这三项细节未验。

---

## 14. G2 出口报告（2026-10-24）

| 项 | 结果 |
|---|---|
| 测试向量 | 9 份 fixture，由 **kdbxweb**（JS，独立实现）生成 |
| 外部校验 | 每份都用 `keepassxc-cli ls` / `show --all` 确认可读 |
| 测试 | `cargo test -p keepass-core --test g2_vault` → **25 passed, 0 failed** |
| 覆盖 | KDBX 3.1 / 4.0 / 4.1；AES-256 / ChaCha20；AES-KDF / Argon2id；密码 / 仅密钥文件 |
| 文档形状 | 附件（逐字节）、历史、自定义图标、鸿蒙私有标记 + `SshKeys` |
| 往返 | 读 → 写 → 再读 → 比对；含附件字节与第三方 `customData` 保留 |
| Q29 | 文件被外部改动时拒绝写入，且原文件字节不变 |
| 错误分类 | 密码错误 / 非 KDBX / 截断 / 密钥文件缺失，四类各自断言 |

### 已知偏离

- **`icon_id` 对自定义图标返回 `None`**。`keepass` 把 `<CustomIcon>` 规范化为 `Icon::Custom(id)`，丢弃字面值 69。用 `has_custom_icon()` 判断。这是上游设计，不是 bug。
- **KDBX 4.0 只能读不能写**，写出即升级为 4.1（见 13.1 第 2 条）。

---

## 15. 测试向量再生成

```powershell
cd tools/fixtures
npm install
node gen.js
```

生成器在 `tools/fixtures/gen.js`，已入库。输出写入
`crates/keepass-core/tests/fixtures/`，也是入库的。依赖 `kdbxweb` 与 `hash-wasm`，`node_modules` 已 ignore。

生成后必须用 `keepassxc-cli ls` 逐份确认可读——fixture 由第三方实现生成、再由第三个实现校验，才有意义。

---

## 16. G5 出口报告（2026-10-24，10-25 修订）

### 交付内容

`crates/design-system` 从注释骨架变成可用设计系统。控件最终形态见 §18，此处只记结构与测试。

| 文件 | 内容 |
|---|---|
| `tokens.rs` | primitive / semantic 两层，CJK+Latin 双度量类型标尺，深浅两套 palette |
| `controls/mod.rs` | `ControlState` 六态、状态推导、描边/填充/文字色解析 |
| `controls/button.rs` | 包装 `gpui_base::Button`，四语义 + 三档尺寸 |
| `controls/text_input.rs` | 包装 `gpui_base::input::Input`，只做样式 |
| `controls/list_row.rs` | 自绘双行条目行，行高可虚拟化 |
| `controls/dialog.rs` | 包装 `gpui_base::Dialog`（自带 focus trap） |
| `../app/src/screens/gallery.rs` | 全控件全状态演示页 |

### 测试

**35 个 design-system 测试**，全是纯逻辑与数值断言，不模拟用户操作：

- CJK 行高恒大于 Latin，且 ratio ≥ 1.3
- 混合文本必须用 CJK 线盒（否则中文裁切）
- 间距 token 全部落在 4px 网格
- 三个层级 surface 颜色两两不同（否则容器消失）
- 静息/悬停/按下/禁用不画描边；聚焦/错误画描边且仅变色
- 按钮高度由线盒推导，padding 随尺寸递增
- 行高留出线盒外的余量
- 对话框宽度钳制，最小宽度能容纳 20 字中文

早期一版有 57 个测试，其中 22 个是自研文本编辑引擎的（UTF-16 转换、选区、splice、IME 合成）。改用上游 `gpui_base::input` 后这些测试随之删除——**那部分行为现在由上游负责，不该由我们测**。

全 workspace：**74 passed, 0 failed**；clippy `-D warnings` 干净；fmt 通过。

### 实测发现并修掉的缺陷

见 §17.5（行高两次）、§17.6（漏 derive）、§17.7（按钮文字变灰）。核心教训：**数值断言 + 人工看界面两层都要有，缺一不可**——行高缺 gap 靠测试抓到，字形被裁只能靠眼睛。

### 已确认的 GPUI 事实

- `RenderOnce` **不自动**实现 `IntoElement`，必须 `#[derive(IntoElement)]`
- 帮助函数返回 `impl IntoElement` 时无法链式 `.child()`，要返回具体 `Div`
- 样式方法（`rounded` / `hover` / `cursor_pointer` / `shadow_sm`）由 `#[refineable]` 宏生成，`style.rs` 里搜不到源码
- `Entity::read` 需要 `&App`，所以在没有 `App` 的地方（元素构造器）不能从 state 读调色板，只能作为参数传入
- `TextInputConfiguration` 只有四个字段（`autocorrect` / `autocapitalize` / `suggestions` / `input_action`），**没有** `input_type`；密码字段靠关掉这三个辅助来防泄漏

---

## 17. 实施踩坑清单（2026-10-24 ~ 10-25）

按时间顺序记录真实踩到的坑。每条都附「怎么发现的」与「现在受什么规则保护」，避免只留结论、下次照旧。

### 17.1 判断错误：自研了本应包装的东西

**坑**：从零手写 `TextField`（约 990 行），包含 UTF-16↔char 转换、光标、选区、IME 合成处理、自绘 `Element`。而 `gpui-base/src/input/base/` 里已有完整实现：`state.rs` 483 KB、`element.rs` 194 KB，覆盖 rope 存储、光标、选区、移动、触摸选择、掩码、撤销栈、原生 IME、绘制。

**怎么发现的**：用户直接质疑「gpui-kit 不是封装好了吗？我只是需要重新设计 UI 和 UX」。核查 `gpui-base/src/lib.rs` 的 `pub use` 后确认 `pub use input::{Editor, Input, InputBase, InputStyles, Textarea}`，且 `Input` 的文档原文写着 *"An unstyled single-line text input. Applications that need a fully styled control can wrap this state with their own presentation."*

**后果**：净删约 990 行；更重要的是第一版漏了 `handle_input` 注册，功能完全不可用（见 17.2）。

**现在受什么保护**：AGENTS.md **Q33「组件实现顺序」**——`gpui_base` → 组合原语 → `gpui_component` → 才自研，且自研必须写清上游做不到的证据。判断标准：自研代码里出现"光标/选区/撤销/IME/虚拟化/焦点陷阱"就先停下。

### 17.2 输入完全无效：`EntityInputHandler` 不够

**坑**：实现了 `EntityInputHandler` 全部方法，但一个字都输不进去。

**根因**：GPUI 的输入链路是两段，缺第二段则第一段完全无效：

1. 实现 `EntityInputHandler`（`replace_text_in_range` / `marked_text_range` / `paste`）
2. **在 `Element::paint` 里调用 `window.handle_input(&focus_handle, ElementInputHandler::new(bounds, entity), cx)`**

**表现**：能聚焦、有光标、按钮可点，但键事件、IME、粘贴全都到不了处理器。看起来像正常控件，实际是死的。

**教训**：「能聚焦」靠 `track_focus`，「能输入」靠 `handle_input`，两件独立的事。

**现在受什么保护**：包装上游后不再需要手动接线；Q33 也会在下次自研时先问一句"上游是不是已经做了"。

### 17.3 无法滚动：缺 `overflow_y_scroll`

**坑**：gallery 内容超出窗口，滚不动。

**根因**：根容器没写 `overflow_y_scroll()`，超出部分只是被裁掉。

**修复要点**：`.id(...)` + `.overflow_y_scroll()` + `.h_full()`。用 `size_full()` 不行，高度撑满就没有可滚动区域；`id` 是必需的，GPUI 只为有稳定身份的元素保存滚动状态。

### 17.4 输入框互相挤压：自定义 `Element` 别声明 `relative(1.0)` 宽度

**坑**：gallery 三个静态输入框重叠，文字互相覆盖。

**根因**：自研 `InputElement::request_layout` 里写了 `style.size.width = relative(1.0)`。在宽度不确定的 flex row 里，这个子元素抢占全部剩余空间，把兄弟压成零宽。

**修复**：`request_layout` 只固定高度，宽度交给父级；`TextField` 提供 `width(f32)`，默认 240px，字段不自己撑开。

> `InputElement` 已随自研引擎一起删除（§18）。这条教训仍然有效：**以后写任何自定义 `Element` 都不要声明 `relative(1.0)` 宽度。**

### 17.5 行高算错两次

**第一次**：`row_height()` 漏了标题与副标题之间的 `gap`。测试断言「内容需要 66、行高只有 58」直接抓到。

**第二次**：补了 gap 后截图里 "GitHub" 的 G 顶部、中文标题上行仍被切。原因：行高等于线盒总和时没有余量，而 CJK 上伸部与拉丁下伸部会画出线盒。改为每边多留一个 `CONTROL_Y`。

**教训**：数值断言和人工看界面是两层，缺一不可。第一次靠测试，第二次只能靠眼睛。

### 17.6 `#[derive(IntoElement)]` 漏了两次

**坑**：重写 `TextField` 和 `Button` 时都漏了，`into_any_element()` 报「方法不存在」。

**规则**：`RenderOnce` **不会**自动提供 `IntoElement`，每个自定义元素都要显式 `#[derive(IntoElement)]`。

### 17.7 按钮文字变灰

**坑**：按钮 label 写成 `child(div().child(label))`，颜色不生效。

**根因**：多包一层 `div` 会另起样式上下文，不继承父级的 `text_color`。

**修复**：直接 `child(label)`。

### 17.8 `Dialog` 默认 `open: true` 盖住整页

**坑**：截图里整个界面被一个对话框挡住，所有点击都被吞。

**修复**：模态默认关闭，调用方显式 `open(true)`。画廊里的对话框保持关闭，只展示触发说明。

### 17.9 上游 API 与直觉不符的几处

| 现象 | 实际 |
|---|---|
| `Entity::read(|s| ...)` 在元素构造器里不可用 | 需要 `&App`；没有 `App` 时调色板只能作为参数传入 |
| `TextInputConfiguration` 没有 `input_type` 字段 | 只有 `autocorrect` / `autocapitalize` / `suggestions` / `input_action`；密码字段靠关掉三个辅助防泄漏 |
| `UpstreamDialog::new()` | 需要 `&mut App`，内部持有 `FocusHandle` |
| 样式方法在 `style.rs` 里搜不到 | 由 `#[refineable]` 宏生成；查 examples 或 `div.rs` 更可靠 |
| `keepass` 的 `Corrupt(&'static str)` | 曾用 `Box::leak` 塞运行时文本，泄露内存；改为 `Corrupt { detail: String }` |

### 17.10 截图验证的硬限制

无人值守环境里**无法可靠**把窗口切到前台：`SetForegroundWindow` 在 DPI 缩放的多显示器环境下经常失败（实测 `foreground matches=False`），合成点击会落到别的窗口。

可靠手段：`PrintWindow(hwnd, hdc, 2)` 直接读窗口内容，不需要前台。但它抓不到合成点击的结果。

| 手段 | 能验证 | 不能验证 |
|---|---|---|
| `PrintWindow` | 静态布局、颜色、文字渲染 | 点击、输入、IME |
| 合成点击 | —（前台拿不到，落点不可控） | — |
| `KEEPDESKTOP_START_ROUTE=gallery` | 直达指定页面，绕过点击 | 仍然不能模拟输入 |

**结论**：交互路径只能人工验收。这与 AGENTS.md「禁止模拟用户操作的测试」一致。

---

## 18. 组件最终形态（2026-10-25）

| 控件 | 实现方式 | 上游来源 | 自研部分 |
|---|---|---|---|
| `TextField` | 包装 | `gpui_base::input::{Input, InputState}` | 边框、背景、状态色、helper 文本 |
| `Button` | 包装 | `gpui_base::Button` | 四语义配色、三档尺寸、padding |
| `Dialog` | 包装 | `gpui_base::{Dialog, DialogTitle, DialogDescription}` | 遮罩色、面板表面、字号 |
| `ListRow` | 自绘 | — | 整行（纯展示，无交互状态机） |
| tokens | 自研 | — | 全部（这就是"重新设计 UI"的本体） |

`TextField` 从 990 行降到 157 行。行为（光标、选区、IME、撤销、掩码、tab 顺序、键盘激活、焦点陷阱）全部归上游。

**`ListRow` 为什么允许自绘**：它是展示组件，自研代码里没有出现 Q33 列出的任何危险词。将来承载多行、需要虚拟化的**列表容器**必须用 `gpui_base::v_virtual_list`，不是自绘滚动。

### 人工验收结果（2026-10-25）

用户实测：**输入、粘贴、IME、撤销、掩码全部可用。**

这一项无法自动验证（见 17.10），是包装上游后才达成的。此前自研版本「一个字都打不进去」。

---

## 19. 图标库：Material Symbols（2026-10-25）

### 19.1 决策

| 项 | 值 |
|---|---|
| 来源 | `google/material-design-icons`，pin commit `10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7` |
| 变体 | **sharp** / wght400 / 24px，`fill0` + `fill1` 成对 |
| 形态 | **SVG**，不用图标字体 |
| 通道 | `Svg::path` + 自有 `AssetSource`（`include_bytes!` 静态字节） |
| 归属 | `design-system`：`src/icons.rs`（枚举 + 资产源）、`src/controls/icon.rs`（元素） |
| 许可 | Apache-2.0，全文随资产放置 |

### 19.2 为什么不是图标字体

`gpui-pre` 0.3.9 只支持 `FontFeatures`（4 字符 OpenType tag，如 `liga`），**没有
`font-variation-settings`**。Material Symbols 的 FILL / GRAD / opsz / wght 四个轴一个都
设不了，字体路线只能固定一个静态实例，等于放弃这套图标的核心能力。变量字体本体
`MaterialSymbolsOutlined[FILL,GRAD,opsz,wght].ttf` 还有 10.7 MB。

SVG 路线配合 GPUI 的 `render_alpha_mask`（只取 alpha 通道、再用 `text_color` 上色）：
浅色与深色主题**共用同一份资源**，主题切换零成本。代价是**不能做多色图标**——这正好
是我们不需要的。

### 19.3 为什么 `path` 而不是 `data`

`Svg::data(&[u8])` 每次构造都：`DefaultHasher` 哈希全部字节 → 生成
`__binary_svg__{hash}` 路径 → `Arc::from` 拷贝一份字节。放在 `render` 里就是**每帧每
图标一次分配 + 一次全文件哈希**。

`Svg::path` 则按 `(path, size)` 命中窗口的 sprite atlas 缓存，字节经 `AssetSource::load`
借用返回，只有首次渲染才会读一次。

代价：必须在 `application()` 上注册资产源。为此 `design_system::prelude::application`
不再是 `pub use gpui::application`，而是包一层 `.with_assets(IconAssets)`。**忘了它图标
会静默变空白**——`paint_svg` 把「资产不存在」当「不画」，不报错也不 panic。

### 19.4 Q33 判定

1. `gpui-base` 0.8.0 的 75 个 `pub use` 里**没有** Icon（`Icon` 只出现在注释里）→ 落空
2. 无原语可组合（`svg()` 是 gpui 的，不在 base）→ 不适用
3. `gpui-component::Icon` 存在，但它是 `svg()` + `Sizable` + 主题前景色的薄封装；用它要
   打开 `component` feature、把整个 gpui-component 拉进依赖图
4. 因此自研，范围限于「选文件 + 定尺寸 + 定颜色」，约 60 行

### 19.5 上游事实：部分图标的 fill1 与常规态字节相同

`add` / `check` / `search` / `attach_file` / `refresh` 的 `_fill1_` 文件与常规态完全一致
——纯几何图形没有「描边 vs 实心」的区别。`filled()` 对它们是**视觉空操作**。

这不是抓取漏文件。测试 `filled_variants_known_identical` 断言这个现状：哪天上游补上
了填充态，它会红，提醒把图标从名单里移除。

### 19.6 加一个图标的步骤

1. `tools/icons/fetch.ps1` 的 `$Names` 加名字，重跑脚本
2. `crates/design-system/src/icons.rs`：`IconName` 加变体 → `files()` 加一条 → `BUNDLED` 加两条
3. 漏了文件 `include_bytes!` 编译期就失败

### 19.7 验收

| 项 | 状态 |
|---|---|
| `cargo test -p design-system` | 44 passed（含 5 条图标断言） |
| `cargo clippy --workspace --all-targets -- -D warnings` | 干净 |
| `cargo fmt --all --check` | 通过 |
| `gallery` 图标展示区肉眼验收 | **待用户** |

断言覆盖：资源键唯一且成对、SVG 开头与 viewBox、填充态命名、尺寸在 4px 栅格、
尺寸递增、默认色与默认填充态。

**不做**：渲染像素断言。图标视觉重量、与文字的基线关系由人工验收（同 17.10）。

---

## 附录 A：与鸿蒙端的功能差异（我们**必须**补齐的）

鸿蒙端现状（`d91b5f6`）标记为 absent/stub，而我们必须做的：

| 功能 | 鸿蒙端 | 我们 |
|---|---|---|
| 搜索 | 无去抖、无排序、无拼音 | 归一化 + 拼音 + 权重排序 |
| 分组 CRUD | 已实现（`GroupManagerPage`） | 已对齐 |
| 回收站 UI | 无（删除直接移除） | 必须做（还原 / 彻底删除 / 清空） |
| 历史记录 UI | 无 | 必须做 |
| TOTP | 无（只有 URI 解析器） | 必须做 |
| 设置页 / 安全中心 | 占位符 | 必须做 |
| 多库 | 无（单库 + 最近路径） | 数据结构支持，v1 单库 |
| 二次验证 | 仅附件预览/导出，无门禁无限速 | 六操作 + 限速 |
| 导入导出 | 无 | v2 |
| autotype | 无 | v2 |
| WebDAV | 只有探测 | 不做（外部同步） |

---

