use crate::config;
use chrono::{DateTime, Local};
use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::Write;

const KEEP_IN_MEMORY: usize = 500;
const MAX_LOG_BYTES: u64 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The user did this, or the window is allowed. Left alone.
    Allowed,
    /// Focus was taken back and returned to the previous window.
    Restored,
    /// Recorded only, because watch mode never intervenes.
    Observed,
    /// Kept grabbing focus, so we stopped fighting it.
    GaveUp,
}

impl Verdict {
    pub fn label(self) -> &'static str {
        match self {
            Verdict::Allowed => "allowed",
            Verdict::Restored => "took back",
            Verdict::Observed => "seen",
            Verdict::GaveUp => "gave up",
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
    pub reason: &'static str,
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
            self.append_to_file(&entry);
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

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// How many times this executable has been caught, for the tray tooltip.
    pub fn thefts_today(&self) -> usize {
        let today = Local::now().date_naive();
        self.entries
            .iter()
            .filter(|e| {
                e.at.date_naive() == today
                    && matches!(e.verdict, Verdict::Restored | Verdict::Observed)
            })
            .count()
    }

    fn append_to_file(&self, entry: &Entry) {
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
        let _ = writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}",
            entry.at.format("%Y-%m-%d %H:%M:%S"),
            entry.verdict.label(),
            entry.exe,
            entry.reason,
            entry.title.replace(['\t', '\n', '\r'], " ")
        );
    }
}
