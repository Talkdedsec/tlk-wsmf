//! Input state, read by polling. There is deliberately no keyboard hook here:
//! a WH_KEYBOARD_LL hook is what a keylogger installs, and it gets tools like this
//! one flagged by antivirus engines. We only ever ask "was a mouse button pressed"
//! and "how long since any input", never which key.

use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetLastInputInfo, LASTINPUTINFO, VK_LBUTTON, VK_LWIN, VK_MBUTTON, VK_MENU,
    VK_RBUTTON, VK_RWIN, VK_TAB,
};

const PRESSED_SINCE_LAST_CALL: i16 = 0x0001;
const DOWN_NOW: u16 = 0x8000;

pub struct InputWatch {
    last_click_tick: Option<u32>,
}

impl Default for InputWatch {
    fn default() -> Self {
        Self::new()
    }
}

impl InputWatch {
    pub fn new() -> Self {
        // Drain the "pressed since last call" bits so a click from before we started
        // is not mistaken for a fresh one.
        for key in [VK_LBUTTON, VK_RBUTTON, VK_MBUTTON] {
            unsafe { GetAsyncKeyState(key.0 as i32) };
        }
        Self {
            last_click_tick: None,
        }
    }

    /// Called from the message loop timer. Cheap: three reads.
    pub fn poll(&mut self) {
        let clicked = [VK_LBUTTON, VK_RBUTTON, VK_MBUTTON]
            .iter()
            .any(|key| unsafe { GetAsyncKeyState(key.0 as i32) } & PRESSED_SINCE_LAST_CALL != 0);
        if clicked {
            self.last_click_tick = Some(unsafe { GetTickCount() });
        }
    }

    pub fn ms_since_click(&self) -> Option<u32> {
        self.last_click_tick
            .map(|tick| unsafe { GetTickCount() }.wrapping_sub(tick))
    }

    /// A button held down right now counts as a click in progress: drag, long press,
    /// a menu being held open. The poll may not have caught it yet.
    pub fn button_down_now(&self) -> bool {
        [VK_LBUTTON, VK_RBUTTON, VK_MBUTTON]
            .iter()
            .any(|key| unsafe { GetAsyncKeyState(key.0 as i32) } as u16 & DOWN_NOW != 0)
    }

    /// Alt+Tab or the Windows key: the user is switching windows on purpose.
    pub fn switching_by_keyboard(&self) -> bool {
        let down = |key: u16| unsafe { GetAsyncKeyState(key as i32) } as u16 & DOWN_NOW != 0;
        (down(VK_MENU.0) && down(VK_TAB.0)) || down(VK_LWIN.0) || down(VK_RWIN.0)
    }
}

/// Milliseconds since the last input of any kind, system wide.
pub fn ms_since_any_input() -> u32 {
    let mut info = LASTINPUTINFO {
        cbSize: size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    unsafe {
        if GetLastInputInfo(&mut info).as_bool() {
            GetTickCount().wrapping_sub(info.dwTime)
        } else {
            u32::MAX
        }
    }
}
