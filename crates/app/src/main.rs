//! Entry point.
//!
//! No command-line password argument is ever accepted: a password in `argv` is
//! visible to every process on the machine (security invariant S7).
//!
//! ## Windows subsystem
//!
//! A Rust binary defaults to the `console` subsystem, so double-clicking it opens
//! a console window alongside the app. The `windows` subsystem removes that, but
//! it also swallows stdout/stderr, so a panic would vanish. Two consequences:
//!
//! - Run `cargo run -p app --features console` when you want to watch live output.
//! - A panic logger is installed unconditionally, so a shipped build still leaves
//!   a trace at `%LOCALAPPDATA%\KeePassDesktop\panic.log`. Its output is
//!   deliberately free of vault data (S3/S7).

#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")]

use app::screens::gallery::Gallery;
use app::screens::main::MainScreen;
use app::screens::unlock::UnlockScreen;
use app::screens::welcome::{WelcomeIntent, WelcomeScreen};
use app::state::{AppState, Route};
use design_system::prelude::*;
use design_system::tokens::DARK;

const APP_NAME: &str = "KeePass Desktop";

struct Root {
    state: AppState,
    welcome: Entity<WelcomeScreen>,
    unlock: Entity<UnlockScreen>,
    main: Entity<MainScreen>,
    gallery: Entity<Gallery>,
}

impl Root {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let welcome = cx.new(|_| WelcomeScreen::new());
        let unlock = cx.new(|cx| UnlockScreen::new(window, cx));
        let main = cx.new(|_| MainScreen::new());
        let gallery = cx.new(|cx| Gallery::new(window, cx));

        // The welcome screen only emits intent; routing stays in Root.
        cx.subscribe(&welcome, |this, _, intent: &WelcomeIntent, cx| {
            this.state.route = match intent {
                WelcomeIntent::ShowGallery => Route::Gallery,
                // Both vault actions land on the unlock screen for now: the
                // file picker and vault creation are not wired yet.
                WelcomeIntent::OpenVault | WelcomeIntent::CreateVault => Route::Unlock,
            };
            // Retained mode: without this the previous frame stays on screen.
            cx.notify();
        })
        .detach();

        let mut state = AppState::boot(false);

        // Development escape hatch: `KEEPDESKTOP_START_ROUTE=gallery` boots
        // straight into the design-system page. Needed because unattended
        // screenshot verification cannot reliably synthesise a click into a
        // background window (docs/architecture.md §17). Not a product feature.
        if std::env::var("KEEPDESKTOP_START_ROUTE").as_deref() == Ok("gallery") {
            state.route = Route::Gallery;
        }

        Self {
            state,
            welcome,
            unlock,
            main,
            gallery,
        }
    }
}

impl Render for Root {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Retained-mode discipline: every route change must notify, or the
        // previous frame stays on screen. See Q26.
        cx.notify();

        let screen: AnyElement = match self.state.route {
            Route::Welcome => self.welcome.clone().into_any_element(),
            Route::Unlock => self.unlock.clone().into_any_element(),
            Route::Main => self.main.clone().into_any_element(),
            Route::Gallery => self.gallery.clone().into_any_element(),
        };

        div().size_full().bg(rgb(DARK.surface.page)).child(screen)
    }
}

/// Write panics to `%LOCALAPPDATA%\KeePassDesktop\panic.log`.
///
/// With the `windows` subsystem there is no console to print to, and a silent
/// crash is worse than a log file. Only the panic payload and location are
/// written; no vault content ever reaches this path.
fn install_panic_logger() {
    use std::io::Write;

    let dir = std::env::var_os("LOCALAPPDATA")
        .map(std::path::PathBuf::from)
        .map(|p| p.join("KeePassDesktop"));

    std::panic::set_hook(Box::new(move |info| {
        let Some(dir) = dir.as_ref() else { return };
        if std::fs::create_dir_all(dir).is_err() {
            return;
        }
        let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("panic.log"))
        else {
            return;
        };
        // 日志写入失败无法再上报，避免递归 panic 只能忽略。
        let _ = writeln!(file, "{info}");
    }));
}

fn main() {
    install_panic_logger();

    application().run(|cx: &mut App| {
        // MUST run before anything else touches a GPUI view.
        design_system::prelude::init(cx);

        let bounds = Bounds::centered(None, size(px(1180.0), px(760.0)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            // Without an explicit titlebar the OS window has no title at all.
            titlebar: Some(TitlebarOptions {
                title: Some(APP_NAME.into()),
                ..Default::default()
            }),
            // Lets the shell group our windows and label the taskbar entry.
            app_id: Some("KeePassDesktop".into()),
            window_min_size: Some(size(px(900.0), px(600.0))),
            ..Default::default()
        };

        design_system::prelude::open_window(options, cx, |window, cx| {
            cx.new(|cx| Root::new(window, cx))
        })
        .expect("open main window");

        cx.activate(true);
    });
}
