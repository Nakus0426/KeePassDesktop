//! Application state machine.
//!
//! ```text
//! Boot -> Welcome (no recent vault) --\
//!      -> Unlock  (recent vault)  ---+--> Unlocked --> Locked
//! ```
//!
//! Navigation deliberately diverges from the reference client: it opens entry
//! detail as a bottom sheet over a tab bar, which suits a phone. A desktop window
//! is three columns (sidebar / list / detail), so detail is a pane, not a route.

use std::time::Duration;

use vault_model::session::Session;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Welcome,
    Unlock,
    Main,
    /// Design-system gallery. Development only: it renders every control in every
    /// state so the visual result can be judged without a real vault.
    Gallery,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Overlay {
    None,
    Search,
    Generator,
    Settings,
    EntryEditor { is_new: bool },
}

pub struct AppState {
    pub route: Route,
    pub overlay: Overlay,
    pub session: Session,
    /// Set when a save was refused because the file changed (Q29).
    pub stale_file: bool,
    pub theme: Theme,
    pub clipboard_ttl: Duration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}

impl AppState {
    pub fn boot(has_recent_vault: bool) -> Self {
        Self {
            route: if has_recent_vault {
                Route::Unlock
            } else {
                Route::Welcome
            },
            overlay: Overlay::None,
            session: Session::new(vault_model::session::DEFAULT_IDLE_TIMEOUT),
            stale_file: false,
            theme: Theme::Dark,
            clipboard_ttl: Duration::from_secs(30),
        }
    }

    /// Any interaction defers the idle lock and closes transient overlays on
    /// lock.
    pub fn on_activity(&mut self) {
        self.session.touch();
    }

    /// Called by the idle timer and by the workstation-lock event.
    pub fn lock(&mut self) {
        self.session.lock();
        self.route = Route::Unlock;
        self.overlay = Overlay::None;
    }

    /// After a successful unlock.
    pub fn on_unlocked(&mut self) {
        self.route = Route::Main;
        self.stale_file = false;
        self.session.touch();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_without_recent_vault_goes_to_welcome() {
        assert_eq!(AppState::boot(false).route, Route::Welcome);
    }

    #[test]
    fn boot_with_recent_vault_goes_to_unlock() {
        assert_eq!(AppState::boot(true).route, Route::Unlock);
    }

    #[test]
    fn lock_closes_overlays() {
        let mut s = AppState::boot(true);
        s.route = Route::Main;
        s.overlay = Overlay::Search;
        s.lock();
        assert_eq!(s.route, Route::Unlock);
        assert_eq!(s.overlay, Overlay::None);
    }
}
