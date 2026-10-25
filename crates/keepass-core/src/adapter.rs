//! The `keepass` crate adapter.
//!
//! Everything that knows the `keepass` crate's types lives here. The rest of the
//! crate sees only [`crate::api`] traits and [`crate::error`] variants, so the
//! upstream crate stays replaceable (architecture.md Q17).
//!
//! `keepass` 0.17 addresses entries by a real `EntryId` (a UUID), not by
//! position, so our public id is that UUID directly. Field ids are derived from
//! the field name because KDBX keys fields by name.

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use keepass::db::{CustomDataItem, CustomDataValue, EntryId as KpEntryId, Icon, Value};
use keepass::{Database, DatabaseKey};
use secrecy::{ExposeSecret, SecretString};
use sha2::{Digest, Sha256};

use crate::api::{
    ChangeSet, Credential, EntryId, FieldId, OpenedIdentity, Vault, VaultTransaction,
};
use crate::error::{CoreError, CoreResult};
use crate::interop;

/// Field names KDBX reserves; anything else is a custom field.
pub const RESERVED_FIELDS: &[&str] = &["Title", "UserName", "Password", "URL", "Notes"];

pub struct KeepassVault {
    db: Database,
    path: PathBuf,
    identity: OpenedIdentity,
    last_activity: Instant,
}

/// Never prints the database, only where it lives and how big it is. A `Debug`
/// impl that walked the `Database` would put every field, including protected
/// ones, into a log line (security invariant S3).
impl std::fmt::Debug for KeepassVault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeepassVault")
            .field("path", &self.path)
            .field("entries", &self.db.num_entries())
            .field("groups", &self.db.num_groups())
            .finish_non_exhaustive()
    }
}

/// Field ids are derived from the field name, since KDBX keys fields by name.
fn field_id_for(name: &str) -> FieldId {
    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    // Mark it as a v4 UUID so it is a well-formed value.
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    uuid::Uuid::from_bytes(bytes)
}

fn build_key(credential: &Credential) -> CoreResult<DatabaseKey> {
    if credential.password.is_none() && credential.key_file.is_none() {
        return Err(CoreError::BadCredential);
    }

    let mut key = DatabaseKey::new();

    if let Some(password) = credential.password.as_ref() {
        key = key.with_password(password.expose_secret());
    }

    if let Some(path) = credential.key_file.as_ref() {
        let bytes = std::fs::read(path).map_err(|source| CoreError::Io {
            path: path.clone(),
            source,
        })?;
        // A key file that cannot even be read as key material is a credential
        // problem, not corruption of the vault.
        key = key
            .with_keyfile(&mut Cursor::new(bytes))
            .map_err(|_| CoreError::BadCredential)?;
    }

    Ok(key)
}

/// Map a `keepass` open error onto our classification.
///
/// Distinguishing "wrong password" from "corrupt file" is the point of having a
/// taxonomy: the user's recovery action differs.
///
/// ## Why `E::Key` is matched by message
///
/// `DatabaseKeyError` lives in a private module (`keepass::key`) and is not
/// re-exported, so it cannot be named here. The variants we care about are
/// `IncorrectKey` (`#[error("Incorrect key")]`) and `EmptyKey`
/// (`#[error("The key contains no components")]`). Matching on those exact
/// strings is fragile, so `tests::error_classification` locks the behaviour
/// against a real wrong-password vault: if upstream rewords the message, that
/// test fails instead of silently misreporting a bad password as corruption.
fn classify_open_error(e: &keepass::db::DatabaseOpenError) -> CoreError {
    use keepass::db::DatabaseOpenError as E;

    match e {
        E::Io(_) => CoreError::Corrupt {
            detail: "io while reading".into(),
        },
        E::UnexpectedEof => CoreError::Corrupt {
            detail: "unexpected end of file".into(),
        },

        // Signature or header version could not be parsed.
        E::VersionParse(_) => CoreError::NotAKdbxFile,

        E::UnsupportedVersion => CoreError::UnsupportedVersion("unsupported by parser".into()),

        // The one path that means "your credential is wrong". The UI must say
        // exactly that, not "file is corrupt".
        E::Key(inner) => {
            let msg = inner.to_string();
            if msg.contains("Incorrect key") || msg.contains("no components") {
                CoreError::BadCredential
            } else {
                CoreError::Corrupt { detail: msg }
            }
        }

        // A padding or length failure after key derivation is what AES-CBC
        // decryption of garbage looks like: wrong key surfaced as a crypto error.
        E::Cryptography(_) => CoreError::BadCredential,

        E::Format(inner) => CoreError::Corrupt {
            detail: inner.to_string(),
        },

        // `DatabaseOpenError` is #[non_exhaustive]: a newer upstream release can
        // add a variant. Treat an unknown one as corruption rather than claiming
        // a specific cause; the log code still distinguishes it from a bad
        // credential, which is the distinction that matters to the user.
        other => CoreError::Corrupt {
            detail: format!("unrecognised open error: {other}"),
        },
    }
}

