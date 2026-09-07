//! The window that answers the question in the product's name.

use crate::app::{App, with_app};
use crate::window::wide;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{CreateFontIndirectW, HFONT};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::{
    ICC_LISTVIEW_CLASSES, INITCOMMONCONTROLSEX, InitCommonControlsEx, LVCF_SUBITEM, LVCF_TEXT,
    LVCF_WIDTH, LVCOLUMNW, LVIF_TEXT, LVITEMW, LVM_DELETEALLITEMS, LVM_GETNEXTITEM,
    LVM_INSERTCOLUMNW, LVM_INSERTITEMW, LVM_SETEXTENDEDLISTVIEWSTYLE, LVM_SETITEMTEXTW,
    LVM_SETTEXTCOLOR, LVNI_SELECTED, LVS_EX_DOUBLEBUFFER, LVS_EX_FULLROWSELECT, LVS_REPORT,
    LVS_SHOWSELALWAYS, LVS_SINGLESEL, NM_RCLICK, NMHDR,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CW_USEDEFAULT, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu,
    GetClientRect, GetCursorPos, GetSystemMetrics, HMENU, MF_SEPARATOR, MF_STRING, MoveWindow,
    NONCLIENTMETRICSW, RegisterClassW, SM_CXSCREEN, SM_CYSCREEN, SPI_GETNONCLIENTMETRICS,
    SW_SHOWNORMAL, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SendMessageW, SetForegroundWindow,
    ShowWindow, SystemParametersInfoW, TPM_RIGHTBUTTON, TrackPopupMenu, WM_CLOSE, WM_COMMAND,
    WM_NOTIFY, WM_SETFONT, WM_SIZE, WNDCLASSW, WS_CHILD, WS_EX_CLIENTEDGE, WS_OVERLAPPEDWINDOW,
    WS_VISIBLE,
};
use windows::core::PCWSTR;

const CLASS_NAME: &str = "WsmfViewer";
const ID_LIST: isize = 200;
const ID_BLOCK: usize = 301;
const ID_ALLOW: usize = 302;
const ID_FORGET: usize = 303;
const ID_REVEAL: usize = 304;

const WIDTH: i32 = 900;
const HEIGHT: i32 = 520;

pub fn show(app: &mut App) {
    if app.viewer.is_invalid() {
        app.viewer = create_window();
    }
    if app.viewer.is_invalid() {
        return;
    }
    fill(app);
    unsafe {
        let _ = ShowWindow(app.viewer, SW_SHOWNORMAL);
        let _ = SetForegroundWindow(app.viewer);
    }
}

pub fn refresh(app: &mut App) {
    if app.viewer.is_invalid() {
        return;
    }
    if unsafe { windows::Win32::UI::WindowsAndMessaging::IsWindowVisible(app.viewer).as_bool() } {
        fill(app);
    }
}

fn create_window() -> HWND {
    unsafe {
        let controls = INITCOMMONCONTROLSEX {
            dwSize: size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_LISTVIEW_CLASSES,
        };
        let _ = InitCommonControlsEx(&controls);

        let instance = GetModuleHandleW(None).unwrap_or_default();
        let class = wide(CLASS_NAME);
        let wc = WNDCLASSW {
            lpfnWndProc: Some(viewer_proc),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(class.as_ptr()),
            hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH(
                (windows::Win32::Graphics::Gdi::COLOR_WINDOW.0 + 1) as isize as *mut _,
            ),
            ..Default::default()
        };
        RegisterClassW(&wc);

        let x = (GetSystemMetrics(SM_CXSCREEN) - WIDTH) / 2;
        let y = (GetSystemMetrics(SM_CYSCREEN) - HEIGHT) / 2;
        let hwnd = CreateWindowExW(
            Default::default(),
            PCWSTR(class.as_ptr()),
            PCWSTR(wide("Who Stole My Focus").as_ptr()),
            WS_OVERLAPPEDWINDOW,
            if x > 0 { x } else { CW_USEDEFAULT },
            if y > 0 { y } else { CW_USEDEFAULT },
            WIDTH,
            HEIGHT,
            None,
            None,
            Some(instance.into()),
            None,
        );
        hwnd.unwrap_or_default()
    }
}

