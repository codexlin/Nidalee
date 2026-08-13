//! Windows 底层键盘钩子：游戏前台时 RegisterHotKey 收不到 Insert。

use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use once_cell::sync::OnceCell;
use tauri::AppHandle;

use super::state;

const DEBOUNCE_MS: u64 = 280;

static APP: OnceCell<Mutex<Option<AppHandle>>> = OnceCell::new();
static INSTALLED: AtomicBool = AtomicBool::new(false);
static TARGET_VK: AtomicU16 = AtomicU16::new(0x2D);
static NEED_CTRL: AtomicBool = AtomicBool::new(false);
static NEED_ALT: AtomicBool = AtomicBool::new(false);
static NEED_SHIFT: AtomicBool = AtomicBool::new(false);
static KEY_HELD: AtomicBool = AtomicBool::new(false);
static LAST_FIRE_MS: AtomicU32 = AtomicU32::new(0);

pub fn install(app: &AppHandle, display: &str) {
    set_shortcut(display);
    if let Ok(mut slot) = APP.get_or_init(|| Mutex::new(None)).lock() {
        *slot = Some(app.clone());
    }
    #[cfg(windows)]
    windows::ensure_hook();
}

pub fn set_shortcut(display: &str) {
    if let Some(spec) = parse_hotkey(display) {
        TARGET_VK.store(spec.vk, Ordering::Relaxed);
        NEED_CTRL.store(spec.ctrl, Ordering::Relaxed);
        NEED_ALT.store(spec.alt, Ordering::Relaxed);
        NEED_SHIFT.store(spec.shift, Ordering::Relaxed);
        log::info!("[augment-overlay] 底层热键 {display} vk=0x{:X}", spec.vk);
    }
}

struct HotkeySpec {
    vk: u16,
    ctrl: bool,
    alt: bool,
    shift: bool,
}

fn parse_hotkey(display: &str) -> Option<HotkeySpec> {
    let mut ctrl = false;
    let mut alt = false;
    let mut shift = false;
    let mut vk = None;
    for part in display.split('+') {
        match part {
            "Ctrl" => ctrl = true,
            "Alt" => alt = true,
            "Shift" => shift = true,
            "Insert" => vk = Some(0x2D),
            "Home" => vk = Some(0x24),
            "Space" => vk = Some(0x20),
            other if other.starts_with('F') && other[1..].bytes().all(|b| b.is_ascii_digit()) => {
                let n: u16 = other[1..].parse().ok()?;
                if (1..=24).contains(&n) {
                    vk = Some(0x70 + n - 1);
                }
            }
            other if other.len() == 1 => {
                let ch = other.chars().next()?.to_ascii_uppercase();
                if ch.is_ascii_alphanumeric() {
                    vk = Some(ch as u16);
                }
            }
            _ => {}
        }
    }
    Some(HotkeySpec { vk: vk?, ctrl, alt, shift })
}

fn fire_toggle() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u32)
        .unwrap_or(0);
    let last = LAST_FIRE_MS.load(Ordering::Relaxed);
    if now.saturating_sub(last) < DEBOUNCE_MS as u32 {
        return;
    }
    LAST_FIRE_MS.store(now, Ordering::Relaxed);
    let Some(app) = APP.get().and_then(|slot| slot.lock().ok().and_then(|g| g.clone())) else {
        return;
    };
    tauri::async_runtime::spawn(async move {
        state::toggle_side_panel(app).await;
    });
}

#[cfg(windows)]
mod windows {
    use super::*;
    use std::ptr;

    const WH_KEYBOARD_LL: i32 = 13;
    const WM_KEYDOWN: usize = 0x0100;
    const WM_SYSKEYDOWN: usize = 0x0104;
    const WM_KEYUP: usize = 0x0101;
    const WM_SYSKEYUP: usize = 0x0105;
    const VK_SHIFT: i32 = 0x10;
    const VK_CONTROL: i32 = 0x11;
    const VK_MENU: i32 = 0x12;

