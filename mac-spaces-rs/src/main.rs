//! MacSpaces - Windows 虚拟桌面空间化体验工具
//!
//! 功能：
//!   Win+←     切换到左边的桌面
//!   Win+→     切换到右边的桌面
//!   Win+F     切换全屏空间（进入/退出）
//!
//! 版本：0.3.0 (Rust 重写版)

#![windows_subsystem = "windows"]

mod desktop;
mod hooks;
mod hotkey;
mod registry;
mod tray;
mod vda;
mod window;

use anyhow::Result;
use hotkey::HotkeyEvent;
use hooks::WindowEvent;
use muda::MenuEvent;
use registry::SpaceRegistry;
use single_instance::SingleInstance;
use std::env;
use std::path::PathBuf;
use std::sync::mpsc;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;
use vda::VirtualDesktopAccessor;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, TranslateMessage, MSG,
};

/// 应用事件
enum AppEvent {
    Hotkey(HotkeyEvent),
    Window(WindowEvent),
}

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
    
    // 创建快捷键事件通道
    let (hotkey_tx, hotkey_rx) = mpsc::channel::<HotkeyEvent>();
    let _hotkey_manager = hotkey::HotkeyManager::new(hotkey_tx)?;
    
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
            // 非阻塞检查消息
            let has_msg = GetMessageW(&mut msg, None, 0, 0).as_bool();
            
            if !has_msg {
                break;
            }
            
            // 处理快捷键事件
            while let Ok(event) = hotkey_rx.try_recv() {
                match event {
                    HotkeyEvent::SwitchLeft => {
                        desktop::switch_left(&vda);
                    }
                    HotkeyEvent::SwitchRight => {
                        desktop::switch_right(&vda);
                    }
                    HotkeyEvent::ToggleFullscreen => {
                        desktop::toggle_fullscreen(&vda, &mut registry);
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
            
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    
    info!("MacSpaces 退出");
    Ok(())
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
    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_OK};
    
    let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let msg_wide: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
    
    unsafe {
        MessageBoxW(
            None,
            PCWSTR(msg_wide.as_ptr()),
            PCWSTR(title_wide.as_ptr()),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}