fn list_of(hwnd: HWND) -> HWND {
    unsafe {
        windows::Win32::UI::WindowsAndMessaging::GetDlgItem(Some(hwnd), ID_LIST as i32)
            .unwrap_or_default()
    }
}

fn create_list(parent: HWND) -> HWND {
    unsafe {
        let instance = GetModuleHandleW(None).unwrap_or_default();
        let Ok(list) = CreateWindowExW(
            WS_EX_CLIENTEDGE,
            PCWSTR(wide("SysListView32").as_ptr()),
            PCWSTR::null(),
            WS_CHILD
                | WS_VISIBLE
                | windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(
                    LVS_REPORT | LVS_SINGLESEL | LVS_SHOWSELALWAYS,
                ),
            0,
            0,
            0,
            0,
            Some(parent),
            Some(HMENU(ID_LIST as *mut _)),
            Some(instance.into()),
            None,
        ) else {
            return HWND::default();
        };

        SendMessageW(
            list,
            LVM_SETEXTENDEDLISTVIEWSTYLE,
            Some(WPARAM(0)),
            Some(LPARAM(
                (LVS_EX_FULLROWSELECT | LVS_EX_DOUBLEBUFFER) as isize,
            )),
        );
        SendMessageW(
            list,
            LVM_SETTEXTCOLOR,
            Some(WPARAM(0)),
            Some(LPARAM(COLORREF(0x00202020).0 as isize)),
        );
        if let Some(font) = message_font() {
            SendMessageW(
                list,
                WM_SETFONT,
                Some(WPARAM(font.0 as usize)),
                Some(LPARAM(1)),
            );
        }

        for (index, (title, width)) in [
            ("When", 140),
            ("Application", 190),
            ("What happened", 150),
            ("Why", 170),
            ("Window title", 230),
        ]
        .iter()
        .enumerate()
        {
            let text = wide(title);
            let column = LVCOLUMNW {
                mask: LVCF_TEXT | LVCF_WIDTH | LVCF_SUBITEM,
                cx: *width,
                pszText: windows::core::PWSTR(text.as_ptr() as *mut u16),
                iSubItem: index as i32,
                ..Default::default()
            };
            SendMessageW(
                list,
                LVM_INSERTCOLUMNW,
                Some(WPARAM(index)),
                Some(LPARAM(&column as *const _ as isize)),
            );
        }
        list
    }
}

