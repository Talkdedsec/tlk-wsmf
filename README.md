# Who Stole My Focus

You are typing. A window you did not ask for jumps in front and eats the next few
keystrokes. By the time you notice, half a sentence went somewhere else.

This is a small Windows tray program that answers two questions: **which application
did that**, and, if you want, **stops it from doing it again**.

[Türkçe](README.tr.md)

## Why this exists

Windows already has a defence against this. The foreground lock timeout is supposed
to stop background applications from calling `SetForegroundWindow` at will. Plenty of
installers quietly set it to `0`, and after that anything can interrupt anything.

The fix has been asked for in Microsoft's own PowerToys repository since 2019 and is
still not there. The registry key that controls it has no interface. The one third
party tool most people find is either abandoned or flagged by Defender.

## What it does

Three modes, switched from the tray icon:

- **Watch only** (default) — never touches your windows. Records every application
  that took your focus, when, and what its window was called. Start here.
- **Guard** — hands focus back if it was taken while you were typing, and always
  hands it back from applications you put on the block list.
- **Strict** — hands focus back from everything that is not on the allow list.

When it takes focus back it flashes the interrupting window in the taskbar, the same
way Windows does when it blocks a window itself. Nothing is lost. It just waits.

Right-click any line in the log to block that application, always allow it, or open
its folder to find out what it actually is.

## Install

Download `wsmf.exe` from [Releases](https://github.com/Talkdedsec/tlk-wsmf/releases)
and run it. One file, no installer, no runtime to install. It appears in the tray.

To have it start with Windows, tick that in the tray menu. To remove it: quit, delete
the exe, and delete `%APPDATA%\wsmf` if you want the log and settings gone too.

## How it decides

A window coming to the front is not automatically a theft — most of the time it is
you, clicking or pressing Alt+Tab. Getting that wrong is worse than the original
problem, so the benefit of the doubt always goes to you:

| Situation | What happens |
|---|---|
| You clicked in the last 400 ms | Left alone |
| A mouse button is down right now | Left alone |
| Alt+Tab or the Windows key is held | Left alone |
| Same application as the window you were in | Left alone |
| On your allow list | Left alone |
| A key was pressed in the last 1.5 s and none of the above | You were typing: this is a theft |
| On your block list | A theft, whatever you were doing |

If an application keeps grabbing focus, it wins after three tries in ten seconds.
Two programs fighting over the foreground is worse for you than one badly behaved
program, so this one stops and says so in the log.

## What it does not do

- It does not install a keyboard hook. It asks the system how long since the last
  input and whether a mouse button is down. It never sees which key you pressed.
- It does not touch the network. Nothing leaves your machine.
- It cannot take focus back from an elevated window if it is not elevated itself.
  Windows blocks that on purpose. Such a case is recorded as "could not take it back"
  rather than passed over in silence.
- It does not stop a window from *opening*. It decides where the keyboard goes.

## Antivirus

Reading foreground changes and moving focus is what this program is for, and it is
also what some malware does, so a heuristic engine may take an interest. Releases are
built by GitHub Actions from the tag they claim to be built from, and the workflow is
in this repository. If your scanner still flags it, the whole thing is about a
thousand lines of Rust and you can read all of it.

## Settings

The tray menu covers everything most people need. The rest lives in
`%APPDATA%\wsmf\config.toml`:

```toml
mode = "watch"              # watch, guard, strict
typing_window_ms = 1500     # a keystroke this recent means you were typing
click_grace_ms = 400        # a click this recent means you opened it yourself
blocklist = []              # exe names, lower case, no path
allowlist = ["explorer.exe"]
flash_thief = true          # blink the interrupting window in the taskbar
max_restores = 3            # give up after this many tries
restore_window_secs = 10
log_to_file = true          # %APPDATA%\wsmf\focus.log
record_everything = false   # also record what was left alone, and why
```

`record_everything` is the one to turn on when you disagree with a decision: it
writes down every focus change together with the reason it was allowed.

## Build

```
cargo build --release
```

Rust 1.85 or newer, MSVC toolchain. No other dependencies. The result is a single
420 KB executable at `target\release\wsmf.exe`.

`cargo test` runs the decision table above as unit tests.

## Licence

MIT.