    #[repr(C)]
    struct KbdLlHook {
        vk_code: u32,
        scan_code: u32,
        flags: u32,
        time: u32,
        extra: usize,
    }

    #[repr(C)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    struct Msg {
        hwnd: isize,
        message: u32,
        w_param: usize,
        l_param: isize,
        time: u32,
        pt: Point,
    }

    extern "system" {
        fn SetWindowsHookExW(
            id: i32,
            func: Option<unsafe extern "system" fn(i32, usize, isize) -> isize>,
            inst: isize,
            thread: u32,
        ) -> isize;
        fn CallNextHookEx(hhk: isize, code: i32, w: usize, l: isize) -> isize;
        fn GetModuleHandleW(name: *const u16) -> isize;
        fn GetMessageW(msg: *mut Msg, hwnd: isize, min: u32, max: u32) -> i32;
        fn TranslateMessage(msg: *const Msg) -> i32;
        fn DispatchMessageW(msg: *const Msg) -> isize;
        fn GetAsyncKeyState(vkey: i32) -> i16;
    }

    fn modifier_down(vkey: i32) -> bool {
        unsafe { GetAsyncKeyState(vkey) as u16 & 0x8000 != 0 }
    }

    fn matches(vk: u32) -> bool {
        if vk as u16 != TARGET_VK.load(Ordering::Relaxed) {
            return false;
        }
        modifier_down(VK_CONTROL) == NEED_CTRL.load(Ordering::Relaxed)
            && modifier_down(VK_MENU) == NEED_ALT.load(Ordering::Relaxed)
            && modifier_down(VK_SHIFT) == NEED_SHIFT.load(Ordering::Relaxed)
    }

    unsafe extern "system" fn hook_proc(code: i32, w_param: usize, l_param: isize) -> isize {
        if code >= 0 && l_param != 0 {
            let info = &*(l_param as *const KbdLlHook);
            let down = w_param == WM_KEYDOWN || w_param == WM_SYSKEYDOWN;
            let up = w_param == WM_KEYUP || w_param == WM_SYSKEYUP;
            if down && matches(info.vk_code) {
                if !KEY_HELD.swap(true, Ordering::SeqCst) {
                    fire_toggle();
                }
            } else if up && info.vk_code as u16 == TARGET_VK.load(Ordering::Relaxed) {
                KEY_HELD.store(false, Ordering::SeqCst);
            }
        }
        CallNextHookEx(0, code, w_param, l_param)
    }

    pub fn ensure_hook() {
        if INSTALLED.swap(true, Ordering::SeqCst) {
            return;
        }
        std::thread::Builder::new()
            .name("augment-hotkey".into())
            .spawn(|| unsafe {
                let inst = GetModuleHandleW(ptr::null());
                let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), inst, 0);
                if hook == 0 {
                    log::error!("[augment-overlay] 安装底层热键失败");
                    INSTALLED.store(false, Ordering::SeqCst);
                    return;
                }
                log::info!("[augment-overlay] 已安装底层热键钩子");
                let mut msg = Msg {
                    hwnd: 0,
                    message: 0,
                    w_param: 0,
                    l_param: 0,
                    time: 0,
                    pt: Point { x: 0, y: 0 },
                };
                while GetMessageW(&mut msg, 0, 0, 0) > 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            })
            .ok();
    }
}

#[cfg(test)]
mod tests {
    use super::parse_hotkey;

    #[test]
    fn parse_insert_and_combos() {
        let insert = parse_hotkey("Insert").expect("Insert");
        assert_eq!(insert.vk, 0x2D);
        assert!(!insert.ctrl);
        let combo = parse_hotkey("Ctrl+Shift+H").expect("combo");
        assert_eq!(combo.vk, b'H' as u16);
        assert!(combo.ctrl && combo.shift && !combo.alt);
        assert_eq!(parse_hotkey("F8").expect("F8").vk, 0x77);
    }
}
