//! # vault-model
//!
//! Domain types and use cases. Knows nothing about GPUI or about the `keepass`
//! crate; it talks to [`keepass_core::Vault`] only.
//!
//! ## The one rule that matters
//!
//! [`EntrySummary`] never carries plaintext secrets. A password only exists as
//! the return value of an explicit `reveal` or `copy` call, and is dropped by
//! the caller immediately. This is the opposite of the reference HarmonyOS
//! implementation, which copies every protected field into its document model.

pub mod entry;
pub mod generator;
pub mod search;
pub mod session;
pub mod strength;
pub mod template;

pub use entry::{EntryId, EntrySummary, FieldId, FieldKind, GroupNode, VaultMeta};
pub use session::Session;
pub use template::Template;