impl KeepassVault {
    pub fn open(path: &Path, credential: &Credential) -> CoreResult<Self> {
        let identity = OpenedIdentity::of(path)?;
        let bytes = std::fs::read(path).map_err(|source| CoreError::Io {
            path: path.to_path_buf(),
            source,
        })?;

        let key = build_key(credential)?;
        let db = Database::parse(&bytes, key).map_err(|e| classify_open_error(&e))?;

        Ok(Self {
            db,
            path: path.to_path_buf(),
            identity,
            last_activity: Instant::now(),
        })
    }

    fn kp(&self, entry: EntryId) -> CoreResult<KpEntryId> {
        // Our EntryId *is* the upstream UUID, so this is a bijection.
        Ok(KpEntryId::from(uuid::Uuid::from_bytes(entry.into_bytes())))
    }

    /// Private `customData` marker on an entry.
    ///
    /// ## Known upstream defect: `"true"` is unreadable
    ///
    /// `keepass` 0.17's `CustomDataValue` deserializer tries base64 **before**
    /// plain text. `"true"` is itself valid base64 ("true" -> 3 bytes
    /// `[182, 187, 158]`), so the parser yields `Binary`, silently destroying the
    /// original text. The transform is lossy: those 3 bytes are not UTF-8, so the
    /// value cannot be recovered here.
    ///
    /// This is not cosmetic. `KeePassHomo.Favorite` and `.Archived` use exactly
    /// the literal `"true"`/`"false"`, so a favorite flag written by the
    /// HarmonyOS client reads back wrong through this crate. `"false"` happens to
    /// survive only because it is *not* valid base64 of the same shape.
    ///
    /// Consequences we accept for now, recorded in architecture.md §13:
    /// - We can still *write* the correct value (writing is unaffected).
    /// - We cannot *read* a `"true"` marker, so favorites are reported as unset.
    ///   Neither we nor the other client corrupts the file; only the reading is
    ///   wrong, and only until either this crate or the fixture is fixed.
    ///
    /// Mitigation if it becomes blocking: decode the entry's `customData` from the
    /// raw XML ourselves inside `keepass-core` instead of trusting
    /// `Entry::custom_data`. Tracked as an open item.
    pub fn custom_data(&self, entry: EntryId, key: &str) -> CoreResult<Option<String>> {
        let id = self.kp(entry)?;
        let e = self.db.entry(id).ok_or(CoreError::NotFound("entry"))?;
        Ok(e.custom_data.get(key).and_then(|item| match &item.value {
            Some(CustomDataValue::String(s)) => Some(s.clone()),
            // The parser chose the binary branch. Only a genuine binary value
            // should land here; the lossy base64 case is unrecoverable, so this
            // returns bytes only when they are valid UTF-8.
            Some(CustomDataValue::Binary(bytes)) => decode_misparsed_text(bytes),
            None => None,
        }))
    }

    /// Whether the entry carries any private marker we must preserve.
    pub fn homo_markers(&self, entry: EntryId) -> CoreResult<Vec<(String, String)>> {
        let mut out = Vec::new();
        for key in interop::WRITTEN_CUSTOM_DATA_KEYS {
            if let Some(v) = self.custom_data(entry, key)? {
                out.push(((*key).to_string(), v));
            }
        }
        Ok(out)
    }

