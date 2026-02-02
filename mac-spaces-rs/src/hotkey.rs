//! 全局快捷键管理
//! 
//! 使用低级键盘钩子 (WH_KEYBOARD_LL) 实现全局快捷键监听

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_LEFT, VK_RIGHT, VK_LWIN, VK_RWIN,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx,
    HHOOK, KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, WM_KEYDOWN,
};
use tracing::{debug, trace};

/// 快捷键事件
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyEvent {
    /// Win+Left: 切换到左边的桌面
    SwitchLeft,
    /// Win+Right: 切换到右边的桌面
    SwitchRight,
    /// Win+F: 切换全屏空间
    ToggleFullscreen,
}

// 全局状态（用于钩子回调）
static mut HOTKEY_TX: Option<Sender<HotkeyEvent>> = None;
static HOOK_ACTIVE: AtomicBool = AtomicBool::new(false);

/// 快捷键管理器
pub struct HotkeyManager {
    hook: HHOOK,
}

impl HotkeyManager {
    /// 创建快捷键管理器并安装钩子
    pub fn new(tx: Sender<HotkeyEvent>) -> windows::core::Result<Self> {
        // 保存发送器到全局状态
        unsafe {
            HOTKEY_TX = Some(tx);
        }
        HOOK_ACTIVE.store(true, Ordering::SeqCst);
        
        // 安装低级键盘钩子
        let hook = unsafe {
            SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0)?
        };
        
        debug!("快捷键钩子已安装");
        
        Ok(Self { hook })
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        HOOK_ACTIVE.store(false, Ordering::SeqCst);
        unsafe {
            let _ = UnhookWindowsHookEx(self.hook);
            HOTKEY_TX = None;
        }
        debug!("快捷键钩子已卸载");
    }
}

/// 键盘钩子回调函数
unsafe extern "system" fn keyboard_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code >= 0 && HOOK_ACTIVE.load(Ordering::SeqCst) {
        let kbd = *(lparam.0 as *const KBDLLHOOKSTRUCT);
        
        // 只处理按键按下事件
        if wparam.0 == WM_KEYDOWN as usize {
            // 检测 Win 键是否按下
            let win_pressed = GetAsyncKeyState(VK_LWIN.0 as i32) < 0
                || GetAsyncKeyState(VK_RWIN.0 as i32) < 0;
            
            if win_pressed {
                let event = match kbd.vkCode as u16 {
                    x if x == VK_LEFT.0 => Some(HotkeyEvent::SwitchLeft),
                    x if x == VK_RIGHT.0 => Some(HotkeyEvent::SwitchRight),
                    0x46 => Some(HotkeyEvent::ToggleFullscreen), // 'F' key
                    _ => None,
                };
                
                if let Some(event) = event {
                    trace!("检测到快捷键: {:?}", event);
                    
                    if let Some(ref tx) = HOTKEY_TX {
                        let _ = tx.send(event);
                    }
                    
                    // 阻止事件传递给系统（避免触发 Windows Snap）
                    // 对于 Win+Left/Right，我们需要拦截
                    if matches!(event, HotkeyEvent::SwitchLeft | HotkeyEvent::SwitchRight) {
                        return LRESULT(1);
                    }
                }
            }
        }
    }
    
    CallNextHookEx(None, code, wparam, lparam)
}
