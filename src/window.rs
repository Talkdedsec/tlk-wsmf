use windows::Win32::Foundation::{CloseHandle, HWND, MAX_PATH};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClassNameW, GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId, IsIconic,
    IsWindow, IsWindowVisible,
};
use windows::Win32::UI::WindowsAndMessaging::{HICON, IDI_APPLICATION, LoadIconW};
use windows::core::{PCWSTR, PWSTR};

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub hwnd: HWND,
    pub pid: u32,
    pub exe: String,
    pub path: String,
    pub title: String,
    pub class: String,
}

impl WindowInfo {
    pub fn capture(hwnd: HWND) -> Option<Self> {
        if hwnd.is_invalid() || unsafe { !IsWindow(Some(hwnd)).as_bool() } {
            return None;
        }
        let mut pid = 0u32;
        let thread_id = unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
        if thread_id == 0 {
            return None;
        }
        let path = process_path(pid).unwrap_or_default();
        let exe = path
            .rsplit(['\\', '/'])
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        Some(Self {
            hwnd,
            pid,
            exe,
            path,
            title: window_text(hwnd),
            class: class_name(hwnd),
        })
    }

    /// Somewhere focus can actually go back to. A window that has closed, been
    /// hidden or been minimised is not it: handing focus to one of those either
    /// fails or drags something back up that the user put away.
    pub fn can_take_focus(&self) -> bool {
        unsafe {
            IsWindow(Some(self.hwnd)).as_bool()
                && IsWindowVisible(self.hwnd).as_bool()
                && !IsIconic(self.hwnd).as_bool()
        }
    }

    /// Windows the user never asked for and cannot act on: tooltips, the desktop
    /// itself, the taskbar's own transient surfaces. Never counted as a focus theft.
    pub fn is_noise(&self) -> bool {
        matches!(
            self.class.as_str(),
            "Progman"
                | "WorkerW"
                | "Shell_TrayWnd"
                | "Shell_SecondaryTrayWnd"
                | "tooltips_class32"
                | "Windows.UI.Core.CoreWindow"
                | "ForegroundStaging"
        ) || self.exe.is_empty()
    }
}

fn process_path(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; MAX_PATH as usize];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut len,
        )
        .is_ok();
        let _ = CloseHandle(handle);
        if ok {
            Some(String::from_utf16_lossy(&buf[..len as usize]))
        } else {
            None
        }
    }
}

fn window_text(hwnd: HWND) -> String {
    let mut buf = [0u16; 512];
    let len = unsafe { GetWindowTextW(hwnd, &mut buf) };
    if len <= 0 {
        String::new()
    } else {
        String::from_utf16_lossy(&buf[..len as usize])
    }
}

fn class_name(hwnd: HWND) -> String {
    let mut buf = [0u16; 256];
    let len = unsafe { GetClassNameW(hwnd, &mut buf) };
    if len <= 0 {
        String::new()
    } else {
        String::from_utf16_lossy(&buf[..len as usize])
    }
}

/// Whatever the user was working in when we started. Without this the first theft
/// after launch has nowhere to give focus back to.
pub fn current_foreground() -> Option<WindowInfo> {
    WindowInfo::capture(unsafe { GetForegroundWindow() })
}

/// The application icon, compiled into the executable as resource 1. Every window
/// class asks for it here rather than falling back to the generic Windows icon.
pub fn app_icon() -> HICON {
    unsafe {
        let module = GetModuleHandleW(None).unwrap_or_default().into();
        LoadIconW(Some(module), PCWSTR(std::ptr::without_provenance(1)))
            .or_else(|_| LoadIconW(None, IDI_APPLICATION))
            .unwrap_or_default()
    }
}

pub fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}