    /// `(id, title)` for every entry, sorted by title. Test and listing helper.
    ///
    /// A missing title is an empty title, not an error: KeePass permits it and
    /// the UI shows a placeholder.
    pub fn titles(&self) -> Vec<(EntryId, String)> {
        let mut out: Vec<(EntryId, String)> = self
            .db
            .iter_all_entries()
            .map(|e| (e.id().uuid(), e.get_title().unwrap_or_default().to_string()))
            .collect();
        out.sort_by(|a, b| a.1.cmp(&b.1));
        out
    }

    pub fn entry_by_title(&self, title: &str) -> Option<EntryId> {
        self.titles()
            .into_iter()
            .find(|(_, t)| t == title)
            .map(|(id, _)| id)
    }

    /// How many entries carry a given custom-data key.
    pub fn count_with_marker(&self, key: &str) -> usize {
        self.db
            .iter_all_entries()
            .filter(|e| e.custom_data.contains_key(key))
            .count()
    }

    pub fn attachment_names(&self, entry: EntryId) -> CoreResult<Vec<String>> {
        let id = self.kp(entry)?;
        let e = self.db.entry(id).ok_or(CoreError::NotFound("entry"))?;
        Ok(e.attachments_named()
            .map(|(name, _)| name.to_string())
            .collect())
    }

    pub fn attachment_bytes(&self, entry: EntryId, name: &str) -> CoreResult<Vec<u8>> {
        let id = self.kp(entry)?;
        let e = self.db.entry(id).ok_or(CoreError::NotFound("entry"))?;
        let (_n, att) = e
            .attachments_named()
            .find(|(n, _)| *n == name)
            .ok_or(CoreError::NotFound("attachment"))?;
        match att.data {
            Value::Unprotected(ref b) => Ok(b.clone()),
            Value::Protected(ref b) => Ok(b.expose_secret().clone()),
        }
    }

    pub fn history_count(&self, entry: EntryId) -> CoreResult<usize> {
        let id = self.kp(entry)?;
        let e = self.db.entry(id).ok_or(CoreError::NotFound("entry"))?;
        // `history` is None until the entry has been edited at least once.
        Ok(e.history
            .as_ref()
            .map(|h| h.get_entries().len())
            .unwrap_or(0))
    }

    pub fn custom_icon(&self, entry: EntryId) -> CoreResult<Option<Vec<u8>>> {
        let id = self.kp(entry)?;
        let e = self.db.entry(id).ok_or(CoreError::NotFound("entry"))?;
        // CustomIconRef derefs to CustomIcon, whose `data` is a plain Vec<u8>.
        Ok(e.custom_icon().map(|icon| icon.data.clone()))
    }

    /// The raw `entry.icon` id, when the icon is one of KeePass's built-ins.
    ///
    /// Returns `None` for a custom icon: `keepass` normalises
    /// `<CustomIcon><UUID>` into `Icon::Custom(id)` and drops the literal
    /// `IconID` of 69. Use [`Self::custom_icon`] to tell whether a custom icon
    /// exists; this method only reports built-in indices.
    pub fn icon_id(&self, entry: EntryId) -> CoreResult<Option<usize>> {
        let id = self.kp(entry)?;
        let e = self.db.entry(id).ok_or(CoreError::NotFound("entry"))?;
        Ok(match e.icon() {
            Some(Icon::BuiltIn(v)) => Some(*v),
            _ => None,
        })
    }

    /// Whether the entry points at a custom icon, regardless of the stored id.
    pub fn has_custom_icon(&self, entry: EntryId) -> CoreResult<bool> {
        let id = self.kp(entry)?;
        let e = self.db.entry(id).ok_or(CoreError::NotFound("entry"))?;
        Ok(matches!(e.icon(), Some(Icon::Custom(_))) || e.custom_icon().is_some())
    }

    /// Field names on an entry, including protected and custom ones.
    pub fn field_names(&self, entry: EntryId) -> CoreResult<Vec<String>> {
        let id = self.kp(entry)?;
        let e = self.db.entry(id).ok_or(CoreError::NotFound("entry"))?;
        Ok(e.fields.keys().cloned().collect())
    }

    /// Whether a named field is stored protected.
    pub fn is_protected(&self, entry: EntryId, name: &str) -> CoreResult<bool> {
        let id = self.kp(entry)?;
        let e = self.db.entry(id).ok_or(CoreError::NotFound("entry"))?;
        Ok(e.fields
            .get(name)
            .map(|v| v.is_protected())
            .unwrap_or(false))
    }

