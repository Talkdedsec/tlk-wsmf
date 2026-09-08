use crate::config;
use crate::i18n::Strings;
use chrono::{DateTime, Local};
use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::Write;

const KEEP_IN_MEMORY: usize = 500;
const MAX_LOG_BYTES: u64 = 2_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The user did this, or the window is allowed. Left alone.
    Allowed,
    /// Focus was taken back and returned to the previous window.
    Restored,
    /// Recorded only, because the mode in force does not intervene.
    Observed,
    /// Kept grabbing focus, so we stopped fighting it.
    GaveUp,
}

impl Verdict {
    /// Written to the log file. Never translated: the file outlives a language choice.
    pub fn key(self) -> &'static str {
        match self {
            Verdict::Allowed => "allowed",
            Verdict::Restored => "restored",
            Verdict::Observed => "observed",
            Verdict::GaveUp => "gave_up",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Some(match key {
            "allowed" => Verdict::Allowed,
            "restored" => Verdict::Restored,
            "observed" => Verdict::Observed,
            "gave_up" => Verdict::GaveUp,
            _ => return None,
        })
    }

    pub fn label(self, s: &'static Strings) -> &'static str {
        match self {
            Verdict::Allowed => s.verdict_allowed,
            Verdict::Restored => s.verdict_restored,
            Verdict::Observed => s.verdict_observed,
            Verdict::GaveUp => s.verdict_gave_up,
        }
    }

    /// Whether this line counts as an interruption at all.
    pub fn is_interruption(self) -> bool {
        matches!(
            self,
            Verdict::Restored | Verdict::Observed | Verdict::GaveUp
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    Typing,
    Blocklist,
    Away,
    Clicked,
    ButtonDown,
    Switching,
    SameApp,
    Allowlist,
    SystemWindow,
    NoTarget,
    Persistent,
    Failed,
}

impl Reason {
    pub fn key(self) -> &'static str {
        match self {
            Reason::Typing => "typing",
            Reason::Blocklist => "blocklist",
            Reason::Away => "away",
            Reason::Clicked => "clicked",
            Reason::ButtonDown => "button_down",
            Reason::Switching => "switching",
            Reason::SameApp => "same_app",
            Reason::Allowlist => "allowlist",
            Reason::SystemWindow => "system_window",
            Reason::NoTarget => "no_target",
            Reason::Persistent => "persistent",
            Reason::Failed => "failed",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Some(match key {
            "typing" => Reason::Typing,
            "blocklist" => Reason::Blocklist,
            "away" => Reason::Away,
            "clicked" => Reason::Clicked,
            "button_down" => Reason::ButtonDown,
            "switching" => Reason::Switching,
            "same_app" => Reason::SameApp,
            "allowlist" => Reason::Allowlist,
            "system_window" => Reason::SystemWindow,
            "no_target" => Reason::NoTarget,
            "persistent" => Reason::Persistent,
            "failed" => Reason::Failed,
            _ => return None,
        })
    }

    pub fn label(self, s: &'static Strings) -> &'static str {
        match self {
            Reason::Typing => s.reason_typing,
            Reason::Blocklist => s.reason_blocklist,
            Reason::Away => s.reason_away,
            Reason::Clicked => s.reason_clicked,
            Reason::ButtonDown => s.reason_button_down,
            Reason::Switching => s.reason_switching,
            Reason::SameApp => s.reason_same_app,
            Reason::Allowlist => s.reason_allowlist,
            Reason::SystemWindow => s.reason_system_window,
            Reason::NoTarget => s.reason_no_target,
            Reason::Persistent => s.reason_persistent,
            Reason::Failed => s.reason_failed,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub at: DateTime<Local>,
    pub exe: String,
    pub path: String,
    pub title: String,
    pub verdict: Verdict,
    pub reason: Reason,
}

impl Entry {
    pub fn to_line(&self) -> String {
        let clean = |text: &str| text.replace(['\t', '\n', '\r'], " ");
        format!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            self.at.to_rfc3339(),
            self.verdict.key(),
            self.reason.key(),
            clean(&self.exe),
            clean(&self.path),
            clean(&self.title)
        )
    }

    pub fn parse(line: &str) -> Option<Self> {
        let mut parts = line.split('\t');
        let at = DateTime::parse_from_rfc3339(parts.next()?)
            .ok()?
            .with_timezone(&Local);
        let verdict = Verdict::from_key(parts.next()?)?;
        let reason = Reason::from_key(parts.next()?)?;
        Some(Self {
            at,
            verdict,
            reason,
            exe: parts.next()?.to_owned(),
            path: parts.next().unwrap_or_default().to_owned(),
            title: parts.next().unwrap_or_default().to_owned(),
        })
    }
}

pub struct Journal {
    entries: VecDeque<Entry>,
    to_file: bool,
}

impl Journal {
    pub fn new(to_file: bool) -> Self {
        Self {
            entries: VecDeque::with_capacity(KEEP_IN_MEMORY),
            to_file,
        }
    }

    pub fn record(&mut self, entry: Entry) {
        if self.to_file {
            append_to_file(&entry);
        }
        if self.entries.len() == KEEP_IN_MEMORY {
            self.entries.pop_back();
        }
        self.entries.push_front(entry);
    }

    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn set_to_file(&mut self, on: bool) {
        self.to_file = on;
    }

    pub fn interruptions_today(&self) -> usize {
        let today = Local::now().date_naive();
        self.entries
            .iter()
            .filter(|e| e.at.date_naive() == today && e.verdict.is_interruption())
            .count()
    }
}

fn append_to_file(entry: &Entry) {
    let path = config::log_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if fs::metadata(&path).map(|m| m.len()).unwrap_or(0) > MAX_LOG_BYTES {
        let _ = fs::rename(&path, path.with_extension("log.1"));
    }
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) else {
        return;
    };
    let _ = writeln!(file, "{}", entry.to_line());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Entry {
        Entry {
            at: Local::now(),
            exe: "updater.exe".into(),
            path: r"C:\Program Files\Thing\updater.exe".into(),
            title: "Update available".into(),
            verdict: Verdict::Restored,
            reason: Reason::Typing,
        }
    }

    #[test]
    fn a_line_survives_a_round_trip() {
        let entry = sample();
        let parsed = Entry::parse(&entry.to_line()).expect("should parse");
        assert_eq!(parsed.exe, entry.exe);
        assert_eq!(parsed.path, entry.path);
        assert_eq!(parsed.title, entry.title);
        assert_eq!(parsed.verdict, entry.verdict);
        assert_eq!(parsed.reason, entry.reason);
        assert_eq!(parsed.at.timestamp(), entry.at.timestamp());
    }

    #[test]
    fn tabs_in_a_window_title_cannot_break_the_format() {
        let mut entry = sample();
        entry.title = "before\tafter\nnext".into();
        let line = entry.to_line();
        assert_eq!(line.matches('\t').count(), 5);
        assert_eq!(Entry::parse(&line).unwrap().title, "before after next");
    }

    #[test]
    fn rubbish_lines_are_skipped_not_fatal() {
        assert!(Entry::parse("").is_none());
        assert!(Entry::parse("not a line at all").is_none());
        assert!(Entry::parse("2026-09-08T00:00:00+03:00\tnope\ttyping\ta.exe").is_none());
    }

    #[test]
    fn a_line_written_by_a_newer_version_does_not_break_an_older_one() {
        let line = format!("{}\textra\tcolumns", sample().to_line());
        assert!(Entry::parse(&line).is_some());
    }

    #[test]
    fn the_oldest_entries_fall_off_the_end() {
        let mut journal = Journal::new(false);
        for _ in 0..KEEP_IN_MEMORY + 50 {
            journal.record(sample());
        }
        assert_eq!(journal.len(), KEEP_IN_MEMORY);
    }

    #[test]
    fn allowed_lines_are_not_interruptions() {
        assert!(!Verdict::Allowed.is_interruption());
        assert!(Verdict::Restored.is_interruption());
        assert!(Verdict::Observed.is_interruption());
        assert!(Verdict::GaveUp.is_interruption());
    }

    #[test]
    fn every_key_maps_back_to_itself() {
        for verdict in [
            Verdict::Allowed,
            Verdict::Restored,
            Verdict::Observed,
            Verdict::GaveUp,
        ] {
            assert_eq!(Verdict::from_key(verdict.key()), Some(verdict));
        }
        for reason in [
            Reason::Typing,
            Reason::Blocklist,
            Reason::Away,
            Reason::Clicked,
            Reason::ButtonDown,
            Reason::Switching,
            Reason::SameApp,
            Reason::Allowlist,
            Reason::SystemWindow,
            Reason::NoTarget,
            Reason::Persistent,
            Reason::Failed,
        ] {
            assert_eq!(Reason::from_key(reason.key()), Some(reason));
        }
    }
}
