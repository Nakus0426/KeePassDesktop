//! # keepass-core
//!
//! The only crate allowed to hold plaintext key material or touch a `.kdbx`
//! file on disk. Everything above this crate talks to the traits in [`api`].
//!
//! ## Hard rules
//!
//! 1. No plaintext secret ever crosses this boundary as a `String`. Secrets
//!    leave as [`secrecy::SecretString`] (or [`zeroize::Zeroizing`] bytes) and
//!    the caller decides when to drop them.
//! 2. Every disk write is a transaction: hash the file we opened, re-hash
//!    before writing, refuse on mismatch, write to a sibling temp file,
//!    re-open and fully unlock that temp file, then rename over the original
//!    with the original kept as `.bak` until the rename is confirmed.
//! 3. Errors are classified at the boundary so the UI can offer a recovery
//!    action. Corruption, unsupported version, bad signature and bad
//!    credential are four different errors, never one "unlock failed".
//!
//! ## Interop obligations
//!
//! Vault files are shared with the HarmonyOS client `KeePassHomo`. This crate
//! must read and write the private markers listed in [`interop`].

pub mod adapter;
pub mod api;
pub mod error;
pub mod interop;

pub use adapter::KeepassVault;
pub use api::{Credential, Vault, VaultTransaction};
pub use error::{CoreError, CoreResult};
