# Contributing

## Reporting a wrong decision

This is the useful kind of report. Turn on `record_everything = true` in
`%APPDATA%\wsmf\config.toml`, restart, reproduce it, and paste the matching line
from `%APPDATA%\wsmf\focus.log`. The reason column says which rule matched, which
is usually the whole answer.

## Building

```
cargo build --release
cargo test --release
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Rust 1.85 or newer with the MSVC toolchain. Nothing else to install.

## Where things are

- `policy.rs` — the decision, as a pure function. Every branch has a test. A change
  here needs a test that fails before it and passes after.
- `guard.rs` — the Win32 side: the foreground hook, and handing focus back.
- `journal.rs` / `history.rs` — the log format and reading it back. The keys written
  to disk are never translated; they are a file format.
- `i18n.rs` — both languages in one struct, so a missing translation will not compile.
- `panel/` — the egui window, run as its own process.
- `viewer.rs` — the Win32 quick view.

## What a change should not do

- Add a keyboard hook. `GetAsyncKeyState` and `GetLastInputInfo` are enough, and a
  `WH_KEYBOARD_LL` hook is what gets tools like this one flagged as malware.
- Take focus back in a case where the user might have done it themselves. When in
  doubt, the user wins; that is the whole design.
- Make the guard process depend on the panel. They are separate on purpose.
- Send anything anywhere. There is no network code in this program.

## Style

Match what is there. Comments explain a constraint the code cannot show, not what
the next line does.