    /// Non-secret field value, for tests that need to compare unprotected data.
    pub fn unprotected_value(&self, entry: EntryId, name: &str) -> CoreResult<Option<String>> {
        let id = self.kp(entry)?;
        let e = self.db.entry(id).ok_or(CoreError::NotFound("entry"))?;
        Ok(e.fields.get(name).map(|v| v.get().to_string()))
    }

    pub fn still_current(&self) -> CoreResult<bool> {
        self.identity.is_current()
    }

    /// Persist, refusing when the file changed underneath us (Q29).
    ///
    /// Transaction: verify identity, write a sibling temp file, close it, re-open
    /// that temp file with the same credential to prove it is readable, back up
    /// the original, rename the temp over it, then re-read the identity. A
    /// failure at any step leaves the original untouched.
    ///
    /// ## Version upgrade on write
    ///
    /// `keepass` 0.17's `dump_kdbx4` accepts **only** `KDB4(1)` — a 4.0 vault
    /// cannot be written at all (`DatabaseSaveError::UnsupportedVersion`). So any
    /// KDBX 3.1 or 4.0 vault is silently upgraded to 4.1 on first save. This is
    /// a real user-visible consequence: the file stops being openable by older
    /// clients that predate 4.1 support. The upgrade is therefore recorded in
    /// the returned error type instead of being hidden, and the caller is
    /// expected to have shown the user a warning.
    pub fn save_with_transaction(&mut self, credential: &Credential) -> CoreResult<()> {
        if !self.identity.is_current()? {
            return Err(CoreError::FileChangedExternally);
        }

        // Writing is 4.1-only upstream (see the doc comment). A KDBX 3.1 vault
        // has to be upgraded before it can be written at all.
        match &self.db.config.version {
            keepass::config::DatabaseVersion::KDB4(1) => {}
            keepass::config::DatabaseVersion::KDB4(_)
            | keepass::config::DatabaseVersion::KDB3(_) => {
                self.db.config.version = keepass::config::DatabaseVersion::KDB4(1);
            }
            other => {
                return Err(CoreError::UnsupportedVersion(format!(
                    "cannot write {other:?}"
                )));
            }
        }

        let parent = self.path.parent().unwrap_or_else(|| Path::new("."));
        let file_name = self
            .path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or(CoreError::NotFound("file name"))?;
        let tmp = parent.join(format!("{file_name}.tmp"));
        let bak = parent.join(format!("{file_name}.bak"));

        // Serialize.
        {
            let key = build_key(credential)?;
            let mut out = std::fs::File::create(&tmp).map_err(|source| CoreError::Io {
                path: tmp.clone(),
                source,
            })?;
            // Surface the real cause: a blanket "save failed" hides whether it was
            // I/O, serialization, crypto or an unsupported version.
            self.db.save(&mut out, key).map_err(|e| match e {
                keepass::db::DatabaseSaveError::Io(err) => CoreError::Io {
                    path: tmp.clone(),
                    source: err,
                },
                keepass::db::DatabaseSaveError::UnsupportedVersion => {
                    CoreError::UnsupportedVersion(format!(
                        "cannot write this version: {:?}",
                        self.db.config.version
                    ))
                }
                other => CoreError::Corrupt {
                    detail: format!("save failed: {other}"),
                },
            })?;
            out.sync_all().map_err(|source| CoreError::Io {
                path: tmp.clone(),
                source,
            })?;
        }

        // Prove the temp file opens before touching the original.
        let tmp_bytes = std::fs::read(&tmp).map_err(|source| CoreError::Io {
            path: tmp.clone(),
            source,
        })?;
        let verify_key = build_key(credential)?;
        if let Err(e) = Database::parse(&tmp_bytes, verify_key) {
            // 清理失败不得覆盖主错误：临时文件残留比丢失真实原因更可接受。
            let _ = std::fs::remove_file(&tmp);
            return Err(classify_open_error(&e));
        }

        std::fs::copy(&self.path, &bak).map_err(|source| CoreError::Io {
            path: bak.clone(),
            source,
        })?;
        std::fs::rename(&tmp, &self.path).map_err(|source| CoreError::Io {
            path: self.path.clone(),
            source,
        })?;

        self.identity = OpenedIdentity::of(&self.path)?;
        // 清理失败不得覆盖主错误：备份残留远好过报告一次假的保存失败。
        let _ = std::fs::remove_file(&bak);
        self.last_activity = Instant::now();
        Ok(())
    }

