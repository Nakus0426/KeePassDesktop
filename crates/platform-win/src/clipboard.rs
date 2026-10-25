//! TTL-bounded clipboard writes.
//!
//! The sequencing rule is the whole point: a write carries a monotonically
//! increasing token, and expiry clears the clipboard **only if the token is
//! still the current one**. Without that check, a timer that fires late destroys
//! whatever the user copied afterwards.
//!
//! [`SeqClipboard`] implements the state machine and is unit-tested. The Win32
//! side (`CF_UNICODETEXT` write, plus `AddClipboardFormatListener` to detect
//! external overwrites) plugs in through [`ClipboardBackend`] and is not wired
//! yet — see docs/architecture.md §7.

use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use platform_api::Clipboard;
use secrecy::SecretString;

/// The OS clipboard. Implemented by the Win32 backend.
pub trait ClipboardBackend: Send + Sync {
    fn write_text(&self, text: &str);
    fn clear(&self);
}

/// Owns write ordering and expiry decisions.
pub struct SeqClipboard<B: ClipboardBackend> {
    backend: B,
    seq: AtomicU64,
    /// Token of the write that currently owns the clipboard.
    owner: Mutex<u64>,
}

impl<B: ClipboardBackend> SeqClipboard<B> {
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            seq: AtomicU64::new(0),
            owner: Mutex::new(0),
        }
    }

    /// Clear only if `token` is still the owner. Returns whether it cleared.
    pub fn expire(&self, token: u64) -> bool {
        let mut owner = self.owner.lock().expect("clipboard owner lock");
        if *owner != token || token == 0 {
            return false;
        }
        self.backend.clear();
        *owner = 0;
        true
    }

    /// The token of the current owner, for the caller's timer.
    pub fn current_token(&self) -> u64 {
        *self.owner.lock().expect("clipboard owner lock")
    }
}

impl<B: ClipboardBackend> Clipboard for SeqClipboard<B> {
    fn set_secret(&self, secret: SecretString, _ttl: Duration) {
        use secrecy::ExposeSecret;
        let token = self.seq.fetch_add(1, Ordering::SeqCst) + 1;
        self.backend.write_text(secret.expose_secret());
        *self.owner.lock().expect("clipboard owner lock") = token;
        // The caller owns the timer and calls `expire(token)` when it fires.
    }

    fn clear(&self) {
        let mut owner = self.owner.lock().expect("clipboard owner lock");
        self.backend.clear();
        *owner = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering as O};

    struct Counting {
        writes: AtomicUsize,
        clears: AtomicUsize,
        last: Mutex<String>,
    }

    impl Counting {
        fn new() -> Self {
            Self {
                writes: AtomicUsize::new(0),
                clears: AtomicUsize::new(0),
                last: Mutex::new(String::new()),
            }
        }
    }

    impl ClipboardBackend for Counting {
        fn write_text(&self, text: &str) {
            self.writes.fetch_add(1, O::SeqCst);
            *self.last.lock().unwrap() = text.to_string();
        }
        fn clear(&self) {
            self.clears.fetch_add(1, O::SeqCst);
            self.last.lock().unwrap().clear();
        }
    }

    #[test]
    fn stale_timer_does_not_clear_new_value() {
        let cb = SeqClipboard::new(Counting::new());
        cb.set_secret(SecretString::from("first"), Duration::from_secs(30));
        let stale = cb.current_token();
        cb.set_secret(SecretString::from("second"), Duration::from_secs(30));

        assert!(
            !cb.expire(stale),
            "stale token must not clear the clipboard"
        );
        assert!(cb.expire(cb.current_token()));
    }

    #[test]
    fn expiry_is_idempotent() {
        let cb = SeqClipboard::new(Counting::new());
        cb.set_secret(SecretString::from("x"), Duration::from_secs(30));
        let token = cb.current_token();
        assert!(cb.expire(token));
        assert!(!cb.expire(token));
    }
}
