# KeePassHomo — Reference Inventory (read-only survey)

Source: `E:\WorkSpace\KeePassHomo`, branch `master`, head `bb067f2`.

## 1. Tech stack as found

- **Language**: ArkTS (`.ets`) + TypeScript. No Rust/C++ app code.
- **UI**: ArkUI declarative, state management **V2** (`@ComponentV2`, `@ObservedV2`, `@Trace`, `@Local`, `@Param`, `@Event`, `@Computed`, `@Monitor`, `AppStorageV2`) plus HDS kit (`@kit.UIDesignKit`: `HdsNavigation`, `HdsTabs`, `HdsListItem`) and `uiMaterial` immersive materials.
- **API level**: target = compatible = **26.0.0** (`build-profile.json5`, `hvigor/hvigor-config.json5` `modelVersion 26.0.0`); devices `phone`, `tablet` (`products/default/src/main/module.json5`).
- **Build**: Hvigor (`hvigorfile.ts` → `appTasks`/`hapTasks`), ohpm, `devecocli build --product default --build-mode debug` (`README.md`). `signingConfigs: []` by design (per-developer auto-sign).
- **Modules**: `products/default` (entry HAP), `common` (HAR `@ohos/common`), `features/adaptiveLayout`, `features/responsiveLayout` (template demos, empty `Index.ets`). `tools/kdbx-test-data` is a Node CLI excluded from the HAP.
- **Libraries**: `kdbxweb@2.3.0` (only in `common`), `@pura/harmony-utils@1.4.2`, `@wolfx/date-fns@1.0.1`; dev `@ohos/hypium@1.0.25`, `@ohos/hamock@1.0.0`. kdbxweb transitives: `@xmldom/xmldom@0.9.8`, `@ohos-rs/argon2@0.1.3`, `@ohos/flate2@1.0.0` (bytecode HAR with arm64-v8a/x86_64 natives).

## 2. KDBX / crypto

**Crypto is not hand-written** — all parsing and cryptography is delegated to `kdbxweb`.

Parsing entry point `common/src/main/ets/vault/KdbxVaultEngine.ets`: `toFormat()` (signatures `0x9AA2D903/0xB54BFB67`, major word `>>>16`; 3→`Kdbx31`, 4→`Kdbx4`), `unlock()` (`new KdbxCredentials(ProtectedValue.fromString(pw), keyFile)` → `Kdbx.load(data, creds, {preserveXml:true})`), `write()`→`kdbx.save()`, `upgrade()`→`kdbx.upgrade()`, plus `mapDocument/mapGroup/mapEntry`. Credentials assembled in `products/default/src/main/ets/model/StageOneVaultStore.ets` `readCredential()`; files via `common/.../vault/FileVaultRepository.ets`.

- **Versions**: library declares `MinSupportedVersion 3`, `MaxSupportedVersion 4` → **KDBX 3.1 and 4.x**. The app only reads the major word, so **4.1 loads and is labelled `VaultFormat.Kdbx4`** (no 4.0/4.1 distinction anywhere). Written files use the library default 4.0.
- **Ciphers**: library supports `CipherId.Aes` (AES-256-CBC) and `CipherId.ChaCha20` (`crypto-engine.d.ts`: `createAesCbc`, `chacha20`); protected-stream `CrsAlgorithm` Null/ArcFour/Salsa20/ChaCha20. **The app never selects cipher or KDF** (no `setKdf`/cipher call in app code) — it inherits the default (AES).
- **KDF**: `KdfId.Aes`, `Argon2d`, `Argon2id` all supported; default Argon2d. Argon2 is supplied by the HAR's bundled native module, not by app code.
- **Compression**: `CompressionAlgorithm.None/GZip` handled by the library; app never sets it.
- **Inner header**: inside `KdbxHeader.writeInnerHeader`; untouched by app code.
- **Attachments**: only counted — `mapEntry` sets `attachmentCount: entry.binaries.size`. No read/add/export.
- **Custom icons**: handled properly. `writeAvatar()` honours the KDBX rule `IconID >= 69` (`CUSTOM_ICON_ID`), stores bytes in `meta.customIcons`, and `customIconUuid()` dedupes by byte compare (`sameBytes`).
- **History**: `entry.pushHistory()` on update/flag change; `historyCount` exposed but never rendered.
- **Recycle bin**: `meta.recycleBinUuid` → `VaultDocument.recycleBinUuid`; `StageOneVaultStore` hides that subtree **by UUID, not name**.