    fn write_marker(&mut self, entry: EntryId, key: &'static str, value: bool) -> CoreResult<()> {
        let id = self.kp(entry)?;
        let mut e = self.db.entry_mut(id).ok_or(CoreError::NotFound("entry"))?;
        e.custom_data.insert(
            key.to_string(),
            CustomDataItem {
                value: Some(CustomDataValue::String(value.to_string())),
                last_modification_time: None,
            },
        );
        Ok(())
    }
}

impl Vault for KeepassVault {
    fn path(&self) -> &Path {
        &self.path
    }

    fn identity(&self) -> &OpenedIdentity {
        &self.identity
    }

    fn entry_count(&self) -> usize {
        self.db.num_entries()
    }

    fn reveal(&self, entry: EntryId, field: FieldId) -> CoreResult<SecretString> {
        let id = self.kp(entry)?;
        let e = self.db.entry(id).ok_or(CoreError::NotFound("entry"))?;
        // Field id is a hash of the name, so recover the name by scanning. Entry
        // field counts are small; this is not a hot path.
        let name = e
            .fields
            .keys()
            .find(|k| field_id_for(k) == field)
            .ok_or(CoreError::NotFound("field"))?;
        let value = e.fields.get(name).ok_or(CoreError::NotFound("field"))?;
        Ok(SecretString::from(value.get().to_string()))
    }

    fn apply(&mut self, tx: VaultTransaction) -> CoreResult<ChangeSet> {
        match tx {
            VaultTransaction::SetFavorite { entry, value } => {
                self.write_marker(entry, interop::KEY_FAVORITE, value)?;
            }
            VaultTransaction::SetArchived { entry, value } => {
                self.write_marker(entry, interop::KEY_ARCHIVED, value)?;
            }
            VaultTransaction::SetTitle { entry, value } => {
                let id = self.kp(entry)?;
                let mut e = self.db.entry_mut(id).ok_or(CoreError::NotFound("entry"))?;
                e.set_unprotected(keepass::db::fields::TITLE, value);
            }
            VaultTransaction::DeleteEntry { entry } => {
                let id = self.kp(entry)?;
                let e = self.db.entry_mut(id).ok_or(CoreError::NotFound("entry"))?;
                e.remove();
            }
        }
        Ok(ChangeSet { dirty: true })
    }

    fn save(&mut self, credential: &Credential) -> CoreResult<()> {
        self.save_with_transaction(credential)
    }

    fn verify_credential(&self, credential: &Credential) -> CoreResult<bool> {
        // Full re-open of the bytes we hold. A header-only HMAC check is still an
        // open item (architecture.md §13): this is correct but slow, and the
        // caller throttles attempts via `Session::record_reauth`.
        let bytes = std::fs::read(&self.path).map_err(|source| CoreError::Io {
            path: self.path.clone(),
            source,
        })?;
        let key = build_key(credential)?;
        match Database::parse(&bytes, key) {
            Ok(_) => Ok(true),
            // `DatabaseKeyError` is not nameable (private module), so any key or
            // crypto failure means "this credential does not open this vault".
            Err(keepass::db::DatabaseOpenError::Key(_)) => Ok(false),
            Err(keepass::db::DatabaseOpenError::Cryptography(_)) => Ok(false),
            Err(e) => Err(classify_open_error(&e)),
        }
    }

    fn idle_remaining(&self, timeout: Duration) -> Duration {
        timeout.saturating_sub(self.last_activity.elapsed())
    }
}

/// Recover the original text of a value `keepass` decoded as base64.
///
/// Returns `None` when the bytes are not valid UTF-8, in which case the marker
/// really was binary and the caller must not pretend otherwise.
fn decode_misparsed_text(bytes: &[u8]) -> Option<String> {
    std::str::from_utf8(bytes).ok().map(|s| s.to_string())
}

/// The field id for a field name, exposed so callers can build a `reveal`.
pub fn field_id(name: &str) -> FieldId {
    field_id_for(name)
}
