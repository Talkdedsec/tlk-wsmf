//! The quick view: what the tray icon opens on a single click. Deliberately thin —
//! a list of what just happened and the two rules you would want to add from it.
//! Anything longer lived belongs in the panel.

use crate::app::{App, with_app};
use crate::system;
use crate::window::{app_icon, wide};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{DWMWA_USE_IMMERSIVE_DARK_MODE, DwmSetWindowAttribute};
use windows::Win32::Graphics::Gdi::{
    COLOR_WINDOW, CreateFontIndirectW, CreateSolidBrush, HBRUSH, HFONT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::IMAGELIST_CREATION_FLAGS;
use windows::Win32::UI::Controls::ImageList_Create;
use windows::Win32::UI::Controls::{
    ICC_LISTVIEW_CLASSES, INITCOMMONCONTROLSEX, InitCommonControlsEx, LVCF_SUBITEM, LVCF_TEXT,
    LVCF_WIDTH, LVCOLUMNW, LVIF_TEXT, LVITEMW, LVM_DELETEALLITEMS, LVM_GETHEADER, LVM_GETNEXTITEM,
    LVM_INSERTCOLUMNW, LVM_INSERTITEMW, LVM_SETBKCOLOR, LVM_SETCOLUMNWIDTH,
    LVM_SETEXTENDEDLISTVIEWSTYLE, LVM_SETIMAGELIST, LVM_SETITEMTEXTW, LVM_SETTEXTBKCOLOR,
    LVM_SETTEXTCOLOR, LVNI_SELECTED, LVS_EX_DOUBLEBUFFER, LVS_EX_FULLROWSELECT, LVS_REPORT,
    LVS_SHOWSELALWAYS, LVS_SINGLESEL, LVSCW_AUTOSIZE_USEHEADER, LVSIL_SMALL, NM_RCLICK, NMHDR,
    SetWindowTheme,
};
use windows::Win32::UI::HiDpi::GetDpiForSystem;
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CW_USEDEFAULT, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu,
    GetClientRect, GetCursorPos, GetDlgItem, GetSystemMetrics, HMENU, IsWindowVisible,
    MF_SEPARATOR, MF_STRING, MoveWindow, NONCLIENTMETRICSW, RegisterClassW, SM_CXSCREEN,
    SM_CYSCREEN, SPI_GETNONCLIENTMETRICS, SW_HIDE, SW_SHOWNORMAL,
    SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SendMessageW, SetForegroundWindow, ShowWindow,
    SystemParametersInfoW, TPM_RIGHTBUTTON, TrackPopupMenu, WINDOW_STYLE, WM_CLOSE, WM_COMMAND,
    WM_CREATE, WM_NOTIFY, WM_SETFONT, WM_SIZE, WNDCLASSW, WS_CHILD, WS_OVERLAPPEDWINDOW,
    WS_VISIBLE,
};
use windows::core::{PCWSTR, PWSTR};

const CLASS_NAME: &str = "WsmfQuickView";
const ID_LIST: i32 = 200;
const ID_BLOCK: usize = 301;
const ID_ALLOW: usize = 302;
const ID_FORGET: usize = 303;
const ID_REVEAL: usize = 304;

/// Window size at 100% scaling. CreateWindowExW takes physical pixels, so on a
/// scaled display these have to be scaled too or the window opens too small.
const WIDTH: i32 = 940;
const HEIGHT: i32 = 540;
/// Rows at the default height are cramped; this is applied through a spacer image list.
const ROW_HEIGHT: i32 = 22;
const COLUMNS: usize = 5;

const DARK_BACKGROUND: u32 = 0x0020_1B18;
const DARK_TEXT: u32 = 0x00E8_E6E3;
const LIGHT_TEXT: u32 = 0x0020_2020;

pub fn show(app: &mut App) {
    if app.viewer.is_invalid() {
        app.viewer = create_window(app.cfg.strings().app_name);
    }
    if app.viewer.is_invalid() {
        return;
    }
    if list_of(app.viewer).is_invalid() {
        create_list(app.viewer, app.cfg.strings());
        resize_list(app.viewer);
    }
    fill(app);
    unsafe {
        let _ = ShowWindow(app.viewer, SW_SHOWNORMAL);
        let _ = SetForegroundWindow(app.viewer);
    }
}

