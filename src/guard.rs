use crate::app::{App, with_app};
use crate::input::ms_since_any_input;
use crate::journal::{Entry, Reason, Verdict};
use crate::policy::{Situation, decide};
use crate::window::WindowInfo;
use chrono::Local;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::Input::KeyboardAndMouse::SetActiveWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, EVENT_SYSTEM_FOREGROUND, FLASHW_TIMERNOFG, FLASHW_TRAY, FLASHWINFO,
    FlashWindowEx, GetForegroundWindow, GetWindowThreadProcessId, OBJID_WINDOW,
    SetForegroundWindow, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
};

pub fn install_hook() -> HWINEVENTHOOK {
    unsafe {
        SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(on_win_event),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        )
    }
}

pub fn remove_hook(hook: HWINEVENTHOOK) {
    if !hook.is_invalid() {
        unsafe {
            let _ = UnhookWinEvent(hook);
        }
    }
}

unsafe extern "system" fn on_win_event(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    id_child: i32,
    _thread: u32,
    _time: u32,
) {
    if event != EVENT_SYSTEM_FOREGROUND || id_object != OBJID_WINDOW.0 || id_child != 0 {
        return;
    }
    with_app(|app| app.on_foreground(hwnd));
}

impl App {
    fn on_foreground(&mut self, hwnd: HWND) {
        if self.expecting.take() == Some(hwnd) {
            return;
        }
        let Some(info) = WindowInfo::capture(hwnd) else {
            return;
        };
        if self.is_paused() {
            self.previous = Some(info);
            return;
        }
        // The same application still opening its windows. We already answered it.
        if self.settling(info.pid) {
            return;
        }

        let previous_alive = self
            .previous
            .as_ref()
            .is_some_and(|prev| prev.can_take_focus() && prev.hwnd != hwnd);
        let same_process = self
            .previous
            .as_ref()
            .is_some_and(|prev| prev.pid == info.pid);

        self.input.poll();
        let strikes = self.strikes_for(&info.exe);

        let situation = Situation {
            exe: &info.exe,
            is_noise: info.is_noise(),
            same_process_as_previous: same_process,
            has_previous_window: previous_alive,
            ms_since_click: self.input.ms_since_click(),
            button_down: self.input.button_down_now(),
            keyboard_switch: self.input.switching_by_keyboard(),
            ms_since_input: ms_since_any_input(),
            restores_used: strikes,
        };
        let call = decide(&self.cfg, &situation);

        if call.verdict == Verdict::Allowed {
            if self.cfg.record_everything {
                self.journal.record(Entry {
                    at: Local::now(),
                    exe: info.exe.clone(),
                    path: info.path.clone(),
                    title: info.title.clone(),
                    verdict: call.verdict,
                    reason: call.reason,
                });
                crate::viewer::refresh(self);
            }
            self.previous = Some(info);
            return;
        }

        if call.verdict == Verdict::Restored {
            let target = self.previous.as_ref().map(|prev| prev.hwnd);
            let handed_back = match target {
                Some(target) => self.hand_focus_back(target, hwnd),
                None => false,
            };
            if handed_back {
                self.add_strike(&info.exe);
                self.last_restore = Some((info.pid, std::time::Instant::now()));
            } else {
                // Windows refused. Recording it is still worth something.
                self.previous = Some(info.clone());
            }
            self.journal.record(Entry {
                at: Local::now(),
                exe: info.exe.clone(),
                path: info.path.clone(),
                title: info.title.clone(),
                verdict: if handed_back {
                    Verdict::Restored
                } else {
                    Verdict::Observed
                },
                reason: if handed_back {
                    call.reason
                } else {
                    Reason::Failed
                },
            });
        } else {
            self.journal.record(Entry {
                at: Local::now(),
                exe: info.exe.clone(),
                path: info.path.clone(),
                title: info.title.clone(),
                verdict: call.verdict,
                reason: call.reason,
            });
            self.previous = Some(info);
        }

        crate::tray::refresh_tooltip(self);
        crate::viewer::refresh(self);
    }

    fn hand_focus_back(&mut self, target: HWND, thief: HWND) -> bool {
        self.expecting = Some(target);
        let ok = unsafe { SetForegroundWindow(target).as_bool() } || force_foreground(target);
        if !ok {
            self.expecting = None;
            return false;
        }
        if self.cfg.flash_thief {
            flash_in_taskbar(thief);
        }
        true
    }
}

/// SetForegroundWindow fails by design unless the calling thread owns the foreground.
/// Borrowing the current foreground thread's input queue is the documented way around
/// it, and the same trick every window manager on Windows ends up using.
pub fn force_foreground(target: HWND) -> bool {
    unsafe {
        let foreground = GetForegroundWindow();
        let foreign = GetWindowThreadProcessId(foreground, None);
        let ours = GetCurrentThreadId();
        let attached =
            foreign != 0 && foreign != ours && AttachThreadInput(ours, foreign, true).as_bool();
        let _ = BringWindowToTop(target);
        let ok = SetForegroundWindow(target).as_bool();
        let _ = SetActiveWindow(target);
        if attached {
            let _ = AttachThreadInput(ours, foreign, false);
        }
        ok
    }
}

/// The behaviour Windows itself falls back to: leave the window where it is and
/// blink it in the taskbar, so nothing is lost, it just does not interrupt.
fn flash_in_taskbar(hwnd: HWND) {
    let info = FLASHWINFO {
        cbSize: size_of::<FLASHWINFO>() as u32,
        hwnd,
        dwFlags: FLASHW_TRAY | FLASHW_TIMERNOFG,
        uCount: 3,
        dwTimeout: 0,
    };
    unsafe {
        let _ = FlashWindowEx(&info);
    }
}
