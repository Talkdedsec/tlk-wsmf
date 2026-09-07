use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Never takes focus back. Only records who took it.
    Watch,
    /// Takes focus back while you are typing, and from anything on the blocklist.
    Guard,
    /// Takes focus back from everything that is not on the allowlist.
    Strict,
}

impl Mode {
    pub fn label(self) -> &'static str {
        match self {
            Mode::Watch => "Watch only",
            Mode::Guard => "Guard",
            Mode::Strict => "Strict",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub mode: Mode,
    /// A focus change this soon after any input, with no click, means you were typing.
    pub typing_window_ms: u64,
    /// A click this recent means you opened the window yourself. Never fight the user.
    pub click_grace_ms: u64,
    /// Executable names, lower case, no path. Always taken back in guard mode.
    pub blocklist: Vec<String>,
    /// Executable names that are never taken back, in any mode.
    pub allowlist: Vec<String>,
    /// Flash the thief in the taskbar after taking focus back, the way Windows does.
    pub flash_thief: bool,
    /// Stop fighting an application that keeps grabbing focus, and say so in the log.
    pub max_restores: u32,
    pub restore_window_secs: u64,
    pub log_to_file: bool,
    /// Also record the focus changes that were left alone, with the reason why.
    /// Off by default, because most of them are just you using your computer.
    pub record_everything: bool,
    pub start_with_windows: bool,
    /// Value written to the system foreground lock timeout, in milliseconds.
    /// Windows ships with 200000; installers routinely set it to 0.
    pub foreground_lock_timeout_ms: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode::Watch,
            typing_window_ms: 1500,
            click_grace_ms: 400,
            blocklist: Vec::new(),
            allowlist: vec![
                "explorer.exe".into(),
                "applicationframehost.exe".into(),
                "shellexperiencehost.exe".into(),
                "searchhost.exe".into(),
                "startmenuexperiencehost.exe".into(),
                "lockapp.exe".into(),
                "consent.exe".into(),
                "credentialuibroker.exe".into(),
                "wsmf.exe".into(),
            ],
            flash_thief: true,
            max_restores: 3,
            restore_window_secs: 10,
            log_to_file: true,
            record_everything: false,
            start_with_windows: false,
            foreground_lock_timeout_ms: 200_000,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();
        let Ok(text) = fs::read_to_string(&path) else {
            let fresh = Config::default();
            let _ = fresh.save();
            return fresh;
        };
        match toml::from_str::<Config>(&text) {
            Ok(mut cfg) => {
                cfg.normalize();
                cfg
            }
            Err(_) => {
                let backup = path.with_extension("toml.broken");
                let _ = fs::rename(&path, backup);
                let fresh = Config::default();
                let _ = fresh.save();
                fresh
            }
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(path, text)
    }

    fn normalize(&mut self) {
        for name in self.blocklist.iter_mut().chain(self.allowlist.iter_mut()) {
            *name = name.trim().to_ascii_lowercase();
        }
        self.blocklist.retain(|n| !n.is_empty());
        self.allowlist.retain(|n| !n.is_empty());
        self.blocklist.sort();
        self.blocklist.dedup();
        self.allowlist.sort();
        self.allowlist.dedup();
        self.typing_window_ms = self.typing_window_ms.clamp(100, 30_000);
        self.click_grace_ms = self.click_grace_ms.clamp(50, 5_000);
        self.max_restores = self.max_restores.clamp(1, 50);
        self.restore_window_secs = self.restore_window_secs.clamp(1, 600);
    }

    pub fn is_blocked(&self, exe: &str) -> bool {
        self.blocklist.iter().any(|n| n == exe)
    }

    pub fn is_allowed(&self, exe: &str) -> bool {
        self.allowlist.iter().any(|n| n == exe)
    }

    pub fn block(&mut self, exe: &str) {
        let exe = exe.to_ascii_lowercase();
        self.allowlist.retain(|n| *n != exe);
        if !self.blocklist.contains(&exe) {
            self.blocklist.push(exe);
            self.blocklist.sort();
        }
        let _ = self.save();
    }

    pub fn allow(&mut self, exe: &str) {
        let exe = exe.to_ascii_lowercase();
        self.blocklist.retain(|n| *n != exe);
        if !self.allowlist.contains(&exe) {
            self.allowlist.push(exe);
            self.allowlist.sort();
        }
        let _ = self.save();
    }

    pub fn forget(&mut self, exe: &str) {
        let exe = exe.to_ascii_lowercase();
        self.blocklist.retain(|n| *n != exe);
        self.allowlist.retain(|n| *n != exe);
        let _ = self.save();
    }
}

pub fn data_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("wsmf")
}

pub fn config_path() -> PathBuf {
    data_dir().join("config.toml")
}

pub fn log_path() -> PathBuf {
    data_dir().join("focus.log")
}
