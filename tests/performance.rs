//! The budget. "Small and light" is a claim, and these tests are what turns it into
//! a number that fails loudly when it stops being true.
//!
//! Thresholds are deliberately generous: they catch a regression of the kind that
//! matters (a decision that suddenly allocates, a tray program holding 200 MB),
//! not the noise of a busy machine.

#![cfg(windows)]

use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::{Duration, Instant};
use tlk_wsmf::config::{Config, Mode};
use tlk_wsmf::journal::{Entry, Reason, Verdict};
use tlk_wsmf::policy::{Situation, decide};

/// A focus change happens on a user's timescale, so the decision has room to spare.
/// The number that matters is the order of magnitude: a decision that starts
/// allocating, or scanning something it should not, lands tens of times over this.
/// The budget is generous because a shared CI runner is not a quiet machine - on a
/// real desktop this measures around a microsecond with a thousand rules loaded.
const MAX_DECISION_NANOS: u128 = 8_000;
/// Best of several runs: one unlucky scheduling slice should not fail a build.
const TIMING_ROUNDS: u32 = 3;
/// The whole point of the tray process is to be forgettable.
const MAX_WORKING_SET_MB: f64 = 48.0;
/// It watches an event that fires a few times a minute. It must be asleep otherwise.
const MAX_CPU_SECONDS_WHILE_IDLE: f64 = 0.35;
const IDLE_OBSERVATION: Duration = Duration::from_secs(6);
const MAX_STARTUP: Duration = Duration::from_secs(3);

fn situation<'a>(exe: &'a str) -> Situation<'a> {
    Situation {
        exe,
        is_noise: false,
        same_process_as_previous: false,
        has_previous_window: true,
        ms_since_click: Some(9_000),
        button_down: false,
        keyboard_switch: false,
        ms_since_input: 50,
        restores_used: 0,
    }
}

#[test]
fn a_decision_is_effectively_free() {
    if cfg!(debug_assertions) {
        eprintln!("skipped: timings only mean something in a release build");
        return;
    }
    let mut cfg = Config {
        mode: Mode::Guard,
        ..Config::default()
    };
    // A pathological config: far more rules than anyone will type by hand.
    for index in 0..500 {
        cfg.block(&format!("blocked{index}.exe"));
        cfg.allow(&format!("allowed{index}.exe"));
    }
    let sit = situation("something-not-on-any-list.exe");

    // Warm up, then measure.
    for _ in 0..1_000 {
        std::hint::black_box(decide(&cfg, &sit));
    }
    const CALLS: u32 = 200_000;
    let per_call = (0..TIMING_ROUNDS)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..CALLS {
                std::hint::black_box(decide(&cfg, &sit));
            }
            started.elapsed().as_nanos() / u128::from(CALLS)
        })
        .min()
        .expect("at least one round");
    assert!(
        per_call <= MAX_DECISION_NANOS,
        "a decision took {per_call} ns against a budget of {MAX_DECISION_NANOS} ns"
    );
}

#[test]
fn parsing_a_fortnight_of_history_is_quick() {
    if cfg!(debug_assertions) {
        eprintln!("skipped: timings only mean something in a release build");
        return;
    }
    let entry = Entry {
        at: chrono::Local::now(),
        exe: "teams.exe".into(),
        path: r"C:\Program Files\Teams\teams.exe".into(),
        title: "Someone is calling you".into(),
        verdict: Verdict::Restored,
        reason: Reason::Typing,
    };
    let line = entry.to_line();
    let started = Instant::now();
    let mut parsed = 0;
    for _ in 0..20_000 {
        if Entry::parse(&line).is_some() {
            parsed += 1;
        }
    }
    let elapsed = started.elapsed();
    assert_eq!(parsed, 20_000);
    assert!(
        elapsed < Duration::from_millis(400),
        "parsing 20000 log lines took {elapsed:?}"
    );
}

struct Measured {
    child: Child,
    #[allow(dead_code)]
    home: PathBuf,
}

impl Drop for Measured {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.home);
    }
}