## 3. Feature inventory

| Feature | State | Evidence |
|---|---|---|
| Unlock: password | yes | `UnlockPage.ets`, `StageOneVaultStore.unlock` |
| Unlock: key file / both | yes, optional combined | `readCredential`, `UnlockPage.keyFileBuilder` |
| Vault list | **absent** — single recent vault from preferences | `persistRecentSource` |
| Group tree | **partial** — top-level chips, multi-select, no hierarchy/expand | `VaultTabChrome.ets` |
| Entry list | yes (cards) | `VaultHomePage.ets`, `EntryCard.ets` |
| Search | **stub** — `SearchPage` body is an empty `Scroll`; `setSearchQuery` never called | `SearchPage.ets:50-66` |
| Entry detail | yes, as bottom sheet not route | `EntryPreviewContent.ets` |
| Create/edit/delete entry | yes | `KdbxVaultEngine.createEntry/updateEntry/removeEntry` |
| Create/edit/delete group | **absent** | no `createGroup`/`removeGroup` call |
| Copy username/password/URL | yes (+notes, +custom fields) | `EntryPreviewContent.copy*` |
| Password generator | yes — random (8–20, 4 classes, exclude similar, min digits/symbols), memorable (3–8 words, separator, case, digit), PIN (4–12, avoid patterns) | `PasswordGenerator.ets`, `GeneratorViewModel.ets` |
| TOTP | **absent in UI** — only `OtpAuthParser.ets` (URI→config); no code generation | no reference under `products/` |
| Attachments | **absent** | count only |
| Custom fields | yes — 8 types, protected flag, type in customData `KeePassHomo.FieldTypes` | `writeFieldTypes`, `EntryCustomFields.ets` |
| Tags | yes, rejects separators | `EntryTagEditor.ets` |
| Favorites / archived | yes, app-private (`KeePassHomo.Favorite` / `.Archived`) | `setEntryFavorite/setEntryArchived` |
| Recycle bin UI | **absent** — delete via `kdbx.remove`, no restore/empty | `isRecycleBin` |
| History UI | **absent** | `historyCount` unused |
| Auto-lock | 5 min fixed + foreground check; toggle has no UI | `VaultSessionViewModel`, `AppSettingsState` |
| Clipboard auto-clear | 30 s, ownership via pasteboard `changeCount` | `SecureClipboard.ets` |
| Autotype | **absent** | grep |
| Import/export CSV/XML | **absent** | grep |
| WebDAV sync | **probe only** (`PROPFIND`+`HEAD`, ETag/Last-Modified); no transfer/conflict | `sync/WebDavProbe.ets` |
| Local file backup/relocate | yes | `FileVaultRepository` |
| Biometric unlock | **code exists, unused** — `LocalUnlockMaterialStore` never imported by `products/` | grep |
| Dark/light | system-follow only; `resources/dark` overrides 2 colors | `dark/element/color.json` |
| i18n | **one language (zh-CN)**; no locale qualifier dirs | resource tree |
| Settings screen | **absent** — placeholder text | `ModulePendingContent.ets` |
| Security center | **absent** — placeholder | same |
| Entry templates | strings only, no UI | `string_vault.json` |

## 4. UI/UX structure

`pages/Index.ets` → `components/AppShell.ets` (one `HdsNavigation` + `HdsTabs`, 4 tabs: Vault / Security / Generator / Settings). Routes in `resources/base/profile/route_map.json`: `welcome`, `createVault`, `fileSummary`, `unlock`, `search`. Entry detail/edit is a `bindSheet` (`EntrySheet.ets`), not a route.

Flow: privacy cover (`SplashContent`) → `resetTo(welcome)` with no recent vault, else `resetTo(unlock)` → `fileSummary` (metadata + read-only banner) → `unlock` → `clearToHome`. Lock → `resetTo(unlock)`. Root back shows a toast; second press within 2 s backgrounds the app (`AppRouter.handleRootBack`).

