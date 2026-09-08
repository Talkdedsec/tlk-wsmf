# Changelog

Notable changes, newest first. Versions follow [semantic versioning](https://semver.org).

## [0.1.1] - 2026-09-08

Everything here was found by looking at the program rather than at the code.

### Fixed

- The settings page did not fit the window it opens in, and the last card was cut
  off. It uses two columns when there is room, and the labels are shorter with the
  explanation on hover
- The quick view opened undersized on a scaled display: the window size was in
  physical pixels with no allowance for DPI
- The Windows focus lock was reported as a raw millisecond count, so a machine with
  it set to never time out read `2147483647 ms`
- A launching application produced one interruption per window it opened - three
  restores in seventeen milliseconds for a single launch, which is a flicker rather
  than a defence. Windows from the same process are now ignored for 400 ms after a
  restore

### Added

- Tests for the strike counter and its expiry, the settling window, an expired
  pause, and the activity filter and search: 68 in total
- `--tab` and `--size` for the panel, and `WSMF_THEME`, so the screenshots can be
  taken without touching the screen or the machine's theme

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

[0.1.1]: https://github.com/Talkdedsec/tlk-wsmf/releases/tag/v0.1.1
[0.1.0]: https://github.com/Talkdedsec/tlk-wsmf/releases/tag/v0.1.0
