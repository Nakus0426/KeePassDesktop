//! Domain types.
//!
//! The rule: no plaintext secret lives in these structs. A password exists only
//! as the return value of `Vault::reveal`.

use uuid::Uuid;

pub type EntryId = Uuid;
pub type FieldId = Uuid;
pub type GroupId = Uuid;

/// Non-secret field kinds, matching the wire values in `keepass_core::interop`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldKind {
    Text,
    Url,
    Email,
    Phone,
    Address,
    Date,
    Password,
    SshKeys,
}

/// A row in the entry list. Everything here is safe to render without a reveal.
#[derive(Clone, Debug)]
pub struct EntrySummary {
    pub id: EntryId,
    pub title: String,
    pub username: String,
    pub url: String,
    pub group_path: String,
    pub tags: Vec<String>,
    pub template: Option<crate::template::Template>,
    pub favorite: bool,
    pub archived: bool,
    pub has_totp: bool,
    pub attachment_count: usize,
    pub history_count: usize,
}

/// A node in the group tree.
#[derive(Clone, Debug)]
pub struct GroupNode {
    pub id: GroupId,
    pub name: String,
    pub children: Vec<GroupNode>,
    pub entry_count: usize,
}

/// Vault-level metadata that is safe to show before unlock.
#[derive(Clone, Debug, Default)]
pub struct VaultMeta {
    pub name: String,
    pub description: Option<String>,
    pub recycle_bin: Option<GroupId>,
    pub custom_icon_count: usize,
}

/// A field definition without its value.
#[derive(Clone, Debug)]
pub struct FieldDescriptor {
    pub id: FieldId,
    pub name: String,
    pub kind: FieldKind,
    pub protected: bool,
}

/// A custom icon reference.
#[derive(Clone, Debug)]
pub struct IconRef {
    pub uuid: Uuid,
}
