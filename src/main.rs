#![windows_subsystem = "windows"]

mod app;
mod config;
mod guard;
mod input;
mod journal;
mod policy;
mod system;
mod tray;
mod viewer;
mod window;

use crate::app::{App, install, take_app, with_app};
use crate::config::Config;
use crate::window::wide;
use windows::Win32::Foundation::{
    ERROR_ALREADY_EXISTS, GetLastError, HWND, LPARAM, LRESULT, WPARAM,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, FindWindowW, GetMessageW, KillTimer, MSG,
    PostMessageW, RegisterClassW, SetTimer, TranslateMessage, WM_APP, WM_COMMAND, WM_DESTROY,
    WM_TIMER, WNDCLASSW, WS_OVERLAPPED,
};
use windows::core::PCWSTR;

const CLASS_NAME: &str = "WsmfHiddenWindow";
const MUTEX_NAME: &str = "Global\\tlk-wsmf-single-instance";
const INPUT_TIMER: usize = 1;
const INPUT_POLL_MS: u32 = 50;
/// Sent by a second launch to the copy already running.
const WM_SHOW_VIEWER: u32 = WM_APP + 2;

fn main() {
    if another_copy_is_running() {
        wake_the_running_copy();
        return;
    }

    let first_run = !config::config_path().exists();
    let cfg = Config::load();
    let mut state = App::new(cfg);

    let hwnd = create_hidden_window();
    if hwnd.is_invalid() {
        return;
    }
    state.tray_window = hwnd;
    state.previous = window::current_foreground();
    state.hook = guard::install_hook();
    if state.cfg.start_with_windows && !system::autostart_enabled() {
        system::set_autostart(true);
    }
    install(state);

    if !tray::add(hwnd) {
        // No tray icon means no way to reach the program. Better to stop than to
        // sit invisible in the process list.
        return;
    }
    with_app(|app| {
        tray::refresh_tooltip(app);
        if first_run {
            tray::say(
                app.tray_window,
                "Running in the tray",
                "Watching which application takes your focus. Right-click the tray icon to see who, or to turn guarding on.",
            );
        }
    });

    unsafe {
        SetTimer(Some(hwnd), INPUT_TIMER, INPUT_POLL_MS, None);
        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        let _ = KillTimer(Some(hwnd), INPUT_TIMER);
    }

    tray::remove(hwnd);
    if let Some(state) = take_app() {
        guard::remove_hook(state.hook);
        let _ = state.cfg.save();
    }
}

fn another_copy_is_running() -> bool {
    unsafe {
        let name = wide(MUTEX_NAME);
        let handle = CreateMutexW(None, true, PCWSTR(name.as_ptr()));
        if handle.is_err() {
            return false;
        }
        // The handle is deliberately leaked: it must outlive main, and the process
        // exiting is what releases it.
        GetLastError() == ERROR_ALREADY_EXISTS
    }
}

fn wake_the_running_copy() {
    unsafe {
        if let Ok(existing) = FindWindowW(PCWSTR(wide(CLASS_NAME).as_ptr()), PCWSTR::null()) {
            let _ = PostMessageW(Some(existing), WM_SHOW_VIEWER, WPARAM(0), LPARAM(0));
        }
    }
}

fn create_hidden_window() -> HWND {
    unsafe {
        let instance = GetModuleHandleW(None).unwrap_or_default();
        let class = wide(CLASS_NAME);
        let wc = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(class.as_ptr()),
            ..Default::default()
        };
        RegisterClassW(&wc);
        CreateWindowExW(
            Default::default(),
            PCWSTR(class.as_ptr()),
            PCWSTR(wide("Who Stole My Focus").as_ptr()),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(instance.into()),
            None,
        )
        .unwrap_or_default()
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        tray::WM_TRAY => {
            with_app(|app| tray::tray_message(app, lparam));
            LRESULT(0)
        }
        WM_SHOW_VIEWER => {
            with_app(viewer::show);
            LRESULT(0)
        }
        WM_COMMAND => {
            let id = tray::command_id(wparam);
            with_app(|app| tray::handle_command(app, id));
            LRESULT(0)
        }
        WM_TIMER => {
            with_app(|app| {
                app.input.poll();
                if app.paused_until.is_some() && !app.is_paused() {
                    app.paused_until = None;
                    tray::refresh_tooltip(app);
                }
            });
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe { windows::Win32::UI::WindowsAndMessaging::PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}
