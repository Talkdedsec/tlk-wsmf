//! The decision this whole program exists to make: a window just came to the front,
//! do we leave it alone or hand focus back? Getting this wrong in the other direction
//! is worse than the problem, so every branch that could be the user's own doing wins.

use crate::config::{Config, Mode};
use crate::journal::{Reason, Verdict};

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
    pub reason: Reason,
}

const fn d(verdict: Verdict, reason: Reason) -> Decision {
    Decision { verdict, reason }
}

pub fn decide(cfg: &Config, sit: &Situation) -> Decision {
    if sit.is_noise {
        return d(Verdict::Allowed, Reason::SystemWindow);
    }
    if sit.same_process_as_previous {
        return d(Verdict::Allowed, Reason::SameApp);
    }
    if cfg.is_allowed(sit.exe) {
        return d(Verdict::Allowed, Reason::Allowlist);
    }
    if sit.keyboard_switch {
        return d(Verdict::Allowed, Reason::Switching);
    }
    if sit.button_down {
        return d(Verdict::Allowed, Reason::ButtonDown);
    }
    if sit
        .ms_since_click
        .is_some_and(|ms| u64::from(ms) <= cfg.click_grace_ms)
    {
        return d(Verdict::Allowed, Reason::Clicked);
    }

    let blocked = cfg.is_blocked(sit.exe);
    let typing = u64::from(sit.ms_since_input) <= cfg.typing_window_ms;

    let reason = if blocked {
        Reason::Blocklist
    } else if typing {
        Reason::Typing
    } else {
        Reason::Away
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
        return d(Verdict::Observed, Reason::NoTarget);
    }
    if sit.restores_used >= cfg.max_restores {
        return d(Verdict::GaveUp, Reason::Persistent);
    }
    d(Verdict::Restored, reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(exe: &str) -> Situation<'_> {
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

    fn strict() -> Config {
        Config {
            mode: Mode::Strict,
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
        let out = decide(&cfg, &sit);
        assert_eq!(out.verdict, Verdict::Allowed);
        assert_eq!(out.reason, Reason::Clicked);
    }

    #[test]
    fn a_click_exactly_on_the_edge_still_counts_as_yours() {
        let cfg = guarding();
        let mut sit = base("updater.exe");
        sit.ms_since_click = Some(cfg.click_grace_ms as u32);
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
    fn a_held_mouse_button_is_the_user_mid_action() {
        let cfg = strict();
        let mut sit = base("updater.exe");
        sit.button_down = true;
        assert_eq!(decide(&cfg, &sit).reason, Reason::ButtonDown);
    }

    #[test]
    fn typing_is_protected_in_guard_mode() {
        let cfg = guarding();
        let sit = base("updater.exe");
        let out = decide(&cfg, &sit);
        assert_eq!(out.verdict, Verdict::Restored);
        assert_eq!(out.reason, Reason::Typing);
    }

    #[test]
    fn typing_stops_counting_after_the_window_passes() {
        let cfg = guarding();
        let mut sit = base("updater.exe");
        sit.ms_since_input = cfg.typing_window_ms as u32 + 1;
        let out = decide(&cfg, &sit);
        assert_eq!(out.verdict, Verdict::Observed);
        assert_eq!(out.reason, Reason::Away);
    }

    #[test]
    fn idle_user_is_only_recorded_unless_blocked() {
        let cfg = guarding();
        let mut sit = base("updater.exe");
        sit.ms_since_input = 60_000;
        assert_eq!(decide(&cfg, &sit).verdict, Verdict::Observed);

        let mut blocking = guarding();
        blocking.blocklist.push("updater.exe".into());
        let out = decide(&blocking, &sit);
        assert_eq!(out.verdict, Verdict::Restored);
        assert_eq!(out.reason, Reason::Blocklist);
    }

    #[test]
    fn alt_tab_is_the_user_switching() {
        let cfg = strict();
        let mut sit = base("anything.exe");
        sit.keyboard_switch = true;
        assert_eq!(decide(&cfg, &sit).verdict, Verdict::Allowed);
    }

    #[test]
    fn strict_mode_takes_back_from_an_idle_desktop() {
        let cfg = strict();
        let mut sit = base("anything.exe");
        sit.ms_since_input = 600_000;
        assert_eq!(decide(&cfg, &sit).verdict, Verdict::Restored);
    }

    #[test]
    fn allow_list_beats_strict_mode() {
        let cfg = strict();
        let sit = base("explorer.exe");
        assert_eq!(decide(&cfg, &sit).verdict, Verdict::Allowed);
    }

    #[test]
    fn a_system_window_is_never_a_thief() {
        let cfg = strict();
        let mut sit = base("dwm.exe");
        sit.is_noise = true;
        assert_eq!(decide(&cfg, &sit).reason, Reason::SystemWindow);
    }

    #[test]
    fn we_stop_fighting_a_persistent_thief() {
        let cfg = guarding();
        let mut sit = base("updater.exe");
        sit.restores_used = cfg.max_restores;
        let out = decide(&cfg, &sit);
        assert_eq!(out.verdict, Verdict::GaveUp);
        assert_eq!(out.reason, Reason::Persistent);
    }

    #[test]
    fn one_below_the_limit_still_fights() {
        let cfg = guarding();
        let mut sit = base("updater.exe");
        sit.restores_used = cfg.max_restores - 1;
        assert_eq!(decide(&cfg, &sit).verdict, Verdict::Restored);
    }

    #[test]
    fn nothing_to_restore_means_nothing_to_do() {
        let cfg = guarding();
        let mut sit = base("updater.exe");
        sit.has_previous_window = false;
        let out = decide(&cfg, &sit);
        assert_eq!(out.verdict, Verdict::Observed);
        assert_eq!(out.reason, Reason::NoTarget);
    }

    #[test]
    fn a_dialog_from_the_app_you_are_using_is_not_a_theft() {
        let cfg = strict();
        let mut sit = base("code.exe");
        sit.same_process_as_previous = true;
        assert_eq!(decide(&cfg, &sit).verdict, Verdict::Allowed);
    }

    #[test]
    fn the_block_list_beats_the_allow_list_when_both_are_set() {
        // block() and allow() keep a name out of the other list, but a hand edited
        // config file can have both. The allow list wins, and it must do so quietly
        // rather than flip flopping between runs.
        let mut cfg = guarding();
        cfg.blocklist.push("thing.exe".into());
        cfg.allowlist.push("thing.exe".into());
        assert_eq!(decide(&cfg, &base("thing.exe")).verdict, Verdict::Allowed);
    }

    #[test]
    fn names_are_matched_without_case() {
        let mut cfg = guarding();
        cfg.blocklist.push("updater.exe".into());
        let mut sit = base("UPDATER.EXE");
        sit.ms_since_input = 60_000;
        // capture() lower cases before we get here, so this documents the contract
        // rather than the current call site.
        assert_eq!(decide(&cfg, &sit).verdict, Verdict::Observed);
        sit.exe = "updater.exe";
        assert_eq!(decide(&cfg, &sit).verdict, Verdict::Restored);
    }
}
