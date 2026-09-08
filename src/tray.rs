use crate::app::App;
use crate::config::Mode;
use crate::system;
use crate::window::{app_icon, wide};
use std::time::{Duration, Instant};
use windows::Win32::Foundation::{HWND, LPARAM, POINT, WPARAM};
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_TIP, NIIF_NONE, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, DestroyMenu, GetCursorPos, MF_CHECKED, MF_GRAYED, MF_SEPARATOR,
    MF_STRING, SetForegroundWindow, TPM_BOTTOMALIGN, TPM_RIGHTBUTTON, TrackPopupMenu, WM_APP,
};
use windows::core::PCWSTR;

pub const WM_TRAY: u32 = WM_APP + 1;
pub const TRAY_ID: u32 = 1;

pub const ID_MODE_WATCH: usize = 10;
pub const ID_MODE_GUARD: usize = 11;
pub const ID_MODE_STRICT: usize = 12;
pub const ID_OPEN_PANEL: usize = 20;
pub const ID_QUICK_VIEW: usize = 21;
pub const ID_CLEAR_LOG: usize = 22;
pub const ID_PAUSE: usize = 30;
pub const ID_AUTOSTART: usize = 31;
pub const ID_LOCK_TIMEOUT: usize = 32;
pub const ID_QUIT: usize = 99;

const PAUSE_MINUTES: u64 = 15;

fn copy_into(target: &mut [u16], text: &str) {
    let source: Vec<u16> = text.encode_utf16().take(target.len() - 1).collect();
    target[..source.len()].copy_from_slice(&source);
    if source.len() < target.len() {
        target[source.len()] = 0;
    }
}

fn icon_data(hwnd: HWND, tip: &str) -> NOTIFYICONDATAW {
    let mut data = NOTIFYICONDATAW {
        cbSize: size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ID,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
        uCallbackMessage: WM_TRAY,
        hIcon: app_icon(),
        ..Default::default()
    };
    copy_into(&mut data.szTip, tip);
    data
}

pub fn add(hwnd: HWND) -> bool {
    let data = icon_data(hwnd, "Who Stole My Focus");
    unsafe { Shell_NotifyIconW(NIM_ADD, &data).as_bool() }
}

pub fn remove(hwnd: HWND) {
    let data = NOTIFYICONDATAW {
        cbSize: size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ID,
        ..Default::default()
    };
    unsafe {
        let _ = Shell_NotifyIconW(NIM_DELETE, &data);
    }
}

/// A balloon, not a dialog: the first run should explain itself without stealing
/// focus. A program that fights focus theft has no business committing one.
pub fn say(hwnd: HWND, title: &str, body: &str) {
    let mut data = icon_data(hwnd, "Who Stole My Focus");
    data.uFlags |= NIF_INFO;
    data.Anonymous.uTimeout = 10_000;
    data.dwInfoFlags = NIIF_NONE;
    copy_into(&mut data.szInfoTitle, title);
    copy_into(&mut data.szInfo, body);
    unsafe {
        let _ = Shell_NotifyIconW(NIM_MODIFY, &data);
    }
}

pub fn refresh_tooltip(app: &App) {
    if app.tray_window.is_invalid() {
        return;
    }
    let s = app.cfg.strings();
    let caught = app.journal.interruptions_today();
    let mode = app.cfg.mode.label(s);
    let tip = if app.is_paused() {
        format!("{} - {}", s.app_name, s.tray_paused)
    } else if caught == 0 {
        format!("{} - {mode}, {}", s.app_name, s.tray_caught_none)
    } else if caught == 1 {
        format!("{} - {mode}, {}", s.app_name, s.tray_caught_one)
    } else {
        format!("{} - {mode}, {caught} {}", s.app_name, s.tray_caught_many)
    };
    let data = icon_data(app.tray_window, &tip);
    unsafe {
        let _ = Shell_NotifyIconW(NIM_MODIFY, &data);
    }
}