pub fn refresh(app: &mut App) {
    if !app.viewer.is_invalid() && unsafe { IsWindowVisible(app.viewer).as_bool() } {
        fill(app);
    }
}

fn create_window(title: &str) -> HWND {
    unsafe {
        let controls = INITCOMMONCONTROLSEX {
            dwSize: size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_LISTVIEW_CLASSES,
        };
        let _ = InitCommonControlsEx(&controls);

        let instance = GetModuleHandleW(None).unwrap_or_default();
        let class = wide(CLASS_NAME);
        let dark = system::dark_mode();
        let wc = WNDCLASSW {
            lpfnWndProc: Some(viewer_proc),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(class.as_ptr()),
            hIcon: app_icon(),
            hbrBackground: if dark {
                CreateSolidBrush(COLORREF(DARK_BACKGROUND))
            } else {
                HBRUSH((COLOR_WINDOW.0 + 1) as isize as *mut _)
            },
            ..Default::default()
        };
        RegisterClassW(&wc);

        let dpi = GetDpiForSystem().max(96) as i32;
        let width = WIDTH * dpi / 96;
        let height = HEIGHT * dpi / 96;
        let x = (GetSystemMetrics(SM_CXSCREEN) - width) / 2;
        let y = (GetSystemMetrics(SM_CYSCREEN) - height) / 2;
        let hwnd = CreateWindowExW(
            Default::default(),
            PCWSTR(class.as_ptr()),
            PCWSTR(wide(title).as_ptr()),
            WS_OVERLAPPEDWINDOW,
            if x > 0 { x } else { CW_USEDEFAULT },
            if y > 0 { y } else { CW_USEDEFAULT },
            width,
            height,
            None,
            None,
            Some(instance.into()),
            None,
        )
        .unwrap_or_default();

        if dark && !hwnd.is_invalid() {
            let on: i32 = 1;
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_USE_IMMERSIVE_DARK_MODE,
                &on as *const i32 as *const _,
                size_of::<i32>() as u32,
            );
        }
        hwnd
    }
}

/// Fills the window. The last column is told to use whatever is left: for the final
/// column ListView reads LVSCW_AUTOSIZE_USEHEADER as "take the remaining width".
fn resize_list(hwnd: HWND) {
    let list = list_of(hwnd);
    if list.is_invalid() {
        return;
    }
    unsafe {
        let mut rect = RECT::default();
        let _ = GetClientRect(hwnd, &mut rect);
        let _ = MoveWindow(list, 0, 0, rect.right, rect.bottom, true);
        SendMessageW(
            list,
            LVM_SETCOLUMNWIDTH,
            Some(WPARAM(COLUMNS - 1)),
            Some(LPARAM(LVSCW_AUTOSIZE_USEHEADER as isize)),
        );
    }
}

fn list_of(hwnd: HWND) -> HWND {
    unsafe { GetDlgItem(Some(hwnd), ID_LIST).unwrap_or_default() }
}

