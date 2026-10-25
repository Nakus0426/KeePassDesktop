//! Single-instance guard.
//!
//! A named mutex decides who owns the session; the loser asks the winner to
//! raise its window. The Win32 wiring (`CreateMutexW`, `WM_COPYDATA`) is not
//! done yet; the ownership logic here is what the tests pin.

use std::sync::Mutex;

use platform_api::SingleInstance;

pub const MUTEX_NAME: &str = r"Local\KeePassDesktop.SingleInstance";

pub struct SharedOwnership {
    acquired: Mutex<Option<bool>>,
}

impl SharedOwnership {
    pub fn new() -> Self {
        Self {
            acquired: Mutex::new(None),
        }
    }

    /// Record the outcome of the Win32 mutex attempt. Idempotent: a second call
    /// keeps the first answer.
    pub fn record(&self, acquired: bool) -> bool {
        let mut slot = self.acquired.lock().expect("single instance slot");
        *slot.get_or_insert(acquired)
    }
}

impl Default for SharedOwnership {
    fn default() -> Self {
        Self::new()
    }
}

impl SingleInstance for SharedOwnership {
    fn acquire(&self) -> bool {
        let mut slot = self.acquired.lock().expect("single instance slot");
        *slot.get_or_insert(true)
    }

    fn focus_existing(&self) {
        // No-op until the WM_COPYDATA channel exists.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ownership_is_sticky() {
        let s = SharedOwnership::new();
        assert!(s.acquire());
        s.record(false);
        assert!(s.acquire(), "first answer must win");
    }
}
