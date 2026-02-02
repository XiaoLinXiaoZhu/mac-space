//! 桌面操作模块
//! 
//! 封装虚拟桌面的高级操作

use crate::registry::SpaceRegistry;
use crate::vda::VirtualDesktopAccessor;
use crate::window::WindowHelper;
use std::thread;
use std::time::Duration;
use tracing::{debug, info, warn};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_F11};

/// 切换延迟（等待动画完成）
const SWITCH_DELAY_MS: u64 = 150;

/// 切换到左边的桌面
pub fn switch_left(vda: &VirtualDesktopAccessor) -> bool {
    let current = vda.get_current_desktop();
    
    if current > 0 {
        debug!("切换到左边桌面: {} -> {}", current, current - 1);
        vda.go_to_desktop(current - 1);
        true
    } else {
        debug!("已经是第一个桌面");
        false
    }
}

/// 切换到右边的桌面
pub fn switch_right(vda: &VirtualDesktopAccessor) -> bool {
    let current = vda.get_current_desktop();
    let count = vda.get_desktop_count();
    
    if current < count - 1 {
        debug!("切换到右边桌面: {} -> {}", current, current + 1);
        vda.go_to_desktop(current + 1);
        true
    } else {
        debug!("已经是最后一个桌面");
        false
    }
}

/// 切换全屏空间
pub fn toggle_fullscreen(vda: &VirtualDesktopAccessor, registry: &mut SpaceRegistry) {
    let hwnd = WindowHelper::get_active();
    
    if !WindowHelper::is_valid(hwnd) {
        warn!("无效的窗口");
        return;
    }
    
    if registry.is_fullscreen_space(hwnd) {
        exit_fullscreen_space(vda, registry, hwnd);
    } else {
        enter_fullscreen_space(vda, registry, hwnd);
    }
}

/// 进入全屏空间
fn enter_fullscreen_space(vda: &VirtualDesktopAccessor, registry: &mut SpaceRegistry, hwnd: HWND) {
    info!("进入全屏空间: hwnd={:?}", hwnd);
    
    // 1. 记录原始桌面
    let original_desktop = vda.get_current_desktop();
    
    // 2. 如果窗口是最大化的，先还原
    let was_maximized = WindowHelper::is_maximized(hwnd);
    if was_maximized {
        WindowHelper::restore(hwnd);
        thread::sleep(Duration::from_millis(50));
    }
    
    // 3. 创建新桌面
    let count_before = vda.get_desktop_count();
    vda.create_desktop();
    thread::sleep(Duration::from_millis(50));
    
    let new_desktop = count_before; // 新桌面在最后
    
    // 4. 移动窗口到新桌面
    vda.move_window_to_desktop(hwnd, new_desktop);
    thread::sleep(Duration::from_millis(50));
    
    // 5. 切换到新桌面
    vda.go_to_desktop(new_desktop);
    thread::sleep(Duration::from_millis(SWITCH_DELAY_MS));
    
    // 6. 最大化窗口
    WindowHelper::maximize(hwnd);
    
    // 7. 发送 F11 进入应用全屏模式
    thread::sleep(Duration::from_millis(100));
    send_f11();
    
    // 8. 注册到空间注册表
    registry.register(hwnd, original_desktop, new_desktop);
    
    info!("进入全屏空间完成: 桌面 #{}", new_desktop + 1);
}

/// 退出全屏空间
fn exit_fullscreen_space(vda: &VirtualDesktopAccessor, registry: &mut SpaceRegistry, hwnd: HWND) {
    let info = match registry.get(hwnd) {
        Some(info) => info.clone(),
        None => {
            warn!("窗口不在注册表中");
            return;
        }
    };
    
    info!("退出全屏空间: hwnd={:?}", hwnd);
    
    let original_desktop = info.original_desktop;
    let created_desktop = info.created_desktop;
    
    // 1. 发送 F11 退出应用全屏模式
    send_f11();
    thread::sleep(Duration::from_millis(100));
    
    // 2. 还原窗口
    WindowHelper::restore(hwnd);
    thread::sleep(Duration::from_millis(50));
    
    // 3. 移动窗口回原桌面
    vda.move_window_to_desktop(hwnd, original_desktop);
    thread::sleep(Duration::from_millis(50));
    
    // 4. 切换到原桌面
    vda.go_to_desktop(original_desktop);
    thread::sleep(Duration::from_millis(SWITCH_DELAY_MS));
    
    // 5. 删除空桌面
    if vda.get_desktop_count() > 1 {
        let fallback = if created_desktop > 0 { created_desktop - 1 } else { 0 };
        vda.remove_desktop(created_desktop, fallback);
        
        // 更新其他空间的桌面索引
        registry.update_indices_after_delete(created_desktop);
    }
    
    // 6. 从注册表移除
    registry.remove(hwnd);
    
    info!("退出全屏空间完成: 返回桌面 #{}", original_desktop + 1);
}

/// 处理窗口关闭事件
pub fn handle_window_closed(vda: &VirtualDesktopAccessor, registry: &mut SpaceRegistry, hwnd: HWND) {
    // 检查窗口是否在注册表中
    if !registry.contains(hwnd) {
        return;
    }
    
    let info = match registry.get(hwnd) {
        Some(info) => info.clone(),
        None => return,
    };
    
    info!("检测到全屏空间窗口关闭: hwnd={:?}", hwnd);
    
    let created_desktop = info.created_desktop;
    let current_desktop = vda.get_current_desktop();
    
    // 如果当前在即将删除的桌面上，先切换走
    if current_desktop == created_desktop {
        let target = if created_desktop > 0 { created_desktop - 1 } else { 0 };
        vda.go_to_desktop(target);
        thread::sleep(Duration::from_millis(SWITCH_DELAY_MS));
    }
    
    // 删除空桌面
    if vda.get_desktop_count() > 1 {
        let fallback = if created_desktop > 0 { created_desktop - 1 } else { 0 };
        vda.remove_desktop(created_desktop, fallback);
        
        // 更新其他空间的桌面索引
        registry.update_indices_after_delete(created_desktop);
    }
    
    // 从注册表移除
    registry.remove(hwnd);
    
    info!("全屏空间窗口关闭处理完成");
}

/// 发送 F11 按键
fn send_f11() {
    unsafe {
        let mut inputs = [
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_F11,
                        wScan: 0,
                        dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0),
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_F11,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
        ];
        
        SendInput(&mut inputs, std::mem::size_of::<INPUT>() as i32);
    }
}
