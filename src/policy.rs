//! The decision this whole program exists to make: a window just came to the front,
//! do we leave it alone or hand focus back? Getting this wrong in the other direction
//! is worse than the problem, so every branch that could be the user's own doing wins.

use crate::config::{Config, Mode};
use crate::journal::Verdict;

pub struct Situation<'a> {
    pub exe: &'a str,
    pub is_noise: bool,
    pub same_process_as_previous: bool,
    pub has_previous_window: bool,
    pub ms_since_click: Option<u32>,
    pub button_down: bool,
    pub keyboard_switch: bool,
    pub ms_since_input: u32,
    pub restores_used: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Decision {
    pub verdict: Verdict,
    pub reason: &'static str,
}

const fn d(verdict: Verdict, reason: &'static str) -> Decision {
    Decision { verdict, reason }
}

pub fn decide(cfg: &Config, sit: &Situation) -> Decision {
    if sit.is_noise {
        return d(Verdict::Allowed, "system window");
    }
    if sit.same_process_as_previous {
        return d(Verdict::Allowed, "same application");
    }
    if cfg.is_allowed(sit.exe) {
        return d(Verdict::Allowed, "on allow list");
    }
    if sit.keyboard_switch {
        return d(Verdict::Allowed, "you switched windows");
    }
    if sit.button_down {
        return d(Verdict::Allowed, "mouse button held");
    }
    if sit
        .ms_since_click
        .is_some_and(|ms| u64::from(ms) <= cfg.click_grace_ms)
    {
        return d(Verdict::Allowed, "you clicked");
    }

    let blocked = cfg.is_blocked(sit.exe);
    let typing = u64::from(sit.ms_since_input) <= cfg.typing_window_ms;

    let reason = if blocked {
        "on block list"
    } else if typing {
        "you were typing"
    } else {
        "you were away"
    };

    let wants_restore = match cfg.mode {
        Mode::Watch => false,
        Mode::Guard => blocked || typing,
        Mode::Strict => true,
    };

    if !wants_restore {
        return d(Verdict::Observed, reason);
    }
    if !sit.has_previous_window {
        return d(Verdict::Observed, "nothing to go back to");
    }
    if sit.restores_used >= cfg.max_restores {
        return d(Verdict::GaveUp, "keeps grabbing focus");
    }
    d(Verdict::Restored, reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base<'a>(exe: &'a str) -> Situation<'a> {
        Situation {
            exe,
            is_noise: false,
            same_process_as_previous: false,
            has_previous_window: true,
            ms_since_click: None,
            button_down: false,
            keyboard_switch: false,
            ms_since_input: 50,
            restores_used: 0,
        }
    }

    fn guarding() -> Config {
        Config {
            mode: Mode::Guard,
            ..Config::default()
        }
    }

    #[test]
    fn watch_mode_never_takes_focus_back() {
        let cfg = Config::default();
        let mut sit = base("updater.exe");
        sit.ms_since_input = 10;
        assert_eq!(decide(&cfg, &sit).verdict, Verdict::Observed);
    }

    #[test]
    fn a_click_always_wins() {
        let mut cfg = guarding();
        cfg.blocklist.push("updater.exe".into());
        let mut sit = base("updater.exe");
        sit.ms_since_click = Some(100);
        assert_eq!(decide(&cfg, &sit).verdict, Verdict::Allowed);
    }

    #[test]
    fn a_stale_click_does_not_excuse_a_steal() {
        let cfg = guarding();
        let mut sit = base("updater.exe");
        sit.ms_since_click = Some(9_000);
        assert_eq!(decide(&cfg, &sit).verdict, Verdict::Restored);
    }

    #[test]
    fn typing_is_protected_in_guard_mode() {
        let cfg = guarding();
        let sit = base("updater.exe");
        let out = decide(&cfg, &sit);
        assert_eq!(out.verdict, Verdict::Restored);
        assert_eq!(out.reason, "you were typing");
    }

    #[test]
    fn idle_user_is_only_recorded_unless_blocked() {
        let cfg = guarding();
        let mut sit = base("updater.exe");
        sit.ms_since_input = 60_000;
        assert_eq!(decide(&cfg, &sit).verdict, Verdict::Observed);

        let mut blocking = guarding();
        blocking.blocklist.push("updater.exe".into());
        assert_eq!(decide(&blocking, &sit).verdict, Verdict::Restored);
    }

    #[test]
    fn alt_tab_is_the_user_switching() {
        let cfg = Config {
            mode: Mode::Strict,
            ..Config::default()
        };
        let mut sit = base("anything.exe");
        sit.keyboard_switch = true;
        assert_eq!(decide(&cfg, &sit).verdict, Verdict::Allowed);
    }

    #[test]
    fn strict_mode_takes_back_from_an_idle_desktop() {
        let cfg = Config {
            mode: Mode::Strict,
            ..Config::default()
        };
        let mut sit = base("anything.exe");
        sit.ms_since_input = 600_000;
        assert_eq!(decide(&cfg, &sit).verdict, Verdict::Restored);
    }

    #[test]
    fn allow_list_beats_strict_mode() {
        let cfg = Config {
            mode: Mode::Strict,
            ..Config::default()
        };
        let sit = base("explorer.exe");
        assert_eq!(decide(&cfg, &sit).verdict, Verdict::Allowed);
    }

    #[test]
    fn we_stop_fighting_a_persistent_thief() {
        let cfg = guarding();
        let mut sit = base("updater.exe");
        sit.restores_used = cfg.max_restores;
        assert_eq!(decide(&cfg, &sit).verdict, Verdict::GaveUp);
    }

    #[test]
    fn nothing_to_restore_means_nothing_to_do() {
        let cfg = guarding();
        let mut sit = base("updater.exe");
        sit.has_previous_window = false;
        assert_eq!(decide(&cfg, &sit).verdict, Verdict::Observed);
    }

    #[test]
    fn a_dialog_from_the_app_you_are_using_is_not_a_theft() {
        let cfg = Config {
            mode: Mode::Strict,
            ..Config::default()
        };
        let mut sit = base("code.exe");
        sit.same_process_as_previous = true;
        assert_eq!(decide(&cfg, &sit).verdict, Verdict::Allowed);
    }
}
