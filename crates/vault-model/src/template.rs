//! Entry templates. Field names are a wire format shared with `KeePassHomo`; see
//! `keepass_core::interop`.

use keepass_core::interop::{LayoutType, template_fields};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Template {
    Login,
    SecureNote,
    CreditCard,
    SshKey,
    Attachment,
    Password,
}

impl Template {
    pub const ALL: [Template; 6] = [
        Template::Login,
        Template::SecureNote,
        Template::CreditCard,
        Template::SshKey,
        Template::Attachment,
        Template::Password,
    ];

    pub const fn layout(self) -> LayoutType {
        match self {
            Self::Login => LayoutType::Login,
            Self::SecureNote => LayoutType::SecureNote,
            Self::CreditCard => LayoutType::CreditCard,
            Self::SshKey => LayoutType::SshKey,
            Self::Attachment => LayoutType::Attachment,
            Self::Password => LayoutType::Password,
        }
    }

    pub fn from_layout(layout: LayoutType) -> Self {
        match layout {
            LayoutType::Login => Self::Login,
            LayoutType::SecureNote => Self::SecureNote,
            LayoutType::CreditCard => Self::CreditCard,
            LayoutType::SshKey => Self::SshKey,
            LayoutType::Attachment => Self::Attachment,
            LayoutType::Password => Self::Password,
        }
    }

    /// Custom (non-reserved) fields this template pre-fills, in display order.
    pub const fn custom_fields(self) -> &'static [&'static str] {
        match self {
            Self::CreditCard => template_fields::CREDIT_CARD,
            Self::SshKey => &[keepass_core::interop::FIELD_SSH_KEYS],
            _ => &[],
        }
    }

    /// Whether `Notes` is part of this template.
    pub const fn has_notes(self) -> bool {
        !matches!(self, Self::Attachment)
    }
}
