use crate::i18n::{Language, Strings};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Never takes focus back. Only records who took it.
    Watch,
    /// Takes focus back while you are typing, and from anything on the block list.
    Guard,
    /// Takes focus back from everything that is not on the allow list.
    Strict,
}

impl Mode {
    pub fn label(self, s: &'static Strings) -> &'static str {
        match self {
            Mode::Watch => s.mode_watch,
            Mode::Guard => s.mode_guard,
            Mode::Strict => s.mode_strict,
        }
    }

    pub fn hint(self, s: &'static Strings) -> &'static str {
        match self {
            Mode::Watch => s.mode_watch_hint,
            Mode::Guard => s.mode_guard_hint,
            Mode::Strict => s.mode_strict_hint,
        }
    }

    pub const ALL: [Mode; 3] = [Mode::Watch, Mode::Guard, Mode::Strict];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub mode: Mode,
    pub language: Language,
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
            language: Language::System,
            typing_window_ms: 1500,
            click_grace_ms: 400,
            blocklist: Vec::new(),
            // Sorted, because normalize() sorts, and a config that reorders itself
            // on first read looks like something went wrong.
            allowlist: vec![
                "applicationframehost.exe".into(),
                "consent.exe".into(),
                "credentialuibroker.exe".into(),
                "explorer.exe".into(),
                "lockapp.exe".into(),
                "searchhost.exe".into(),
                "shellexperiencehost.exe".into(),
                "startmenuexperiencehost.exe".into(),
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
        Self::load_from(&config_path())
    }

    pub fn load_from(path: &Path) -> Self {
        let Ok(text) = fs::read_to_string(path) else {
            let fresh = Config::default();
            let _ = fresh.save_to(path);
            return fresh;
        };
        match toml::from_str::<Config>(&text) {
            Ok(mut cfg) => {
                cfg.normalize();
                cfg
            }
            Err(_) => {
                // Keep whatever the user had; they may have meant something by it.
                let _ = fs::rename(path, path.with_extension("toml.broken"));
                let fresh = Config::default();
                let _ = fresh.save_to(path);
                fresh
            }
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        self.save_to(&config_path())
    }

    pub fn save_to(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(path, text)
    }

    pub fn strings(&self) -> &'static Strings {
        self.language.strings()
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
        let exe = exe.trim().to_ascii_lowercase();
        if exe.is_empty() {
            return;
        }
        self.allowlist.retain(|n| *n != exe);
        if !self.blocklist.contains(&exe) {
            self.blocklist.push(exe);
            self.blocklist.sort();
        }
    }

    pub fn allow(&mut self, exe: &str) {
        let exe = exe.trim().to_ascii_lowercase();
        if exe.is_empty() {
            return;
        }
        self.blocklist.retain(|n| *n != exe);
        if !self.allowlist.contains(&exe) {
            self.allowlist.push(exe);
            self.allowlist.sort();
        }
    }

    pub fn forget(&mut self, exe: &str) {
        let exe = exe.trim().to_ascii_lowercase();
        self.blocklist.retain(|n| *n != exe);
        self.allowlist.retain(|n| *n != exe);
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

/// When the settings file last changed, so the running copy can pick up an edit
/// made in the panel, or in a text editor, without being restarted.
pub fn changed_at() -> Option<SystemTime> {
    fs::metadata(config_path()).ok()?.modified().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One directory for every test run, so a test suite does not litter the
    /// temporary folder with a directory per case per run.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("wsmf-tests");
        let _ = fs::create_dir_all(&dir);
        dir.join(format!("{name}-{}.toml", std::process::id()))
    }

    #[test]
    fn a_saved_config_reads_back_the_same() {
        let path = scratch("roundtrip");
        let mut original = Config {
            mode: Mode::Guard,
            language: Language::Turkish,
            ..Config::default()
        };
        original.block("teams.exe");
        original.save_to(&path).unwrap();

        let loaded = Config::load_from(&path);
        assert_eq!(loaded, original);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn a_missing_file_gives_defaults_and_writes_one() {
        let path = scratch("missing");
        let _ = fs::remove_file(&path);
        let cfg = Config::load_from(&path);
        assert_eq!(cfg.mode, Mode::Watch);
        assert!(path.exists(), "a fresh config should be written out");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn a_broken_file_is_kept_aside_rather_than_overwritten() {
        let path = scratch("broken");
        fs::write(&path, "this is not toml at all {{{").unwrap();
        let cfg = Config::load_from(&path);
        assert_eq!(cfg.mode, Mode::Watch);
        let kept = path.with_extension("toml.broken");
        assert!(kept.exists(), "the unreadable file should be preserved");
        let _ = fs::remove_file(path);
        let _ = fs::remove_file(kept);
    }

    #[test]
    fn a_partial_file_keeps_what_it_says_and_defaults_the_rest() {
        let path = scratch("partial");
        fs::write(&path, "mode = \"strict\"\n").unwrap();
        let cfg = Config::load_from(&path);
        assert_eq!(cfg.mode, Mode::Strict);
        assert_eq!(cfg.typing_window_ms, Config::default().typing_window_ms);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn absurd_numbers_are_pulled_back_into_range() {
        let path = scratch("clamp");
        fs::write(
            &path,
            "typing_window_ms = 999999999\nclick_grace_ms = 0\nmax_restores = 0\nrestore_window_secs = 100000\n",
        )
        .unwrap();
        let cfg = Config::load_from(&path);
        assert_eq!(cfg.typing_window_ms, 30_000);
        assert_eq!(cfg.click_grace_ms, 50);
        assert_eq!(cfg.max_restores, 1);
        assert_eq!(cfg.restore_window_secs, 600);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn hand_written_names_are_tidied_up() {
        let path = scratch("tidy");
        fs::write(
            &path,
            "blocklist = [\"  Teams.EXE \", \"teams.exe\", \"\", \"  \"]\n",
        )
        .unwrap();
        let cfg = Config::load_from(&path);
        assert_eq!(cfg.blocklist, vec!["teams.exe"]);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn blocking_something_removes_it_from_the_allow_list() {
        let mut cfg = Config::default();
        cfg.allow("teams.exe");
        cfg.block("teams.exe");
        assert!(cfg.is_blocked("teams.exe"));
        assert!(!cfg.is_allowed("teams.exe"));
    }

    #[test]
    fn forgetting_takes_it_out_of_both() {
        let mut cfg = Config::default();
        cfg.block("teams.exe");
        cfg.forget("teams.exe");
        assert!(!cfg.is_blocked("teams.exe"));
        assert!(!cfg.is_allowed("teams.exe"));
    }

    #[test]
    fn adding_the_same_name_twice_does_not_duplicate_it() {
        let mut cfg = Config::default();
        cfg.block("Teams.exe");
        cfg.block("teams.exe");
        assert_eq!(
            cfg.blocklist.iter().filter(|n| *n == "teams.exe").count(),
            1
        );
    }

    #[test]
    fn an_empty_name_is_not_a_rule() {
        let mut cfg = Config::default();
        let before = cfg.blocklist.len();
        cfg.block("   ");
        assert_eq!(cfg.blocklist.len(), before);
    }
}
