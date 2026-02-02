//! 窗口事件监听
//! 
//! 使用 SetWinEventHook 监听窗口事件，替代轮询方式

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::{
    SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EVENT_OBJECT_DESTROY, OBJID_WINDOW, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
};
use tracing::{debug, trace};

/// 窗口事件
#[derive(Debug, Clone, Copy)]
pub enum WindowEvent {
    /// 窗口被销毁
    Destroyed(HWND),
}

// 全局状态（用于钩子回调）
static mut WINDOW_TX: Option<Sender<WindowEvent>> = None;
static HOOK_ACTIVE: AtomicBool = AtomicBool::new(false);

/// 窗口事件监听器
pub struct WindowEventHook {
    hook: HWINEVENTHOOK,
}

impl WindowEventHook {
    /// 创建窗口事件监听器
    pub fn new(tx: Sender<WindowEvent>) -> windows::core::Result<Self> {
        // 保存发送器到全局状态
        unsafe {
            WINDOW_TX = Some(tx);
        }
        HOOK_ACTIVE.store(true, Ordering::SeqCst);
        
        // 设置 WinEvent 钩子监听窗口销毁事件
        let hook = unsafe {
            SetWinEventHook(
                EVENT_OBJECT_DESTROY,
                EVENT_OBJECT_DESTROY,
                None,
                Some(win_event_proc),
                0,  // 所有进程
                0,  // 所有线程
                WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
            )
        };
        
        if hook.is_invalid() {
            return Err(windows::core::Error::from_win32());
        }
        
        debug!("窗口事件钩子已安装");
        
        Ok(Self { hook })
    }
}

impl Drop for WindowEventHook {
    fn drop(&mut self) {
        HOOK_ACTIVE.store(false, Ordering::SeqCst);
        unsafe {
            let _ = UnhookWinEvent(self.hook);
            WINDOW_TX = None;
        }
        debug!("窗口事件钩子已卸载");
    }
}

/// WinEvent 回调函数
unsafe extern "system" fn win_event_proc(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    _id_child: i32,
    _event_thread: u32,
    _event_time: u32,
) {
    if !HOOK_ACTIVE.load(Ordering::SeqCst) {
        return;
    }
    
    // 只处理窗口本身的销毁事件（不是子对象）
    if event == EVENT_OBJECT_DESTROY && id_object == OBJID_WINDOW.0 {
        trace!("窗口销毁事件: hwnd={:?}", hwnd);
        
        if let Some(ref tx) = WINDOW_TX {
            let _ = tx.send(WindowEvent::Destroyed(hwnd));
        }
    }
}
