//! Error classification at the boundary.
//!
//! The UI never displays a raw error string. Each variant maps to a distinct
//! recovery action, so "unlock failed" can never mean four different things.

use std::path::PathBuf;

use thiserror::Error;

pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Debug, Error)]
pub enum CoreError {
    /// Magic signature did not match either KDBX signature.
    #[error("not a KDBX file")]
    NotAKdbxFile,

    /// Valid KDBX, but a version this build does not read.
    #[error("unsupported KDBX version: {0}")]
    UnsupportedVersion(String),

    /// Structurally broken, or an HMAC/authentication block did not verify.
    ///
    /// `detail` is a short human-readable reason. It is `String` rather than
    /// `&'static str` so upstream error text can be carried without leaking
    /// memory via `Box::leak`; the UI never shows it verbatim, only `log_code()`.
    #[error("vault is corrupt: {detail}")]
    Corrupt { detail: String },

    /// Well-formed vault, wrong password and/or key file.
    #[error("incorrect credentials")]
    BadCredential,

    /// The file changed on disk since we opened it (see Q29).
    #[error("vault file changed on disk since it was opened")]
    FileChangedExternally,

    #[error("io error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// No vault is open, or it is locked.
    #[error("vault is locked")]
    Locked,

    /// The requested item is gone (concurrent edit, deleted group).
    #[error("not found: {0}")]
    NotFound(&'static str),
}

impl CoreError {
    /// Stable code for logs. Never includes user data.
    pub fn log_code(&self) -> &'static str {
        match self {
            Self::NotAKdbxFile => "kdbx.signature",
            Self::UnsupportedVersion(_) => "kdbx.version",
            Self::Corrupt { .. } => "kdbx.corrupt",
            Self::BadCredential => "kdbx.credential",
            Self::FileChangedExternally => "kdbx.stale",
            Self::Io { .. } => "io",
            Self::Locked => "session.locked",
            Self::NotFound(_) => "model.not_found",
        }
    }
}
