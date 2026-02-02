//! MacSpaces - Windows 虚拟桌面空间化体验工具
//!
//! 功能：
//!   Win+←     切换到左边的桌面
//!   Win+→     切换到右边的桌面
//!   Win+F     切换全屏空间（进入/退出）
//!
//! 版本：0.3.0 (Rust 重写版)

#![windows_subsystem = "windows"]

mod animation;
mod desktop;
mod hooks;
mod hotkey;
mod registry;
mod tray;
mod vda;
mod window;

use anyhow::Result;
use animation::{AnimationOverlay, Direction};
use hotkey::{HotkeyEvent, HotkeyManager, WM_HOTKEY_EVENT};
use hooks::WindowEvent;
use muda::MenuEvent;
use registry::SpaceRegistry;
use single_instance::SingleInstance;
use std::env;
use std::path::PathBuf;
use std::sync::mpsc;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;
use vda::VirtualDesktopAccessor;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW,
    RegisterClassW, TranslateMessage, CS_HREDRAW, CS_VREDRAW,
    CW_USEDEFAULT, MSG, WINDOW_EX_STYLE, WM_DESTROY, WNDCLASSW, WS_OVERLAPPED,
};

fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();
    
    info!("MacSpaces v0.3.0 启动中...");
    
    // 单实例检查
    let instance = SingleInstance::new("MacSpaces-Rust-v0.3.0")?;
    if !instance.is_single() {
        warn!("MacSpaces 已经在运行");
        return Ok(());
    }
    
    // 获取 DLL 路径
    let dll_path = get_dll_path()?;
    info!("DLL 路径: {}", dll_path.display());
    
    // 初始化虚拟桌面 API
    let vda = VirtualDesktopAccessor::new(&dll_path)?;
    info!("VirtualDesktopAccessor 加载成功，桌面数量: {}", vda.get_desktop_count());
    
    // 创建空间注册表
    let mut registry = SpaceRegistry::new();
    
    // 创建动画管理器
    let mut animator = AnimationOverlay::new();
    
    // 创建消息窗口（用于接收钩子线程的 PostMessage）
    let main_hwnd = create_message_window()?;
    info!("消息窗口已创建: {:?}", main_hwnd);
    
    // 设置快捷键钩子（在独立线程中运行）
    let _hotkey_manager = HotkeyManager::new(main_hwnd);
    
    // 设置窗口事件钩子
    let (window_tx, window_rx) = mpsc::channel();
    let _window_hook = hooks::WindowEventHook::new(window_tx)?;
    
    // 创建托盘图标
    let tray = tray::TrayManager::new()?;
    
    info!("MacSpaces 初始化完成");
    info!("快捷键: Win+← (左切换), Win+→ (右切换), Win+F (全屏空间)");
    
    // 消息循环
    unsafe {
        let mut msg = MSG::default();
        
        loop {
            let ret = GetMessageW(&mut msg, None, 0, 0);
            
            if !ret.as_bool() {
                break;
            }
            
            // 处理自定义快捷键消息
            if msg.message == WM_HOTKEY_EVENT {
                if let Some(event) = HotkeyEvent::from_wparam(msg.wParam.0) {
                    match event {
                        HotkeyEvent::SwitchLeft => {
                            // 先检查是否可以切换
                            if desktop::can_switch_left(&vda) {
                                animator.play(Direction::Left, || {
                                    desktop::switch_left(&vda);
                                });
                            }
                        }
                        HotkeyEvent::SwitchRight => {
                            // 先检查是否可以切换
                            if desktop::can_switch_right(&vda) {
                                animator.play(Direction::Right, || {
                                    desktop::switch_right(&vda);
                                });
                            }
                        }
                        HotkeyEvent::ToggleFullscreen => {
                            desktop::toggle_fullscreen(&vda, &mut registry);
                        }
                    }
                }
            }
            
            // 处理窗口事件
            while let Ok(event) = window_rx.try_recv() {
                match event {
                    WindowEvent::Destroyed(hwnd) => {
                        desktop::handle_window_closed(&vda, &mut registry, hwnd);
                    }
                }
            }
            
            // 处理托盘菜单事件
            if let Ok(event) = MenuEvent::receiver().try_recv() {
                if event.id == tray.menu_exit.id() {
                    info!("用户请求退出");
                    break;
                } else if event.id == tray.menu_show_info.id() {
                    show_desktop_info(&vda);
                } else if event.id == tray.menu_show_registry.id() {
                    show_registry_info(&registry);
                } else if event.id == tray.menu_reload.id() {
                    info!("重新加载请求（暂不支持）");
                }
            }
            
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    
    info!("MacSpaces 退出");
    Ok(())
}

/// 创建消息窗口（隐藏窗口，仅用于接收消息）
fn create_message_window() -> Result<HWND> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        
        // 注册窗口类
        let class_name = wide_string("MacSpacesMessageWindow");
        
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(message_window_proc),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };
        
        RegisterClassW(&wc);
        
        // 创建隐藏窗口
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR::null(),
            WS_OVERLAPPED,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            0,
            0,
            None,
            None,
            instance,
            None,
        )?;
        
        Ok(hwnd)
    }
}

/// 消息窗口过程
unsafe extern "system" fn message_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => {
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// 转换为宽字符串
fn wide_string(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// 获取 DLL 路径
fn get_dll_path() -> Result<PathBuf> {
    let exe_dir = env::current_exe()?
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    
    // 尝试多个可能的路径
    let candidates = [
        exe_dir.join("assets").join("VirtualDesktopAccessor.dll"),
        exe_dir.join("VirtualDesktopAccessor.dll"),
        PathBuf::from("assets").join("VirtualDesktopAccessor.dll"),
    ];
    
    for path in &candidates {
        if path.exists() {
            return Ok(path.clone());
        }
    }
    
    // 默认返回第一个候选路径（让后续加载报错）
    Ok(candidates[0].clone())
}

/// 显示桌面信息
fn show_desktop_info(vda: &VirtualDesktopAccessor) {
    let count = vda.get_desktop_count();
    let current = vda.get_current_desktop() + 1;
    
    let msg = format!("桌面总数: {}\n当前桌面: #{}", count, current);
    
    show_message_box("MacSpaces 桌面信息", &msg);
}

/// 显示注册表信息
fn show_registry_info(registry: &SpaceRegistry) {
    let msg = registry.debug_info();
    show_message_box("MacSpaces 空间注册表", &msg);
}

/// 显示消息框
fn show_message_box(title: &str, message: &str) {
    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_OK};
    
    let title_wide = wide_string(title);
    let msg_wide = wide_string(message);
    
    unsafe {
        MessageBoxW(
            None,
            PCWSTR(msg_wide.as_ptr()),
            PCWSTR(title_wide.as_ptr()),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}