fn create_list(parent: HWND, s: &'static crate::i18n::Strings) -> HWND {
    unsafe {
        let instance = GetModuleHandleW(None).unwrap_or_default();
        // No sunken border: it is a 1990s frame that looks wrong in either theme.
        let Ok(list) = CreateWindowExW(
            Default::default(),
            PCWSTR(wide("SysListView32").as_ptr()),
            PCWSTR::null(),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(LVS_REPORT | LVS_SINGLESEL | LVS_SHOWSELALWAYS),
            0,
            0,
            0,
            0,
            Some(parent),
            Some(HMENU(ID_LIST as isize as *mut _)),
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

        // An image list nothing draws into, purely to give the rows some height.
        let spacer = ImageList_Create(1, ROW_HEIGHT, IMAGELIST_CREATION_FLAGS(0), 1, 0);
        if !spacer.is_invalid() {
            SendMessageW(
                list,
                LVM_SETIMAGELIST,
                Some(WPARAM(LVSIL_SMALL as usize)),
                Some(LPARAM(spacer.0 as isize)),
            );
        }

        let dark = system::dark_mode();
        let header =
            HWND(SendMessageW(list, LVM_GETHEADER, Some(WPARAM(0)), Some(LPARAM(0))).0 as *mut _);
        if dark {
            let _ = SetWindowTheme(list, PCWSTR(wide("DarkMode_Explorer").as_ptr()), None);
            if !header.is_invalid() {
                let _ = SetWindowTheme(header, PCWSTR(wide("DarkMode_ItemsView").as_ptr()), None);
            }
            for (message, colour) in [
                (LVM_SETBKCOLOR, DARK_BACKGROUND),
                (LVM_SETTEXTBKCOLOR, DARK_BACKGROUND),
                (LVM_SETTEXTCOLOR, DARK_TEXT),
            ] {
                SendMessageW(
                    list,
                    message,
                    Some(WPARAM(0)),
                    Some(LPARAM(COLORREF(colour).0 as isize)),
                );
            }
        } else {
            let _ = SetWindowTheme(list, PCWSTR(wide("Explorer").as_ptr()), None);
            if !header.is_invalid() {
                let _ = SetWindowTheme(header, PCWSTR(wide("ItemsView").as_ptr()), None);
            }
            SendMessageW(
                list,
                LVM_SETTEXTCOLOR,
                Some(WPARAM(0)),
                Some(LPARAM(COLORREF(LIGHT_TEXT).0 as isize)),
            );
        }

        if let Some(font) = message_font() {
            SendMessageW(
                list,
                WM_SETFONT,
                Some(WPARAM(font.0 as usize)),
                Some(LPARAM(1)),
            );
        }

        for (index, (title, width)) in [
            (s.column_when, 130),
            (s.column_app, 200),
            (s.column_what, 130),
            (s.column_why, 190),
            (s.column_title, 260),
        ]
        .iter()
        .enumerate()
        {
            let text = wide(title);
            let column = LVCOLUMNW {
                mask: LVCF_TEXT | LVCF_WIDTH | LVCF_SUBITEM,
                cx: *width,
                pszText: PWSTR(text.as_ptr() as *mut u16),
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
    let s = app.cfg.strings();
    unsafe {
        SendMessageW(list, LVM_DELETEALLITEMS, None, None);
        for (row, entry) in app.journal.entries().enumerate() {
            let when = wide(&entry.at.format("%d.%m %H:%M:%S").to_string());
            let item = LVITEMW {
                mask: LVIF_TEXT,
                iItem: row as i32,
                iSubItem: 0,
                pszText: PWSTR(when.as_ptr() as *mut u16),
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
                entry.verdict.label(s).to_string(),
                entry.reason.label(s).to_string(),
                entry.title.clone(),
            ];
            for (offset, text) in columns.iter().enumerate() {
                let value = wide(text);
                let sub = LVITEMW {
                    mask: LVIF_TEXT,
                    iItem: row as i32,
                    iSubItem: offset as i32 + 1,
                    pszText: PWSTR(value.as_ptr() as *mut u16),
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
    let s = app.cfg.strings();
    unsafe {
        let Ok(menu) = CreatePopupMenu() else {
            return;
        };
        let label = |id: usize, text: String| {
            let _ = AppendMenuW(menu, MF_STRING, id, PCWSTR(wide(&text).as_ptr()));
        };
        label(ID_BLOCK, format!("{} {exe}", s.menu_block));
        label(ID_ALLOW, format!("{} {exe}", s.menu_allow));
        label(ID_FORGET, format!("{} {exe}", s.menu_forget));
        if !path.is_empty() {
            let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
            label(ID_REVEAL, s.menu_reveal.to_string());
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
        WM_CREATE => LRESULT(0),
        WM_SIZE => {
            resize_list(hwnd);
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
                    ID_BLOCK => {
                        app.cfg.block(&exe);
                        app.save_config();
                    }
                    ID_ALLOW => {
                        app.cfg.allow(&exe);
                        app.save_config();
                    }
                    ID_FORGET => {
                        app.cfg.forget(&exe);
                        app.save_config();
                    }
                    ID_REVEAL => reveal_in_explorer(&path),
                    _ => {}
                }
            });
            LRESULT(0)
        }
        WM_CLOSE => {
            unsafe {
                let _ = ShowWindow(hwnd, SW_HIDE);
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
