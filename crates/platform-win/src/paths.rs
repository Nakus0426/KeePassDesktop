//! Application data locations.

use std::path::PathBuf;

use platform_api::AppPaths;

pub struct WinPaths {
    config: PathBuf,
    data: PathBuf,
}

impl WinPaths {
    pub fn new() -> Self {
        let base = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let local = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        Self {
            config: base.join("KeePassDesktop"),
            data: local.join("KeePassDesktop"),
        }
    }
}

impl Default for WinPaths {
    fn default() -> Self {
        Self::new()
    }
}

impl AppPaths for WinPaths {
    fn config_dir(&self) -> PathBuf {
        self.config.clone()
    }

    fn data_dir(&self) -> PathBuf {
        self.data.clone()
    }
}
