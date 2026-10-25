//! The storage boundary.
//!
//! Nothing above this module holds plaintext. [`Vault::reveal`] is the single
//! exit for a secret, and its signature forces the caller to say what it wants.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use secrecy::SecretString;
use sha2::{Digest, Sha256};

use crate::error::CoreResult;

/// Stable identifier for an entry. A UUID from the KDBX document.
pub type EntryId = uuid::Uuid;

/// Stable identifier for a field within an entry. Derived from the field name,
/// because KDBX keys entry fields by name, not by id.
pub type FieldId = uuid::Uuid;

/// What makes a vault openable. Zeroized on drop by the caller.
pub struct Credential {
    pub password: Option<SecretString>,
    pub key_file: Option<PathBuf>,
}

impl std::fmt::Debug for Credential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never print the password or the key file contents.
        f.debug_struct("Credential")
            .field("has_password", &self.password.is_some())
            .field("has_key_file", &self.key_file.is_some())
            .finish()
    }
}

/// Identity of the file we opened, used to refuse stale writes (Q29).
#[derive(Clone, Debug)]
pub struct OpenedIdentity {
    pub path: PathBuf,
    pub sha256: [u8; 32],
    pub size: u64,
    pub modified: Option<SystemTime>,
}

impl OpenedIdentity {
    /// Hash the file as it is now and compare with what we opened.
    pub fn is_current(&self) -> CoreResult<bool> {
        let now = Self::of(&self.path)?;
        Ok(now.sha256 == self.sha256 && now.size == self.size)
    }

    pub fn of(path: &Path) -> CoreResult<Self> {
        let bytes = std::fs::read(path).map_err(|source| crate::error::CoreError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let sha256: [u8; 32] = hasher.finalize().into();
        let meta = std::fs::metadata(path).map_err(|source| crate::error::CoreError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(Self {
            path: path.to_path_buf(),
            sha256,
            size: meta.len(),
            modified: meta.modified().ok(),
        })
    }
}

/// A change to apply to an open vault.
#[derive(Debug, Clone)]
pub enum VaultTransaction {
    SetFavorite {
        entry: crate::api::EntryId,
        value: bool,
    },
    SetArchived {
        entry: crate::api::EntryId,
        value: bool,
    },
    SetTitle {
        entry: crate::api::EntryId,
        value: String,
    },
    DeleteEntry {
        entry: crate::api::EntryId,
    },
}

/// Outcome of [`Vault::apply`], used to roll back and to mark the file dirty.
#[derive(Debug, Clone, Default)]
pub struct ChangeSet {
    pub dirty: bool,
}

/// An opened vault.
///
/// Deliberately has no `entries_with_plaintext` accessor: the reference app
/// copies every protected value into its document model, which contradicts its
/// own security doc. We do not.
pub trait Vault {
    fn path(&self) -> &Path;
    fn identity(&self) -> &OpenedIdentity;
    fn entry_count(&self) -> usize;

    /// The only way to obtain a secret. The caller drops it immediately after
    /// use; for clipboard flows it goes straight into `Clipboard::set_secret`.
    fn reveal(
        &self,
        entry: crate::api::EntryId,
        field: crate::api::FieldId,
    ) -> CoreResult<SecretString>;

    fn apply(&mut self, tx: VaultTransaction) -> CoreResult<ChangeSet>;

    /// Persist, refusing if the file changed since [`Vault::identity`] was taken.
    fn save(&mut self, credential: &Credential) -> CoreResult<()>;

    /// Verify a credential without decrypting the whole vault.
    ///
    /// Intended to validate the KDBX header HMAC only. See the open item in
    /// docs/architecture.md §13: if `keepass` does not expose a header-only
    /// check, this is implemented here directly.
    fn verify_credential(&self, credential: &Credential) -> CoreResult<bool>;

    /// How long until the session should auto-lock, given the configured idle
    /// timeout. Returns `Duration::ZERO` once expired.
    fn idle_remaining(&self, timeout: Duration) -> Duration;
}
