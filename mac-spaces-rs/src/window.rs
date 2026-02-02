//! 窗口操作辅助模块

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowLongW, IsWindow, ShowWindow,
    GWL_STYLE, SW_MAXIMIZE, SW_RESTORE, WS_MAXIMIZE,
};
use tracing::trace;

/// 窗口辅助函数
pub struct WindowHelper;

impl WindowHelper {
    /// 获取当前活动窗口
    pub fn get_active() -> HWND {
        unsafe { GetForegroundWindow() }
    }
    
    /// 检查窗口是否有效
    pub fn is_valid(hwnd: HWND) -> bool {
        if hwnd.0.is_null() {
            return false;
        }
        
        unsafe { IsWindow(hwnd).as_bool() }
    }
    
    /// 检查窗口是否最大化
    pub fn is_maximized(hwnd: HWND) -> bool {
        if !Self::is_valid(hwnd) {
            return false;
        }
        
        unsafe {
            let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
            (style & WS_MAXIMIZE.0) != 0
        }
    }
    
    /// 最大化窗口
    pub fn maximize(hwnd: HWND) {
        if Self::is_valid(hwnd) {
            trace!("最大化窗口: {:?}", hwnd);
            unsafe {
                let _ = ShowWindow(hwnd, SW_MAXIMIZE);
            }
        }
    }
    
    /// 还原窗口
    pub fn restore(hwnd: HWND) {
        if Self::is_valid(hwnd) {
            trace!("还原窗口: {:?}", hwnd);
            unsafe {
                let _ = ShowWindow(hwnd, SW_RESTORE);
            }
        }
    }
}
