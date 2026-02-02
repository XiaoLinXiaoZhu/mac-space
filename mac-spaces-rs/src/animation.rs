//! 切换动画模块
//!
//! 方案 C：渐变遮罩动画（优化版）
//! 使用简单的半透明窗口 + 快速渐变

use std::time::{Duration, Instant};
use std::thread;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, InvalidateRect,
    PAINTSTRUCT, GRADIENT_FILL_RECT_H, GradientFill, TRIVERTEX, GRADIENT_RECT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetSystemMetrics,
    RegisterClassW, SetLayeredWindowAttributes, ShowWindow,
    CS_HREDRAW, CS_VREDRAW, LWA_ALPHA,
    SM_CXSCREEN, SM_CYSCREEN, SW_HIDE,
    WM_DESTROY, WM_PAINT, WNDCLASSW,
    WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_POPUP, SetWindowPos, HWND_TOPMOST, SWP_SHOWWINDOW,
};
use tracing::debug;

/// 动画方向
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Left,
    Right,
}

/// 动画配置
const ANIMATION_DURATION_MS: u64 = 200;  // 动画时长
const MAX_ALPHA: u8 = 220;               // 最大透明度
const FRAME_DURATION_MS: u64 = 16;       // ~60fps

// 全局状态
static mut CURRENT_DIRECTION: Direction = Direction::Right;

/// 动画窗口管理器
pub struct AnimationOverlay {
    hwnd: Option<HWND>,
    screen_width: i32,
    screen_height: i32,
}

impl AnimationOverlay {
    /// 创建动画管理器
    pub fn new() -> Self {
        let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) };
        
        Self {
            hwnd: None,
            screen_width,
            screen_height,
        }
    }
    
    /// 播放切换动画
    pub fn play<F>(&mut self, direction: Direction, switch_fn: F)
    where
        F: FnOnce(),
    {
        debug!("播放切换动画: {:?}", direction);
        
        // 设置当前方向
        unsafe {
            CURRENT_DIRECTION = direction;
        }
        
        // 创建遮罩窗口（快速，不做复杂绘制）
        if let Err(e) = self.create_overlay_window() {
            tracing::warn!("创建动画窗口失败: {:?}, 直接切换", e);
            switch_fn();
            return;
        }
        
        let start = Instant::now();
        let duration = Duration::from_millis(ANIMATION_DURATION_MS);
        let switch_point = Duration::from_millis(ANIMATION_DURATION_MS * 35 / 100); // 35% 时切换
        let mut switched = false;
        let mut switch_fn = Some(switch_fn);
        
        // 动画循环
        while start.elapsed() < duration {
            let progress = start.elapsed().as_secs_f32() / duration.as_secs_f32();
            
            // 更新遮罩
            self.update_overlay(direction, progress);
            
            // 在切换点执行切换
            if !switched && start.elapsed() >= switch_point {
                if let Some(f) = switch_fn.take() {
                    f();
                }
                switched = true;
            }
            
            thread::sleep(Duration::from_millis(FRAME_DURATION_MS));
        }
        
        // 确保切换已执行
        if let Some(f) = switch_fn.take() {
            f();
        }
        
        // 销毁遮罩窗口
        self.destroy_overlay_window();
        
        debug!("切换动画完成");
    }
    
    /// 创建遮罩窗口
    fn create_overlay_window(&mut self) -> windows::core::Result<()> {
        unsafe {
            let instance = GetModuleHandleW(None)?;
            
            let class_name = wide_string("MacSpacesOverlayV2");
            
            let wc = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(overlay_window_proc),
                hInstance: instance.into(),
                lpszClassName: PCWSTR(class_name.as_ptr()),
                ..Default::default()
            };
            
            RegisterClassW(&wc);
            
            // 创建窗口（初始在屏幕外）
            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
                PCWSTR(class_name.as_ptr()),
                PCWSTR::null(),
                WS_POPUP,
                -self.screen_width,
                0,
                self.screen_width,
                self.screen_height,
                None,
                None,
                instance,
                None,
            )?;
            
            // 设置初始透明度为 0
            SetLayeredWindowAttributes(hwnd, None, 0, LWA_ALPHA)?;
            
            self.hwnd = Some(hwnd);
            
            Ok(())
        }
    }
    
    /// 更新遮罩
    fn update_overlay(&self, direction: Direction, progress: f32) {
        let Some(hwnd) = self.hwnd else { return };
        
        let eased = ease_out_cubic(progress);
        
        // 透明度：快速淡入，缓慢淡出
        let alpha = if progress < 0.35 {
            (MAX_ALPHA as f32 * (progress / 0.35)) as u8
        } else {
            (MAX_ALPHA as f32 * (1.0 - (progress - 0.35) / 0.65)) as u8
        };
        
        // 位置计算
        let x = match direction {
            Direction::Left => {
                let start = -self.screen_width;
                let end = self.screen_width;
                start + ((end - start) as f32 * eased) as i32
            }
            Direction::Right => {
                let start = self.screen_width;
                let end = -self.screen_width;
                start + ((end - start) as f32 * eased) as i32
            }
        };
        
        unsafe {
            let _ = SetLayeredWindowAttributes(hwnd, None, alpha, LWA_ALPHA);
            
            let _ = SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                x,
                0,
                self.screen_width,
                self.screen_height,
                SWP_SHOWWINDOW,
            );
            
            let _ = InvalidateRect(hwnd, None, false);
        }
    }
    
    /// 销毁遮罩窗口
    fn destroy_overlay_window(&mut self) {
        if let Some(hwnd) = self.hwnd.take() {
            unsafe {
                let _ = ShowWindow(hwnd, SW_HIDE);
                let _ = DestroyWindow(hwnd);
            }
        }
    }
}

impl Drop for AnimationOverlay {
    fn drop(&mut self) {
        self.destroy_overlay_window();
    }
}

/// 遮罩窗口过程
unsafe extern "system" fn overlay_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            
            let width = GetSystemMetrics(SM_CXSCREEN);
            let height = GetSystemMetrics(SM_CYSCREEN);
            
            // 使用 GradientFill 绘制水平渐变
            let direction = CURRENT_DIRECTION;
            
            let (left_color, right_color) = match direction {
                Direction::Left => (0x0000u16, 0xFF00u16),  // 黑 -> 透明
                Direction::Right => (0xFF00u16, 0x0000u16), // 透明 -> 黑
            };
            
            let mut vertices = [
                TRIVERTEX {
                    x: 0,
                    y: 0,
                    Red: left_color,
                    Green: left_color,
                    Blue: left_color,
                    Alpha: 0,
                },
                TRIVERTEX {
                    x: width,
                    y: height,
                    Red: right_color,
                    Green: right_color,
                    Blue: right_color,
                    Alpha: 0,
                },
            ];
            
            let rect = GRADIENT_RECT {
                UpperLeft: 0,
                LowerRight: 1,
            };
            
            // GradientFill 签名: (hdc, pvertex: &[TRIVERTEX], pmesh: *const c_void, nmesh: u32, ulmode)
            let _ = GradientFill(
                hdc,
                &vertices,
                &rect as *const GRADIENT_RECT as *const std::ffi::c_void,
                1,  // nmesh: 矩形数量
                GRADIENT_FILL_RECT_H,
            );
            
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_DESTROY => LRESULT(0),
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// ease-out-cubic 缓动函数
fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

/// 转换为宽字符串
fn wide_string(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
