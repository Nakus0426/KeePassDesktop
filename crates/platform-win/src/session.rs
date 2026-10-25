//! Workstation lock notification.
//!
//! v1 must lock the vault the moment the session locks, not when the idle timer
//! happens to expire. The Win32 path is `WTSRegisterSessionNotification` on the
//! app window and handling `WTS_SESSION_LOCK` — not wired yet (docs §7).

use std::sync::Mutex;

use platform_api::SessionEvents;

/// Registry of lock callbacks, kept separate from Win32 so the fan-out is
/// testable.
#[derive(Default)]
pub struct LockBroadcaster {
    callbacks: Mutex<Vec<Box<dyn Fn() + Send + Sync>>>,
}

impl LockBroadcaster {
    pub fn new() -> Self {
        Self::default()
    }

    /// Invoked by the Win32 message loop on `WTS_SESSION_LOCK`.
    pub fn broadcast(&self) {
        let cbs = self.callbacks.lock().expect("lock callback registry");
        for cb in cbs.iter() {
            cb();
        }
    }
}

impl SessionEvents for LockBroadcaster {
    fn on_lock(&self, cb: Box<dyn Fn() + Send + Sync>) {
        self.callbacks
            .lock()
            .expect("lock callback registry")
            .push(cb);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn broadcast_reaches_all_callbacks() {
        let b = LockBroadcaster::new();
        // Callbacks are `'static`, so the counter must be shared, not borrowed.
        let hits = Arc::new(AtomicUsize::new(0));

        let h1 = Arc::clone(&hits);
        b.on_lock(Box::new(move || {
            h1.fetch_add(1, Ordering::SeqCst);
        }));
        let h2 = Arc::clone(&hits);
        b.on_lock(Box::new(move || {
            h2.fetch_add(1, Ordering::SeqCst);
        }));

        b.broadcast();
        assert_eq!(hits.load(Ordering::SeqCst), 2);
    }
}
