//! Vault session: lock state, idle timer, re-authentication throttling.
//!
//! The reference client has no attempt counter and no rate limit on
//! re-authentication, and its re-auth is a full vault decrypt. We throttle and
//! use a header-only check.

use std::time::{Duration, Instant};

use keepass_core::api::Vault;

/// Default idle timeout: 5 minutes, configurable.
pub const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(5 * 60);

/// Re-auth policy: three failures lock the prompt for 30 seconds; ten failures
/// force a full unlock.
pub const REAUTH_FREE_ATTEMPTS: u32 = 3;
pub const REAUTH_COOLDOWN: Duration = Duration::from_secs(30);
pub const REAUTH_FORCE_RELOCK_ATTEMPTS: u32 = 10;

pub struct Session {
    vault: Option<Box<dyn Vault>>,
    idle_timeout: Duration,
    last_activity: Instant,
    reauth_failures: u32,
    reauth_locked_until: Option<Instant>,
}

impl Session {
    pub fn new(idle_timeout: Duration) -> Self {
        Self {
            vault: None,
            idle_timeout,
            last_activity: Instant::now(),
            reauth_failures: 0,
            reauth_locked_until: None,
        }
    }

    pub fn is_unlocked(&self) -> bool {
        self.vault.is_some()
    }

    pub fn vault(&self) -> Option<&dyn Vault> {
        self.vault.as_deref()
    }

    /// Called by every user interaction that should defer the idle lock.
    pub fn touch(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Whether the idle timeout has elapsed.
    pub fn should_auto_lock(&self) -> bool {
        self.last_activity.elapsed() >= self.idle_timeout
    }

    /// Force a lock. Drops the vault, which zeroizes its key material.
    pub fn lock(&mut self) {
        self.vault = None;
        self.reauth_failures = 0;
        self.reauth_locked_until = None;
    }

    /// Remaining cooldown before a re-auth prompt may be shown.
    pub fn reauth_cooldown(&self) -> Option<Duration> {
        let until = self.reauth_locked_until?;
        let now = Instant::now();
        (until > now).then(|| until - now)
    }

    /// Record a re-auth result. Returns `true` when the vault must be fully
    /// re-opened because the failure budget is exhausted.
    pub fn record_reauth(&mut self, ok: bool) -> bool {
        if ok {
            self.reauth_failures = 0;
            self.reauth_locked_until = None;
            self.touch();
            return false;
        }

        self.reauth_failures += 1;
        if self.reauth_failures.is_multiple_of(REAUTH_FREE_ATTEMPTS) {
            self.reauth_locked_until = Some(Instant::now() + REAUTH_COOLDOWN);
        }
        self.reauth_failures >= REAUTH_FORCE_RELOCK_ATTEMPTS
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn three_failures_start_cooldown() {
        let mut s = Session::new(Duration::from_secs(300));
        assert!(!s.record_reauth(false));
        assert!(!s.record_reauth(false));
        s.record_reauth(false);
        assert!(s.reauth_cooldown().is_some());
    }

    #[test]
    fn ten_failures_force_relock() {
        let mut s = Session::new(Duration::from_secs(300));
        let mut forced = false;
        for _ in 0..REAUTH_FORCE_RELOCK_ATTEMPTS {
            forced = s.record_reauth(false);
        }
        assert!(forced);
    }

    #[test]
    fn success_resets_budget() {
        let mut s = Session::new(Duration::from_secs(300));
        s.record_reauth(false);
        s.record_reauth(false);
        assert!(!s.record_reauth(true));
        assert!(s.reauth_cooldown().is_none());
    }

    #[test]
    fn idle_timeout_eligible() {
        let mut s = Session::new(Duration::from_millis(1));
        std::thread::sleep(Duration::from_millis(5));
        assert!(s.should_auto_lock());
        s.touch();
        assert!(!s.should_auto_lock());
    }
}