fn message_font() -> Option<HFONT> {
    unsafe {
        let mut metrics = NONCLIENTMETRICSW {
            cbSize: size_of::<NONCLIENTMETRICSW>() as u32,
            ..Default::default()
        };
        SystemParametersInfoW(
            SPI_GETNONCLIENTMETRICS,
            metrics.cbSize,
            Some(&mut metrics as *mut _ as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
        .ok()?;
        CreateFontIndirectW(&metrics.lfMessageFont).into()
    }
}

fn fill(app: &mut App) {
    let list = list_of(app.viewer);
    if list.is_invalid() {
        return;
    }
    unsafe {
        SendMessageW(list, LVM_DELETEALLITEMS, None, None);
        for (row, entry) in app.journal.entries().enumerate() {
            let when = wide(&entry.at.format("%d.%m %H:%M:%S").to_string());
            let item = LVITEMW {
                mask: LVIF_TEXT,
                iItem: row as i32,
                iSubItem: 0,
                pszText: windows::core::PWSTR(when.as_ptr() as *mut u16),
                ..Default::default()
            };
            SendMessageW(
                list,
                LVM_INSERTITEMW,
                Some(WPARAM(0)),
                Some(LPARAM(&item as *const _ as isize)),
            );
            let columns = [
                entry.exe.clone(),
                entry.verdict.label().to_string(),
                entry.reason.to_string(),
                entry.title.clone(),
            ];
            for (offset, text) in columns.iter().enumerate() {
                let value = wide(text);
                let sub = LVITEMW {
                    mask: LVIF_TEXT,
                    iItem: row as i32,
                    iSubItem: offset as i32 + 1,
                    pszText: windows::core::PWSTR(value.as_ptr() as *mut u16),
                    ..Default::default()
                };
                SendMessageW(
                    list,
                    LVM_SETITEMTEXTW,
                    Some(WPARAM(row)),
                    Some(LPARAM(&sub as *const _ as isize)),
                );
            }
        }
    }
}

fn selected_entry(app: &App) -> Option<(String, String)> {
    let list = list_of(app.viewer);
    if list.is_invalid() {
        return None;
    }
    let index = unsafe {
        SendMessageW(
            list,
            LVM_GETNEXTITEM,
            Some(WPARAM(usize::MAX)),
            Some(LPARAM(LVNI_SELECTED as isize)),
        )
        .0
    };
    if index < 0 {
        return None;
    }
    app.journal
        .entries()
        .nth(index as usize)
        .map(|entry| (entry.exe.clone(), entry.path.clone()))
}

fn row_menu(app: &mut App) {
    let Some((exe, path)) = selected_entry(app) else {
        return;
    };
    unsafe {
        let Ok(menu) = CreatePopupMenu() else {
            return;
        };
        let label = |id: usize, text: String| {
            let _ = AppendMenuW(menu, MF_STRING, id, PCWSTR(wide(&text).as_ptr()));
        };
        label(ID_BLOCK, format!("Always take focus back from {exe}"));
        label(ID_ALLOW, format!("Never touch {exe}"));
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
        label(ID_FORGET, format!("Forget the rule for {exe}"));
        if !path.is_empty() {
            let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
            label(ID_REVEAL, "Show me the file".to_string());
        }

        let mut point = POINT::default();
        let _ = GetCursorPos(&mut point);
        let _ = SetForegroundWindow(app.viewer);
        let _ = TrackPopupMenu(
            menu,
            TPM_RIGHTBUTTON,
            point.x,
            point.y,
            None,
            app.viewer,
            None,
        );
        let _ = DestroyMenu(menu);
    }
}

unsafe extern "system" fn viewer_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        windows::Win32::UI::WindowsAndMessaging::WM_CREATE => {
            create_list(hwnd);
            LRESULT(0)
        }
        WM_SIZE => {
            let list = list_of(hwnd);
            if !list.is_invalid() {
                let mut rect = RECT::default();
                unsafe {
                    let _ = GetClientRect(hwnd, &mut rect);
                    let _ = MoveWindow(list, 0, 0, rect.right, rect.bottom, true);
                }
            }
            LRESULT(0)
        }
        WM_NOTIFY => {
            let header = unsafe { &*(lparam.0 as *const NMHDR) };
            if header.code == NM_RCLICK {
                with_app(row_menu);
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            let id = wparam.0 & 0xFFFF;
            with_app(|app| {
                let Some((exe, path)) = selected_entry(app) else {
                    return;
                };
                match id {
                    ID_BLOCK => app.cfg.block(&exe),
                    ID_ALLOW => app.cfg.allow(&exe),
                    ID_FORGET => app.cfg.forget(&exe),
                    ID_REVEAL => reveal_in_explorer(&path),
                    _ => {}
                }
            });
            LRESULT(0)
        }
        WM_CLOSE => {
            unsafe {
                let _ = ShowWindow(hwnd, windows::Win32::UI::WindowsAndMessaging::SW_HIDE);
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

fn reveal_in_explorer(path: &str) {
    if path.is_empty() {
        return;
    }
    let _ = std::process::Command::new("explorer.exe")
        .arg(format!("/select,{path}"))
        .spawn();
}