pub fn show_menu(app: &mut App) {
    let hwnd = app.tray_window;
    let s = app.cfg.strings();
    unsafe {
        let Ok(menu) = CreatePopupMenu() else {
            return;
        };
        let item = |flags, id: usize, text: &str| {
            let _ = AppendMenuW(menu, flags, id, PCWSTR(wide(text).as_ptr()));
        };
        let checked = |on: bool| {
            if on {
                MF_STRING | MF_CHECKED
            } else {
                MF_STRING
            }
        };

        item(MF_STRING | MF_GRAYED, 0, s.app_name);
        item(MF_SEPARATOR, 0, "");

        item(
            checked(app.cfg.mode == Mode::Watch),
            ID_MODE_WATCH,
            s.mode_watch,
        );
        item(
            checked(app.cfg.mode == Mode::Guard),
            ID_MODE_GUARD,
            s.mode_guard,
        );
        item(
            checked(app.cfg.mode == Mode::Strict),
            ID_MODE_STRICT,
            s.mode_strict,
        );
        item(MF_SEPARATOR, 0, "");

        item(MF_STRING, ID_OPEN_PANEL, s.tray_open_panel);
        item(
            MF_STRING,
            ID_QUICK_VIEW,
            &format!("{} ({})", s.tray_quick_view, app.journal.len()),
        );
        item(MF_STRING, ID_CLEAR_LOG, s.tray_clear);
        item(MF_SEPARATOR, 0, "");

        item(
            MF_STRING,
            ID_PAUSE,
            if app.is_paused() {
                s.tray_resume
            } else {
                s.tray_pause
            },
        );
        item(
            checked(system::autostart_enabled()),
            ID_AUTOSTART,
            s.settings_autostart,
        );

        let timeout = system::foreground_lock_timeout().unwrap_or(0);
        let on = timeout >= app.cfg.foreground_lock_timeout_ms;
        item(
            checked(on),
            ID_LOCK_TIMEOUT,
            &format!(
                "{} ({})",
                s.settings_windows_lock,
                if on {
                    s.settings_windows_lock_on
                } else {
                    s.settings_windows_lock_off
                }
            ),
        );
        item(MF_SEPARATOR, 0, "");
        item(MF_STRING, ID_QUIT, s.tray_quit);

        let mut point = POINT::default();
        let _ = GetCursorPos(&mut point);
        // Without this the menu will not close when the user clicks elsewhere.
        let _ = SetForegroundWindow(hwnd);
        let _ = TrackPopupMenu(
            menu,
            TPM_RIGHTBUTTON | TPM_BOTTOMALIGN,
            point.x,
            point.y,
            None,
            hwnd,
            None,
        );
        let _ = DestroyMenu(menu);
    }
}

pub fn handle_command(app: &mut App, id: usize) {
    match id {
        ID_MODE_WATCH => set_mode(app, Mode::Watch),
        ID_MODE_GUARD => set_mode(app, Mode::Guard),
        ID_MODE_STRICT => set_mode(app, Mode::Strict),
        ID_OPEN_PANEL => open_panel(),
        ID_QUICK_VIEW => crate::viewer::show(app),
        ID_CLEAR_LOG => {
            app.journal.clear();
            crate::viewer::refresh(app);
            refresh_tooltip(app);
        }
        ID_PAUSE => {
            app.paused_until = if app.is_paused() {
                None
            } else {
                Some(Instant::now() + Duration::from_secs(PAUSE_MINUTES * 60))
            };
            refresh_tooltip(app);
        }
        ID_AUTOSTART => {
            let now_on = system::autostart_enabled();
            if system::set_autostart(!now_on) {
                app.cfg.start_with_windows = !now_on;
                app.save_config();
            }
        }
        ID_LOCK_TIMEOUT => {
            let wanted = app.cfg.foreground_lock_timeout_ms;
            let current = system::foreground_lock_timeout().unwrap_or(0);
            let target = if current >= wanted { 0 } else { wanted };
            system::set_foreground_lock_timeout(target);
        }
        ID_QUIT => unsafe {
            windows::Win32::UI::WindowsAndMessaging::PostQuitMessage(0);
        },
        _ => {}
    }
}

fn set_mode(app: &mut App, mode: Mode) {
    app.cfg.mode = mode;
    app.save_config();
    refresh_tooltip(app);
}

/// The panel is the same executable in a different mode. Running it as its own
/// process keeps the part that guards your focus small and independent of the
/// part that draws charts.
pub fn open_panel() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let _ = std::process::Command::new(exe).arg("--panel").spawn();
}

pub fn tray_message(app: &mut App, lparam: LPARAM) {
    const WM_LBUTTONUP: u32 = 0x0202;
    const WM_LBUTTONDBLCLK: u32 = 0x0203;
    const WM_RBUTTONUP: u32 = 0x0205;
    match lparam.0 as u32 {
        WM_LBUTTONUP => crate::viewer::show(app),
        WM_LBUTTONDBLCLK => open_panel(),
        WM_RBUTTONUP => show_menu(app),
        _ => {}
    }
}

pub fn command_id(wparam: WPARAM) -> usize {
    wparam.0 & 0xFFFF
}
