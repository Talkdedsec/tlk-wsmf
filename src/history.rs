//! Reading back what was recorded. The live journal only holds the last few hundred
//! entries in memory; the statistics view wants weeks, and gets them from the file.

use crate::config;
use crate::journal::Entry;
use chrono::{Duration, Local, NaiveDate, Timelike};
use std::collections::HashMap;
use std::fs;

/// Entries from the last `days` days, newest first. Unreadable lines are skipped:
/// a corrupt log should cost you one line, not the whole history.
pub fn load(days: i64) -> Vec<Entry> {
    let cutoff = Local::now() - Duration::days(days);
    let mut entries = Vec::new();
    let current = config::log_path();
    let rotated = current.with_extension("log.1");
    for path in [rotated, current] {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        entries.extend(
            text.lines()
                .filter_map(Entry::parse)
                .filter(|entry| entry.at >= cutoff),
        );
    }
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.at));
    entries
}

#[derive(Debug, Default, Clone)]
pub struct Stats {
    pub total: usize,
    pub restored: usize,
    /// Applications that interrupted, worst first.
    pub by_app: Vec<(String, usize)>,
    pub by_hour: [usize; 24],
    /// One entry per day in the range, oldest first, including quiet days.
    pub by_day: Vec<(NaiveDate, usize)>,
}

pub fn summarize(entries: &[Entry], days: i64) -> Stats {
    let interruptions: Vec<&Entry> = entries
        .iter()
        .filter(|entry| entry.verdict.is_interruption())
        .collect();

    let mut by_app: HashMap<&str, usize> = HashMap::new();
    let mut by_hour = [0usize; 24];
    let mut per_day: HashMap<NaiveDate, usize> = HashMap::new();

    for entry in &interruptions {
        *by_app.entry(entry.exe.as_str()).or_default() += 1;
        by_hour[entry.at.hour() as usize] += 1;
        *per_day.entry(entry.at.date_naive()).or_default() += 1;
    }

    let mut by_app: Vec<(String, usize)> = by_app
        .into_iter()
        .map(|(exe, count)| (exe.to_owned(), count))
        .collect();
    by_app.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let today = Local::now().date_naive();
    let by_day = (0..days)
        .rev()
        .filter_map(|back| today.checked_sub_signed(Duration::days(back)))
        .map(|date| (date, per_day.get(&date).copied().unwrap_or(0)))
        .collect();

    Stats {
        total: interruptions.len(),
        restored: interruptions
            .iter()
            .filter(|entry| entry.verdict == crate::journal::Verdict::Restored)
            .count(),
        by_app,
        by_hour,
        by_day,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::journal::{Reason, Verdict};
    use chrono::TimeZone;

    fn at(hours_ago: i64, exe: &str, verdict: Verdict) -> Entry {
        Entry {
            at: Local::now() - Duration::hours(hours_ago),
            exe: exe.into(),
            path: String::new(),
            title: String::new(),
            verdict,
            reason: Reason::Typing,
        }
    }

    #[test]
    fn the_worst_offender_comes_first() {
        let entries = vec![
            at(1, "teams.exe", Verdict::Restored),
            at(2, "updater.exe", Verdict::Restored),
            at(3, "teams.exe", Verdict::Observed),
            at(4, "teams.exe", Verdict::Restored),
        ];
        let stats = summarize(&entries, 7);
        assert_eq!(stats.by_app[0], ("teams.exe".to_string(), 3));
        assert_eq!(stats.by_app[1], ("updater.exe".to_string(), 1));
        assert_eq!(stats.total, 4);
        assert_eq!(stats.restored, 3);
    }

    #[test]
    fn a_tie_is_broken_by_name_so_the_order_does_not_jitter() {
        let entries = vec![
            at(1, "zeta.exe", Verdict::Restored),
            at(2, "alpha.exe", Verdict::Restored),
        ];
        let first = summarize(&entries, 7).by_app;
        let second = summarize(&entries, 7).by_app;
        assert_eq!(first, second);
        assert_eq!(first[0].0, "alpha.exe");
    }

    #[test]
    fn allowed_lines_do_not_count_as_interruptions() {
        let entries = vec![
            at(1, "explorer.exe", Verdict::Allowed),
            at(2, "teams.exe", Verdict::Restored),
        ];
        let stats = summarize(&entries, 7);
        assert_eq!(stats.total, 1);
        assert!(!stats.by_app.iter().any(|(exe, _)| exe == "explorer.exe"));
    }

    #[test]
    fn quiet_days_are_still_days() {
        let stats = summarize(&[at(1, "a.exe", Verdict::Restored)], 14);
        assert_eq!(stats.by_day.len(), 14);
        assert!(stats.by_day.windows(2).all(|w| w[0].0 < w[1].0));
        assert_eq!(stats.by_day.iter().map(|(_, n)| n).sum::<usize>(), 1);
    }

    #[test]
    fn an_empty_history_summarises_to_zero_rather_than_panicking() {
        let stats = summarize(&[], 14);
        assert_eq!(stats.total, 0);
        assert!(stats.by_app.is_empty());
        assert_eq!(stats.by_hour.iter().sum::<usize>(), 0);
        assert_eq!(stats.by_day.len(), 14);
    }

    #[test]
    fn hours_land_in_the_right_bucket() {
        let entry = Entry {
            at: Local.with_ymd_and_hms(2026, 9, 8, 14, 30, 0).unwrap(),
            exe: "a.exe".into(),
            path: String::new(),
            title: String::new(),
            verdict: Verdict::Restored,
            reason: Reason::Typing,
        };
        let stats = summarize(&[entry], 1);
        assert_eq!(stats.by_hour[14], 1);
        assert_eq!(stats.by_hour.iter().sum::<usize>(), 1);
    }
}
