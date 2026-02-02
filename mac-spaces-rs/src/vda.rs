//! VirtualDesktopAccessor.dll FFI 封装
//! 
//! 封装对 VirtualDesktopAccessor.dll 的调用，提供类型安全的 Rust 接口

use libloading::{Library, Symbol};
use std::path::Path;
use thiserror::Error;
use windows::Win32::Foundation::HWND;

#[derive(Error, Debug)]
pub enum VdaError {
    #[error("DLL 文件未找到: {0}")]
    DllNotFound(String),
    
    #[error("无法加载 DLL: {0}")]
    LoadError(#[from] libloading::Error),
    
    #[error("函数调用失败: {0}")]
    CallError(String),
}

/// VirtualDesktopAccessor DLL 封装
pub struct VirtualDesktopAccessor {
    lib: Library,
}

impl VirtualDesktopAccessor {
    /// 加载 DLL
    pub fn new<P: AsRef<Path>>(dll_path: P) -> Result<Self, VdaError> {
        let path = dll_path.as_ref();
        
        if !path.exists() {
            return Err(VdaError::DllNotFound(path.display().to_string()));
        }
        
        let lib = unsafe { Library::new(path)? };
        
        Ok(Self { lib })
    }
    
    /// 获取桌面总数
    pub fn get_desktop_count(&self) -> i32 {
        unsafe {
            let func: Symbol<unsafe extern "C" fn() -> i32> = 
                self.lib.get(b"GetDesktopCount").expect("GetDesktopCount not found");
            func()
        }
    }
    
    /// 获取当前桌面索引（0-based）
    pub fn get_current_desktop(&self) -> i32 {
        unsafe {
            let func: Symbol<unsafe extern "C" fn() -> i32> = 
                self.lib.get(b"GetCurrentDesktopNumber").expect("GetCurrentDesktopNumber not found");
            func()
        }
    }
    
    /// 切换到指定桌面（0-based）
    /// 注意：这个方法实际上是有动画的，因为它内部调用的是 IVirtualDesktopManagerInternal::SwitchDesktop
    pub fn go_to_desktop(&self, index: i32) {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(i32)> = 
                self.lib.get(b"GoToDesktopNumber").expect("GoToDesktopNumber not found");
            func(index)
        }
    }
    
    /// 创建新桌面
    pub fn create_desktop(&self) {
        unsafe {
            let func: Symbol<unsafe extern "C" fn()> = 
                self.lib.get(b"CreateDesktop").expect("CreateDesktop not found");
            func()
        }
    }
    
    /// 删除指定桌面
    /// 
    /// # Arguments
    /// * `index` - 要删除的桌面索引
    /// * `fallback` - 窗口移动到的目标桌面索引
    pub fn remove_desktop(&self, index: i32, fallback: i32) {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(i32, i32)> = 
                self.lib.get(b"RemoveDesktop").expect("RemoveDesktop not found");
            func(index, fallback)
        }
    }
    
    /// 将窗口移动到指定桌面
    pub fn move_window_to_desktop(&self, hwnd: HWND, index: i32) {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(isize, i32)> = 
                self.lib.get(b"MoveWindowToDesktopNumber").expect("MoveWindowToDesktopNumber not found");
            func(hwnd.0 as isize, index)
        }
    }
    
    /// 获取窗口所在的桌面索引
    pub fn get_window_desktop(&self, hwnd: HWND) -> i32 {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(isize) -> i32> = 
                self.lib.get(b"GetWindowDesktopNumber").expect("GetWindowDesktopNumber not found");
            func(hwnd.0 as isize)
        }
    }
    
    /// 检查窗口是否在当前桌面
    pub fn is_window_on_current_desktop(&self, hwnd: HWND) -> bool {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(isize) -> i32> = 
                self.lib.get(b"IsWindowOnCurrentVirtualDesktop").expect("IsWindowOnCurrentVirtualDesktop not found");
            func(hwnd.0 as isize) != 0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[ignore] // 需要 DLL 文件才能运行
    fn test_load_dll() {
        let vda = VirtualDesktopAccessor::new("../assets/VirtualDesktopAccessor.dll");
        assert!(vda.is_ok());
    }
}
