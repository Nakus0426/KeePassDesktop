//! Interop contract with the HarmonyOS client `KeePassHomo`.
//!
//! Vault files are shared between this app and `KeePassHomo` (repo HEAD
//! `d91b5f6`). The strings in this module are a wire format: changing one breaks
//! the other client. They are pinned by tests in `tests/interop.rs`.
//!
//! ## What the other client writes
//!
//! All private state is **entry-level `customData`**. Database-level
//! (`meta.customData`) and group-level writes do not happen there at all, so
//! anything we put there is invisible to it, and anything a third party puts
//! there must survive our saves untouched.
//!
//! ## The failure mode that matters
//!
//! Its save path deletes every entry field absent from its in-memory model and
//! rewrites the whole file. So a concurrent edit from this app is lost
//! last-writer-wins. See Q29: we refuse to save when the file changed underneath
//! us.

/// Prefix every private `customData` key on an entry carries.
pub const META_KEY_PREFIX: &str = "KeePassHomo.";

/// `"true"` / `"false"`.
pub const KEY_FAVORITE: &str = "KeePassHomo.Favorite";

/// `"true"` / `"false"`.
pub const KEY_ARCHIVED: &str = "KeePassHomo.Archived";

/// JSON object mapping field name to kind: `{"<field>":"<kind>"}`.
///
/// Only non-`text` fields are stored. The whole key is deleted when the map is
/// empty. Kinds are the lowercase values of [`FieldKindWire`].
pub const KEY_FIELD_TYPES: &str = "KeePassHomo.FieldTypes";

/// Deliberately **not** prefixed: the other client treats this as a
/// cross-client draft key name. PascalCase template identity.
pub const KEY_LAYOUT_TYPE: &str = "KPEX_LayoutType";

/// Dead code in the other client (no runtime caller). **Never write this.**
pub const KEY_VALIDATION: &str = "KeePassHomo.Validation";

/// Protected field holding the SSH key list as JSON.
pub const FIELD_SSH_KEYS: &str = "SshKeys";

/// Every private entry custom-data key we write. `KEY_VALIDATION` is absent on
/// purpose.
pub const WRITTEN_CUSTOM_DATA_KEYS: &[&str] =
    &[KEY_FAVORITE, KEY_ARCHIVED, KEY_FIELD_TYPES, KEY_LAYOUT_TYPE];

/// Field-type values as they appear on the wire, lowercase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldKindWire {
    Url,
    Email,
    Phone,
    Address,
    Date,
    Password,
    Custom,
}

impl FieldKindWire {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Url => "url",
            Self::Email => "email",
            Self::Phone => "phone",
            Self::Address => "address",
            Self::Date => "date",
            Self::Password => "password",
            Self::Custom => "custom",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "url" => Self::Url,
            "email" => Self::Email,
            "phone" => Self::Phone,
            "address" => Self::Address,
            "date" => Self::Date,
            "password" => Self::Password,
            "custom" => Self::Custom,
            _ => return None,
        })
    }
}

/// Template identity written to [`KEY_LAYOUT_TYPE`], PascalCase, create-only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutType {
    Login,
    SecureNote,
    CreditCard,
    SshKey,
    Attachment,
    Password,
}

impl LayoutType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Login => "Login",
            Self::SecureNote => "SecureNote",
            Self::CreditCard => "CreditCard",
            Self::SshKey => "SshKey",
            Self::Attachment => "Attachment",
            Self::Password => "Password",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "Login" => Self::Login,
            "SecureNote" => Self::SecureNote,
            "CreditCard" => Self::CreditCard,
            "SshKey" => Self::SshKey,
            "Attachment" => Self::Attachment,
            "Password" => Self::Password,
            _ => return None,
        })
    }
}

/// Field names each template pre-fills. These strings are part of the wire
/// format: a typo here means the other client renders the entry as a generic
/// Login.
pub mod template_fields {
    /// Reserved names, never renamed.
    pub const RESERVED: &[&str] = &["Title", "UserName", "Password", "URL", "Notes"];

    pub const CREDIT_CARD: &[&str] = &[
        "CardNumber",
        "Issuer",
        "CardType",
        "CardHolderName",
        "Expiry",
    ];

    /// Options the other client offers for `CardType`.
    pub const CARD_TYPES: &[&str] = &[
        "银联",
        "Visa",
        "Mastercard",
        "American Express",
        "JCB",
        "Discover",
    ];
}

/// KeePass ignores `customIcon` unless `entry.icon` is at least this value.
pub const CUSTOM_ICON_ID: u16 = 69;

/// Shape of the [`FIELD_SSH_KEYS`] JSON payload.
pub const SSH_KEYS_VERSION: u32 = 1;
