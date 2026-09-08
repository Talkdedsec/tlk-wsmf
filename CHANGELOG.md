# Changelog

Notable changes, newest first. Versions follow [semantic versioning](https://semver.org).

## [0.1.0] - 2026-09-08

First release.

### Added

- Records which application took the foreground, when, and why it counted as a theft
- Watch, guard and strict modes, switched from the tray or the panel
- Hands focus back to the window you were in, and flashes the interrupting window
  in the taskbar the way Windows does
- Quick view on a tray click: a Win32 list of what just happened
- Panel on a double click: activity, rules, statistics and settings, in its own
  process so the guard stays independent of it
- Statistics over the last fortnight: worst offenders, by day, by hour
- Block and allow lists, editable from the panel or by right-clicking a log line
- English and Turkish, following the Windows display language by default
- Turns the Windows foreground lock timeout back on
- Optional start with Windows, optional log file, `record_everything` for diagnosing
  a decision you disagree with

### Notes

- Not code signed yet, so SmartScreen warns on first run
- No keyboard hook and no network code, by design

[0.1.0]: https://github.com/Talkdedsec/tlk-wsmf/releases/tag/v0.1.0
