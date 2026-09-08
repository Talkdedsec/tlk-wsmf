use crate::config::{self, Config};
use crate::input::InputWatch;
use crate::journal::Journal;
use crate::window::WindowInfo;
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::HWINEVENTHOOK;

pub struct App {
    pub cfg: Config,
    pub journal: Journal,
    pub input: InputWatch,
    /// The window focus should go back to. Never the thief.
    pub previous: Option<WindowInfo>,
    /// A foreground change we caused ourselves, to be ignored when it arrives.
    pub expecting: Option<HWND>,
    pub strikes: HashMap<String, Strike>,
    pub hook: HWINEVENTHOOK,
    pub tray_window: HWND,
    pub viewer: HWND,
    pub paused_until: Option<Instant>,
    /// Who we last took focus from, and when. An application opening often puts up
    /// two or three windows in a few milliseconds; taking focus back from each of
    /// them in turn is a flicker, not a defence.
    pub last_restore: Option<(u32, Instant)>,
    pub ticks: u64,
    /// What the settings file looked like when we last read or wrote it, so an
    /// edit from the panel is picked up and our own save is not read back.
    config_stamp: Option<SystemTime>,
}

pub struct Strike {
    pub count: u32,
    pub since: Instant,
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

pub fn install(app: App) {
    APP.with(|slot| *slot.borrow_mut() = Some(app));
}

/// Every callback in this program funnels through here. Reentrancy is real:
/// restoring focus fires the very hook that called us, so a nested borrow would
/// panic. Returning None on a busy cell is the whole point.
pub fn with_app<T>(f: impl FnOnce(&mut App) -> T) -> Option<T> {
    APP.with(|slot| match slot.try_borrow_mut() {
        Ok(mut borrowed) => borrowed.as_mut().map(f),
        Err(_) => None,
    })
}

pub fn take_app() -> Option<App> {
    APP.with(|slot| slot.borrow_mut().take())
}

impl App {
    pub fn new(cfg: Config) -> Self {
        let journal = Journal::new(cfg.log_to_file);
        Self {
            cfg,
            journal,
            input: InputWatch::new(),
            previous: None,
            expecting: None,
            strikes: HashMap::new(),
            hook: HWINEVENTHOOK::default(),
            tray_window: HWND::default(),
            viewer: HWND::default(),
            paused_until: None,
            last_restore: None,
            ticks: 0,
            config_stamp: config::changed_at(),
        }
    }

    pub fn save_config(&mut self) {
        let _ = self.cfg.save();
        self.config_stamp = config::changed_at();
    }

    /// Returns true when the settings actually changed under us.
    pub fn reload_config_if_changed(&mut self) -> bool {
        let stamp = config::changed_at();
        if stamp == self.config_stamp {
            return false;
        }
        self.config_stamp = stamp;
        let fresh = Config::load();
        if fresh == self.cfg {
            return false;
        }
        self.journal.set_to_file(fresh.log_to_file);
        self.cfg = fresh;
        true
    }

    /// True while the same process is still mid-launch after a restore.
    pub fn settling(&self, pid: u32) -> bool {
        const SETTLE: Duration = Duration::from_millis(400);
        self.last_restore
            .is_some_and(|(last, at)| last == pid && at.elapsed() < SETTLE)
    }

    pub fn is_paused(&self) -> bool {
        self.paused_until
            .is_some_and(|until| Instant::now() < until)
    }

    pub fn strikes_for(&mut self, exe: &str) -> u32 {
        let window = Duration::from_secs(self.cfg.restore_window_secs);
        match self.strikes.get(exe) {
            Some(strike) if strike.since.elapsed() < window => strike.count,
            _ => {
                self.strikes.remove(exe);
                0
            }
        }
    }

    pub fn add_strike(&mut self, exe: &str) {
        let window = Duration::from_secs(self.cfg.restore_window_secs);
        match self.strikes.get_mut(exe) {
            Some(strike) if strike.since.elapsed() < window => strike.count += 1,
            _ => {
                self.strikes.insert(
                    exe.to_owned(),
                    Strike {
                        count: 1,
                        since: Instant::now(),
                    },
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::thread::sleep;

    fn app() -> App {
        App::new(Config {
            restore_window_secs: 1,
            log_to_file: false,
            ..Config::default()
        })
    }

    #[test]
    fn an_application_starts_with_a_clean_sheet() {
        let mut app = app();
        assert_eq!(app.strikes_for("updater.exe"), 0);
    }

    #[test]
    fn strikes_are_counted_per_application() {
        let mut app = app();
        app.add_strike("updater.exe");
        app.add_strike("updater.exe");
        assert_eq!(app.strikes_for("updater.exe"), 2);
        assert_eq!(app.strikes_for("teams.exe"), 0);
    }

    #[test]
    fn strikes_expire_so_yesterdays_offender_is_not_punished_today() {
        let mut app = app();
        app.add_strike("updater.exe");
        assert_eq!(app.strikes_for("updater.exe"), 1);
        sleep(Duration::from_millis(1_100));
        assert_eq!(app.strikes_for("updater.exe"), 0);
    }

    #[test]
    fn a_window_from_the_process_we_just_answered_is_still_settling() {
        let mut app = app();
        app.last_restore = Some((4242, Instant::now()));
        assert!(app.settling(4242));
        assert!(!app.settling(99), "a different process is a fresh event");
    }

    #[test]
    fn settling_wears_off() {
        let mut app = app();
        app.last_restore = Some((4242, Instant::now() - Duration::from_millis(500)));
        assert!(!app.settling(4242));
    }

    #[test]
    fn nothing_is_settling_before_the_first_restore() {
        let app = app();
        assert!(!app.settling(4242));
    }

    #[test]
    fn a_pause_ends_by_itself() {
        let mut app = app();
        assert!(!app.is_paused());
        app.paused_until = Some(Instant::now() + Duration::from_secs(60));
        assert!(app.is_paused());
        app.paused_until = Some(Instant::now() - Duration::from_secs(1));
        assert!(!app.is_paused(), "a pause in the past is not a pause");
    }
}