Patterns: floating capsule tab bar; module-supplied title/filter rows via the `ModuleTabChrome` `WrappedBuilder` contract; `HdsListItem` swipe edit/delete over `Repeat.virtualScroll`; delete collapses row height to 0 then rebuilds the snapshot in `animateTo.onFinish` (avoids a flash on the removed node); FAB flips side to the holding hand via `MultimodalAwarenessKit` with two spring curves; unlock screen splits two `Rect`s 20/80 % that translate apart and reverse on lock; strength meter is five 14 vp dots coloured by score; search sits in the title bar as an ULTRA_THIN `Search` on its own fade route (non-functional); generator uses a segmented capsule and a 17.7 MB mono font with per-character-class colouring.

## 5. Decisions worth copying / avoiding

**Copy**
1. `FileVaultRepository.writeSafely`: temp write → re-read → validate by real unlock → back up target → overwrite → validate again → restore backup on failure; temp cleaned in `finally`. Docs refuse to claim power-fail atomicity.
2. `StageOneVaultStore.lock()` zeroes `keyFile.fill(0)`, blanks the password, drops `credential` and replaces the engine (discarding the in-memory `Kdbx`); re-auth zeroes in `finally`.
3. Boundary-only error taxonomy → `AppErrorModel{category,message,recoveryAction,logCode}`; generic user text, codes in `hilog`; corrupt vs unsupported vs bad-signature vs bad-credential are distinct.
4. `hasAnyEntry` separates empty vault from filter-no-match.
5. Snapshot **reference** comparison (`VaultHomePage.syncRows`) instead of "list empty → rebuild", avoiding rebuild loops.
6. Row models pre-resolve avatar bytes/favorite so list items never load data in `aboutToAppear` (cites the ArkUI node-reuse FAQ).
7. `KdbxVaultEngine.selfTest` runs v4 round-trip (protected field, history, attachment, recycle bin), v3 read-only + upgrade, then deliberate signature/version/byte corruption asserting each error category.
8. `IconID >= 69` + byte-deduped icon pool — a genuine KeePass interop detail.

**Avoid**
- `mapEntry`/`fieldValue` call `ProtectedValue.getText()`, copying protected plaintext into `VaultDocument` and to the UI — contradicting `docs/stage-0-technical-validation.md` ("protected fields expose only name and protection state"). `credential.password` is also a live `string`.
- App-private metadata (`KeePassHomo.*`) instead of standard KDBX tags/fields — invisible to KeePass.
- Two module-level singletons plus an `AppViewModel.revision` counter as a refresh protocol, with manual `invalidate()` at ~10 call sites.
- 17.7 MB font and 664 KB empty-state PNG shipped in the HAP.

## 6. Admitted limitations / TODOs

- `docs/keepass-harmonyos-development-plan.md`: only stages 0–1 are ticked; **M2–M8 all unchecked**. Sections 7–11 stay largely `[ ]`: group management, attachments, history, filters, TOTP, multi-select, recycle bin, import/export, WebDAV sync, biometric, autofill.
- §6.4/§17: unified HDS title bar with `IMMERSIVE_GRADIENT_BLUR` **not done**; undecided whether it belongs to stage 2 or 7.
- §18: `checkPersistentPermission()` returned `false` on the API 26 Beta2 emulator even after granting, and revocation did not take effect there — **must be re-verified on real hardware**. Same section states the write path is not power-fail atomic.
- §15 and `AGENTS.md`: no unit/`ohosTest`/white-box tests; only static lint (`code-linter.json5`, incl. security rules) and build are trusted.
- Code gaps: `SearchPage` renders an empty column; `string_technical_validation.json` and template strings are unreferenced; `StageZero*Verifier`, `OtpAuthParser`, `LocalUnlockMaterialStore`, `WebDavProbe` are exported from `common/Index.ets` but unused by the app.
- `tools/kdbx-test-data/README.md`: the CLI cannot open key-file vaults, and the app keeps a stale in-memory copy after an external file swap (lock/unlock needed).
