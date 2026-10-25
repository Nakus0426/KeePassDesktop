//! # platform-api
//!
//! Traits for the four OS integrations v1 needs. Windows lives in
//! `platform-win`; adding macOS or Linux later means a new implementation crate,
//! not a change here.
//!
//! Tray and autotype are v2; their trait shapes are not written yet on purpose.

use std::path::PathBuf;
use std::time::Duration;

use secrecy::SecretString;

/// Clipboard writes are TTL-bounded and self-invalidating.
///
/// A write carries a sequence number. When the timer fires, the payload is
/// cleared **only if the sequence number still matches**, so a value the user
/// copied afterwards is never destroyed.
pub trait Clipboard {
    fn set_secret(&self, secret: SecretString, ttl: Duration);
    fn clear(&self);
}

/// OS session events that must force an immediate lock.
pub trait SessionEvents {
    /// Fires on workstation lock. v1 requires an immediate vault lock, not a
    /// timer-based one.
    fn on_lock(&self, cb: Box<dyn Fn() + Send + Sync>);
}

/// One instance per user session; a second launch focuses the first.
pub trait SingleInstance {
    fn acquire(&self) -> bool;
    fn focus_existing(&self);
}

/// Application data locations.
pub trait AppPaths {
    fn config_dir(&self) -> PathBuf;
    fn data_dir(&self) -> PathBuf;
}
