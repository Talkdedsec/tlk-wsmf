use crate::config::Config;
use crate::input::InputWatch;
use crate::journal::Journal;
use crate::window::WindowInfo;
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Instant;
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
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused_until
            .is_some_and(|until| Instant::now() < until)
    }

    pub fn strikes_for(&mut self, exe: &str) -> u32 {
        let window = std::time::Duration::from_secs(self.cfg.restore_window_secs);
        match self.strikes.get(exe) {
            Some(strike) if strike.since.elapsed() < window => strike.count,
            _ => {
                self.strikes.remove(exe);
                0
            }
        }
    }

    pub fn add_strike(&mut self, exe: &str) {
        let window = std::time::Duration::from_secs(self.cfg.restore_window_secs);
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