/// Runs the real executable with its own settings directory and its own instance
/// name, so it neither disturbs nor is disturbed by a copy already in the tray.
fn start_guard(tag: &str) -> Option<Measured> {
    let exe = built_binary()?;
    let home = std::env::temp_dir().join(format!("wsmf-perf-{tag}-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&home);
    let child = Command::new(exe)
        .env("APPDATA", &home)
        .env("WSMF_INSTANCE", tag)
        .spawn()
        .ok()?;
    Some(Measured { child, home })
}

fn built_binary() -> Option<PathBuf> {
    // tests run from target/<profile>/deps, and the binary sits one level up.
    let mut dir = std::env::current_exe().ok()?;
    dir.pop();
    if dir.ends_with("deps") {
        dir.pop();
    }
    let exe = dir.join("wsmf.exe");
    exe.exists().then_some(exe)
}

#[test]
fn the_guard_starts_quickly_and_stays_small_and_idle() {
    let Some(mut guard) = start_guard("startup") else {
        eprintln!("skipped: wsmf.exe has not been built yet");
        return;
    };

    let started = Instant::now();
    let mut ready = false;
    while started.elapsed() < MAX_STARTUP {
        if matches!(guard.child.try_wait(), Ok(None)) && counters(guard.child.id()).is_some() {
            ready = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(
        ready,
        "the guard did not come up within {MAX_STARTUP:?}; it exited or never started"
    );

    let before = counters(guard.child.id()).expect("a running process has counters");
    std::thread::sleep(IDLE_OBSERVATION);
    let after = counters(guard.child.id()).expect("the guard should still be running");

    let working_set_mb = after.working_set as f64 / (1024.0 * 1024.0);
    assert!(
        working_set_mb <= MAX_WORKING_SET_MB,
        "the guard held {working_set_mb:.1} MB against a budget of {MAX_WORKING_SET_MB} MB"
    );

    let cpu = after.cpu_seconds - before.cpu_seconds;
    assert!(
        cpu <= MAX_CPU_SECONDS_WHILE_IDLE,
        "the guard used {cpu:.3} s of CPU while idle for {IDLE_OBSERVATION:?}, \
         against a budget of {MAX_CPU_SECONDS_WHILE_IDLE} s"
    );
}

#[test]
fn a_second_launch_does_not_leave_a_second_process() {
    let Some(guard) = start_guard("single") else {
        eprintln!("skipped: wsmf.exe has not been built yet");
        return;
    };
    std::thread::sleep(Duration::from_millis(900));

    let exe = built_binary().expect("the binary was found a moment ago");
    let mut second = Command::new(exe)
        .env("APPDATA", &guard.home)
        .env("WSMF_INSTANCE", "single")
        .spawn()
        .expect("a second launch should at least start");

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match second.try_wait() {
            Ok(Some(_)) => break,
            _ if Instant::now() > deadline => {
                let _ = second.kill();
                panic!("the second copy stayed alive; the single instance check is broken");
            }
            _ => std::thread::sleep(Duration::from_millis(50)),
        }
    }
}

struct Counters {
    working_set: usize,
    cpu_seconds: f64,
}

fn counters(pid: u32) -> Option<Counters> {
    use windows::Win32::Foundation::{CloseHandle, FILETIME};
    use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    use windows::Win32::System::Threading::{
        GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
    };

    unsafe {
        let handle = OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ,
            false,
            pid,
        )
        .ok()?;

        let mut memory = PROCESS_MEMORY_COUNTERS {
            cb: size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            ..Default::default()
        };
        let memory_ok = GetProcessMemoryInfo(
            handle,
            &mut memory,
            size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        )
        .is_ok();

        let mut created = FILETIME::default();
        let mut exited = FILETIME::default();
        let mut kernel = FILETIME::default();
        let mut user = FILETIME::default();
        let times_ok =
            GetProcessTimes(handle, &mut created, &mut exited, &mut kernel, &mut user).is_ok();
        let _ = CloseHandle(handle);

        if !memory_ok || !times_ok {
            return None;
        }
        let to_seconds = |time: FILETIME| {
            let ticks = ((time.dwHighDateTime as u64) << 32) | time.dwLowDateTime as u64;
            ticks as f64 / 10_000_000.0
        };
        Some(Counters {
            working_set: memory.WorkingSetSize,
            cpu_seconds: to_seconds(kernel) + to_seconds(user),
        })
    }
}
