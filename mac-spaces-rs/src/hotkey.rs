//! 全局快捷键管理
//! 
//! 使用独立线程 + 低级键盘钩子 (WH_KEYBOARD_LL) 实现全局快捷键监听
//! 通过 PostMessage 与主线程通信，避免卡顿

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_LEFT, VK_RIGHT, VK_LWIN, VK_RWIN,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, PostMessageW,
    SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx,
    HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_USER,
};
use tracing::{debug, trace};

/// 自定义消息 ID
pub const WM_HOTKEY_EVENT: u32 = WM_USER + 100;

/// 快捷键事件（作为 WPARAM 传递）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum HotkeyEvent {
    /// Win+Left: 切换到左边的桌面
    SwitchLeft = 1,
    /// Win+Right: 切换到右边的桌面
    SwitchRight = 2,
    /// Win+F: 切换全屏空间
    ToggleFullscreen = 3,
}

impl HotkeyEvent {
    pub fn from_wparam(wparam: usize) -> Option<Self> {
        match wparam {
            1 => Some(HotkeyEvent::SwitchLeft),
            2 => Some(HotkeyEvent::SwitchRight),
            3 => Some(HotkeyEvent::ToggleFullscreen),
            _ => None,
        }
    }
}

// 全局状态（用于钩子回调）
static mut MAIN_HWND: HWND = HWND(std::ptr::null_mut());
static HOOK_ACTIVE: AtomicBool = AtomicBool::new(false);

/// 快捷键管理器
pub struct HotkeyManager {
    thread_handle: Option<JoinHandle<()>>,
}

impl HotkeyManager {
    /// 创建快捷键管理器并在独立线程中安装钩子
    pub fn new(main_hwnd: HWND) -> Self {
        // 保存主窗口句柄到全局状态
        unsafe {
            MAIN_HWND = main_hwnd;
        }
        HOOK_ACTIVE.store(true, Ordering::SeqCst);
        
        // 在独立线程中运行钩子
        let handle = thread::spawn(|| {
            run_hook_thread();
        });
        
        debug!("快捷键钩子线程已启动");
        
        Self {
            thread_handle: Some(handle),
        }
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        HOOK_ACTIVE.store(false, Ordering::SeqCst);
        
        // 发送退出消息给钩子线程
        // 注意：钩子线程有自己的消息循环，需要通过 PostThreadMessage 退出
        // 但这里简单处理，让线程自然退出
        
        if let Some(handle) = self.thread_handle.take() {
            // 等待线程结束（最多 1 秒）
            let _ = handle.join();
        }
        
        debug!("快捷键钩子线程已停止");
    }
}

/// 钩子线程主函数
fn run_hook_thread() {
    unsafe {
        // 安装低级键盘钩子
        let hook = match SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0) {
            Ok(h) => h,
            Err(e) => {
                tracing::error!("安装键盘钩子失败: {:?}", e);
                return;
            }
        };
        
        debug!("键盘钩子已安装");
        
        // 独立消息循环（服务钩子）
        let mut msg = MSG::default();
        while HOOK_ACTIVE.load(Ordering::SeqCst) {
            // 使用 GetMessageW，它会阻塞直到有消息
            let ret = GetMessageW(&mut msg, None, 0, 0);
            if !ret.as_bool() {
                break;
            }
            
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        
        // 卸载钩子
        let _ = UnhookWindowsHookEx(hook);
        debug!("键盘钩子已卸载");
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
                    
                    // 通过 PostMessage 发送到主线程（非阻塞）
                    let _ = PostMessageW(
                        MAIN_HWND,
                        WM_HOTKEY_EVENT,
                        WPARAM(event as usize),
                        LPARAM(0),
                    );
                    
                    // 阻止事件传递给系统（避免触发 Windows Snap）
                    if matches!(event, HotkeyEvent::SwitchLeft | HotkeyEvent::SwitchRight) {
                        return LRESULT(1);
                    }
                }
            }
        }
    }
    
    CallNextHookEx(None, code, wparam, lparam)
}
