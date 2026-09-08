//! The half that unit tests cannot reach: real windows, real focus, real Win32.
//! Each test makes its own windows and cleans them up.

#![cfg(windows)]

use std::sync::atomic::{AtomicUsize, Ordering};
use tlk_wsmf::guard::force_foreground;
use tlk_wsmf::window::{WindowInfo, wide};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetForegroundWindow, MSG,
    PM_REMOVE, PeekMessageW, RegisterClassW, SW_HIDE, SW_MINIMIZE, SW_SHOWNORMAL,
    SetForegroundWindow, ShowWindow, TranslateMessage, WNDCLASSW, WS_OVERLAPPEDWINDOW,
};
use windows::core::PCWSTR;

static NEXT_CLASS: AtomicUsize = AtomicUsize::new(0);

struct TestWindow {
    hwnd: HWND,
}

impl TestWindow {
    fn new(title: &str) -> Self {
        let index = NEXT_CLASS.fetch_add(1, Ordering::Relaxed);
        let class_name = format!("WsmfTestWindow{index}");
        unsafe {
            let instance = GetModuleHandleW(None).unwrap_or_default();
            let class = wide(&class_name);
            let wc = WNDCLASSW {
                lpfnWndProc: Some(test_proc),
                hInstance: instance.into(),
                lpszClassName: PCWSTR(class.as_ptr()),
                ..Default::default()
            };
            RegisterClassW(&wc);
            let hwnd = CreateWindowExW(
                Default::default(),
                PCWSTR(class.as_ptr()),
                PCWSTR(wide(title).as_ptr()),
                WS_OVERLAPPEDWINDOW,
                40,
                40,
                420,
                260,
                None,
                None,
                Some(instance.into()),
                None,
            )
            .expect("a test window should be creatable");
            let _ = ShowWindow(hwnd, SW_SHOWNORMAL);
            pump();
            Self { hwnd }
        }
    }

    fn hide(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
        pump();
    }

    fn minimise(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_MINIMIZE);
        }
        pump();
    }

    fn focus(&self) {
        unsafe {
            let _ = SetForegroundWindow(self.hwnd);
        }
        pump();
    }

    fn info(&self) -> WindowInfo {
        WindowInfo::capture(self.hwnd).expect("a live window should describe itself")
    }
}

impl Drop for TestWindow {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
        pump();
    }
}

unsafe extern "system" fn test_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}

/// Windows will not update window state until the messages are drained.
fn pump() {
    unsafe {
        let mut message = MSG::default();
        for _ in 0..64 {
            if !PeekMessageW(&mut message, None, 0, 0, PM_REMOVE).as_bool() {
                break;
            }
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}

#[test]
fn a_window_describes_itself() {
    let window = TestWindow::new("A test window");
    let info = window.info();
    assert!(
        info.exe.ends_with(".exe"),
        "expected an executable name, got {:?}",
        info.exe
    );
    assert!(
        info.exe.chars().all(|c| !c.is_uppercase()),
        "names must be lower case for list matching: {:?}",
        info.exe
    );
    assert_eq!(info.title, "A test window");
    assert!(info.class.starts_with("WsmfTestWindow"));
    assert_eq!(info.pid, std::process::id());
    assert!(info.path.to_lowercase().ends_with(&info.exe));
}

#[test]
fn a_visible_window_can_take_focus() {
    let window = TestWindow::new("Visible");
    assert!(window.info().can_take_focus());
}

#[test]
fn a_hidden_window_cannot_take_focus() {
    let window = TestWindow::new("Hidden");
    let info = window.info();
    window.hide();
    assert!(
        !info.can_take_focus(),
        "focus must never be handed to a hidden window"
    );
}

#[test]
fn a_minimised_window_cannot_take_focus() {
    let window = TestWindow::new("Minimised");
    let info = window.info();
    window.minimise();
    assert!(
        !info.can_take_focus(),
        "restoring a window the user minimised is not a favour"
    );
}

#[test]
fn a_destroyed_window_cannot_take_focus() {
    let info = {
        let window = TestWindow::new("Gone in a moment");
        window.info()
    };
    assert!(!info.can_take_focus());
}

#[test]
fn a_system_window_is_recognised_as_noise() {
    // The desktop is always there, and is never a thief.
    let desktop = unsafe { windows::Win32::UI::WindowsAndMessaging::GetShellWindow() };
    if let Some(info) = WindowInfo::capture(desktop) {
        assert!(
            info.is_noise(),
            "the shell window should not count as an interruption, class was {:?}",
            info.class
        );
    }
}

#[test]
fn focus_can_be_handed_back_between_two_windows() {
    let first = TestWindow::new("First");
    let second = TestWindow::new("Second");

    second.focus();
    pump();
    let stolen = unsafe { GetForegroundWindow() };
    if stolen != second.hwnd {
        // Another application owns the foreground right now (a locked session, or
        // a CI runner with no interactive desktop). Nothing to assert about.
        eprintln!("skipped: this session does not control the foreground");
        return;
    }

    assert!(force_foreground(first.hwnd));
    pump();
    assert_eq!(
        unsafe { GetForegroundWindow() },
        first.hwnd,
        "focus should have gone back to the first window"
    );
}

#[test]
fn the_system_answers_what_we_ask_it() {
    // These are reads, not writes: no registry key is changed by running the tests.
    let timeout = tlk_wsmf::system::foreground_lock_timeout();
    assert!(
        timeout.is_some(),
        "Windows should report its foreground lock timeout"
    );

    // Either answer is correct; what matters is that it returns one rather than
    // panicking on a missing registry value.
    let _ = tlk_wsmf::system::dark_mode();
    let _ = tlk_wsmf::system::autostart_enabled();
}

#[test]
fn the_settings_path_is_written_the_way_a_person_would() {
    let label = tlk_wsmf::config::config_path_label();
    assert!(label.ends_with("config.toml"));
    if std::env::var("APPDATA").is_ok() {
        assert!(
            label.starts_with("%APPDATA%"),
            "the roaming folder should be shortened, got {label:?}"
        );
    }
}
