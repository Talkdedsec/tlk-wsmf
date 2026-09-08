//! Two things Windows already offers but hides from anyone without a registry editor:
//! the foreground lock timeout, and starting with the session.

use crate::window::wide;
use std::ffi::c_void;
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ, RegCloseKey, RegDeleteValueW,
    RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    SPI_GETFOREGROUNDLOCKTIMEOUT, SPI_SETFOREGROUNDLOCKTIMEOUT, SPIF_SENDCHANGE,
    SPIF_UPDATEINIFILE, SystemParametersInfoW,
};
use windows::core::PCWSTR;

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const RUN_VALUE: &str = "wsmf";

/// Milliseconds an application must wait, after the user's last interaction with it,
/// before Windows lets it call SetForegroundWindow. Windows ships with 200000.
/// Plenty of installers quietly set it to 0, which is where the whole problem starts.
pub fn foreground_lock_timeout() -> Option<u32> {
    let mut value: u32 = 0;
    unsafe {
        SystemParametersInfoW(
            SPI_GETFOREGROUNDLOCKTIMEOUT,
            0,
            Some(&mut value as *mut u32 as *mut c_void),
            SPIF_UPDATEINIFILE, // ignored for a read
        )
        .ok()?;
    }
    Some(value)
}

pub fn set_foreground_lock_timeout(ms: u32) -> bool {
    unsafe {
        SystemParametersInfoW(
            SPI_SETFOREGROUNDLOCKTIMEOUT,
            0,
            Some(ms as usize as *mut c_void),
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        )
        .is_ok()
    }
}

/// Whether Windows is currently in dark mode, so the quick view can match it.
pub fn dark_mode() -> bool {
    unsafe {
        let mut key = HKEY::default();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(wide(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize").as_ptr()),
            None,
            KEY_READ,
            &mut key,
        ) != ERROR_SUCCESS
        {
            return false;
        }
        let mut value: u32 = 1;
        let mut size = size_of::<u32>() as u32;
        let read = RegQueryValueExW(
            key,
            PCWSTR(wide("AppsUseLightTheme").as_ptr()),
            None,
            None,
            Some(&mut value as *mut u32 as *mut u8),
            Some(&mut size),
        );
        let _ = RegCloseKey(key);
        read == ERROR_SUCCESS && value == 0
    }
}

pub fn autostart_enabled() -> bool {
    unsafe {
        let mut key = HKEY::default();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(wide(RUN_KEY).as_ptr()),
            None,
            KEY_READ,
            &mut key,
        ) != ERROR_SUCCESS
        {
            return false;
        }
        let mut size = 0u32;
        let found = RegQueryValueExW(
            key,
            PCWSTR(wide(RUN_VALUE).as_ptr()),
            None,
            None,
            None,
            Some(&mut size),
        ) == ERROR_SUCCESS;
        let _ = RegCloseKey(key);
        found
    }
}

pub fn set_autostart(on: bool) -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    let command = format!("\"{}\"", exe.display());
    unsafe {
        let mut key = HKEY::default();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(wide(RUN_KEY).as_ptr()),
            None,
            KEY_WRITE,
            &mut key,
        ) != ERROR_SUCCESS
        {
            return false;
        }
        let name = wide(RUN_VALUE);
        let result = if on {
            let value = wide(&command);
            let bytes = std::slice::from_raw_parts(
                value.as_ptr() as *const u8,
                value.len() * size_of::<u16>(),
            );
            RegSetValueExW(key, PCWSTR(name.as_ptr()), None, REG_SZ, Some(bytes))
        } else {
            RegDeleteValueW(key, PCWSTR(name.as_ptr()))
        };
        let _ = RegCloseKey(key);
        result == ERROR_SUCCESS
    }
}
