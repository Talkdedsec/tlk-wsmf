use crate::app::App;
use crate::config::Mode;
use crate::system;
use crate::window::wide;
use std::time::{Duration, Instant};
use windows::Win32::Foundation::{HWND, LPARAM, POINT, WPARAM};
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_TIP, NIIF_NONE, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, DestroyMenu, GetCursorPos, HICON, IDI_APPLICATION, LoadIconW,
    MF_CHECKED, MF_GRAYED, MF_SEPARATOR, MF_STRING, SetForegroundWindow, TPM_BOTTOMALIGN,
    TPM_RIGHTBUTTON, TrackPopupMenu, WM_APP,
};
use windows::core::PCWSTR;

pub const WM_TRAY: u32 = WM_APP + 1;
pub const TRAY_ID: u32 = 1;

pub const ID_MODE_WATCH: usize = 10;
pub const ID_MODE_GUARD: usize = 11;
pub const ID_MODE_STRICT: usize = 12;
pub const ID_SHOW_LOG: usize = 20;
pub const ID_OPEN_CONFIG: usize = 21;
pub const ID_PAUSE: usize = 30;
pub const ID_AUTOSTART: usize = 31;
pub const ID_LOCK_TIMEOUT: usize = 32;
pub const ID_CLEAR_LOG: usize = 22;
pub const ID_QUIT: usize = 99;

const PAUSE_MINUTES: u64 = 15;

fn app_icon() -> HICON {
    unsafe {
        LoadIconW(
            Some(
                windows::Win32::System::LibraryLoader::GetModuleHandleW(None)
                    .unwrap_or_default()
                    .into(),
            ),
            PCWSTR(std::ptr::without_provenance(1)),
        )
        .or_else(|_| LoadIconW(None, IDI_APPLICATION))
        .unwrap_or_default()
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

fn copy_into(target: &mut [u16], text: &str) {
    let source: Vec<u16> = text.encode_utf16().take(target.len() - 1).collect();
    target[..source.len()].copy_from_slice(&source);
}

pub fn refresh_tooltip(app: &App) {
    if app.tray_window.is_invalid() {
        return;
    }
    let caught = app.journal.thefts_today();
    let tip = if app.is_paused() {
        "Who Stole My Focus - paused".to_string()
    } else if caught == 0 {
        format!(
            "Who Stole My Focus - {}, nothing caught today",
            app.cfg.mode.label()
        )
    } else if caught == 1 {
        format!(
            "Who Stole My Focus - {}, 1 caught today",
            app.cfg.mode.label()
        )
    } else {
        format!(
            "Who Stole My Focus - {}, {caught} caught today",
            app.cfg.mode.label()
        )
    };
    let data = icon_data(app.tray_window, &tip);
    unsafe {
        let _ = Shell_NotifyIconW(NIM_MODIFY, &data);
    }
}

pub fn show_menu(app: &mut App) {
    let hwnd = app.tray_window;
    unsafe {
        let Ok(menu) = CreatePopupMenu() else {
            return;
        };
        let item = |flags, id: usize, text: &str| {
            let _ = AppendMenuW(menu, flags, id, PCWSTR(wide(text).as_ptr()));
        };

        item(MF_STRING | MF_GRAYED, 0, "Who Stole My Focus");
        item(MF_SEPARATOR, 0, "");

        let checked = |on: bool| {
            if on {
                MF_STRING | MF_CHECKED
            } else {
                MF_STRING
            }
        };
        item(
            checked(app.cfg.mode == Mode::Watch),
            ID_MODE_WATCH,
            "Watch only - just tell me who",
        );
        item(
            checked(app.cfg.mode == Mode::Guard),
            ID_MODE_GUARD,
            "Guard - take focus back while I type",
        );
        item(
            checked(app.cfg.mode == Mode::Strict),
            ID_MODE_STRICT,
            "Strict - take it back from everything",
        );
        item(MF_SEPARATOR, 0, "");

        let seen = app.journal.len();
        item(
            MF_STRING,
            ID_SHOW_LOG,
            &format!("Show the log ({seen} recorded)"),
        );
        item(MF_STRING, ID_CLEAR_LOG, "Clear what has been recorded");
        item(MF_STRING, ID_OPEN_CONFIG, "Open the settings file");
        item(MF_SEPARATOR, 0, "");

        item(
            MF_STRING,
            ID_PAUSE,
            if app.is_paused() {
                "Resume now"
            } else {
                "Pause for 15 minutes"
            },
        );
        item(
            checked(system::autostart_enabled()),
            ID_AUTOSTART,
            "Start with Windows",
        );

        let timeout = system::foreground_lock_timeout().unwrap_or(0);
        let lock_label = if timeout >= app.cfg.foreground_lock_timeout_ms {
            format!("Windows focus lock is on ({timeout} ms)")
        } else {
            format!("Turn the Windows focus lock back on (now {timeout} ms)")
        };
        item(MF_STRING, ID_LOCK_TIMEOUT, &lock_label);
        item(MF_SEPARATOR, 0, "");
        item(MF_STRING, ID_QUIT, "Quit");

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
        ID_SHOW_LOG => crate::viewer::show(app),
        ID_CLEAR_LOG => {
            app.journal.clear();
            crate::viewer::refresh(app);
            refresh_tooltip(app);
        }
        ID_OPEN_CONFIG => open_in_shell(&crate::config::config_path().to_string_lossy()),
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
                let _ = app.cfg.save();
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
    let _ = app.cfg.save();
    refresh_tooltip(app);
}

pub fn open_in_shell(path: &str) {
    unsafe {
        windows::Win32::UI::Shell::ShellExecuteW(
            None,
            PCWSTR(wide("open").as_ptr()),
            PCWSTR(wide(path).as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL,
        );
    }
}

pub fn tray_message(app: &mut App, lparam: LPARAM) {
    const WM_LBUTTONUP: u32 = 0x0202;
    const WM_RBUTTONUP: u32 = 0x0205;
    match lparam.0 as u32 {
        WM_LBUTTONUP => crate::viewer::show(app),
        WM_RBUTTONUP => show_menu(app),
        _ => {}
    }
}

pub fn command_id(wparam: WPARAM) -> usize {
    wparam.0 & 0xFFFF
}
