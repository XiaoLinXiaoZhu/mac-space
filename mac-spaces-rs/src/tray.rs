//! 托盘图标模块

use muda::{Menu, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder};
use tracing::debug;

/// 托盘菜单事件
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    ShowInfo,
    ShowRegistry,
    ToggleDebug,
    Reload,
    Exit,
}

/// 托盘图标管理器
pub struct TrayManager {
    _tray: TrayIcon,
    pub menu_show_info: MenuItem,
    pub menu_show_registry: MenuItem,
    pub menu_toggle_debug: MenuItem,
    pub menu_reload: MenuItem,
    pub menu_exit: MenuItem,
}

impl TrayManager {
    /// 创建托盘图标
    pub fn new() -> anyhow::Result<Self> {
        // 创建菜单
        let menu = Menu::new();
        
        // 标题（禁用）
        let title = MenuItem::new("MacSpaces v0.3.0", false, None);
        menu.append(&title)?;
        
        menu.append(&PredefinedMenuItem::separator())?;
        
        // 功能菜单项
        let menu_show_info = MenuItem::new("桌面信息", true, None);
        let menu_show_registry = MenuItem::new("空间注册表", true, None);
        
        menu.append(&menu_show_info)?;
        menu.append(&menu_show_registry)?;
        
        menu.append(&PredefinedMenuItem::separator())?;
        
        let menu_toggle_debug = MenuItem::new("调试模式", true, None);
        menu.append(&menu_toggle_debug)?;
        
        menu.append(&PredefinedMenuItem::separator())?;
        
        let menu_reload = MenuItem::new("重新加载", true, None);
        let menu_exit = MenuItem::new("退出", true, None);
        
        menu.append(&menu_reload)?;
        menu.append(&menu_exit)?;
        
        // 创建托盘图标
        let tray = TrayIconBuilder::new()
            .with_tooltip("MacSpaces - 虚拟桌面空间化")
            .with_menu(Box::new(menu))
            .build()?;
        
        debug!("托盘图标已创建");
        
        Ok(Self {
            _tray: tray,
            menu_show_info,
            menu_show_registry,
            menu_toggle_debug,
            menu_reload,
            menu_exit,
        })
    }
}
