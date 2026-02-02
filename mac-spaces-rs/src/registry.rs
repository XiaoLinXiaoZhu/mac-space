//! 空间注册表
//! 
//! 管理全屏空间的状态，记录窗口与桌面的映射关系

use std::collections::HashMap;
use windows::Win32::Foundation::HWND;
use tracing::debug;

/// 空间信息
#[derive(Debug, Clone)]
pub struct SpaceInfo {
    /// 窗口句柄
    pub hwnd: HWND,
    /// 原始桌面索引（用于退出时返回）
    pub original_desktop: i32,
    /// 创建的桌面索引（用于删除）
    pub created_desktop: i32,
}

/// 空间注册表
pub struct SpaceRegistry {
    /// hwnd.0 -> SpaceInfo
    spaces: HashMap<isize, SpaceInfo>,
}

impl SpaceRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self {
            spaces: HashMap::new(),
        }
    }
    
    /// 注册一个全屏空间
    pub fn register(&mut self, hwnd: HWND, original_desktop: i32, created_desktop: i32) {
        let info = SpaceInfo {
            hwnd,
            original_desktop,
            created_desktop,
        };
        
        debug!(
            "注册空间: hwnd={:?}, original={}, created={}",
            hwnd, original_desktop, created_desktop
        );
        
        self.spaces.insert(hwnd.0 as isize, info);
    }
    
    /// 检查窗口是否是全屏空间
    pub fn is_fullscreen_space(&self, hwnd: HWND) -> bool {
        self.spaces.contains_key(&(hwnd.0 as isize))
    }
    
    /// 获取空间信息
    pub fn get(&self, hwnd: HWND) -> Option<&SpaceInfo> {
        self.spaces.get(&(hwnd.0 as isize))
    }
    
    /// 移除空间
    pub fn remove(&mut self, hwnd: HWND) -> Option<SpaceInfo> {
        let info = self.spaces.remove(&(hwnd.0 as isize));
        if info.is_some() {
            debug!("移除空间: hwnd={:?}", hwnd);
        }
        info
    }
    
    /// 更新桌面索引（当删除桌面后，后面的索引需要减 1）
    pub fn update_indices_after_delete(&mut self, deleted_index: i32) {
        for info in self.spaces.values_mut() {
            if info.original_desktop > deleted_index {
                info.original_desktop -= 1;
            }
            if info.created_desktop > deleted_index {
                info.created_desktop -= 1;
            }
        }
        debug!("更新桌面索引: 删除了索引 {}", deleted_index);
    }
    
    /// 获取所有注册的窗口句柄
    pub fn all_hwnds(&self) -> Vec<HWND> {
        self.spaces.values().map(|s| s.hwnd).collect()
    }
    
    /// 检查窗口是否在注册表中
    pub fn contains(&self, hwnd: HWND) -> bool {
        self.spaces.contains_key(&(hwnd.0 as isize))
    }
    
    /// 获取注册的空间数量
    pub fn len(&self) -> usize {
        self.spaces.len()
    }
    
    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.spaces.is_empty()
    }
    
    /// 调试输出
    pub fn debug_info(&self) -> String {
        if self.spaces.is_empty() {
            return "空间注册表为空".to_string();
        }
        
        let mut info = format!("已注册 {} 个空间:\n", self.spaces.len());
        for (_, space) in &self.spaces {
            info.push_str(&format!(
                "  - hwnd={:?}, 原桌面={}, 创建桌面={}\n",
                space.hwnd, space.original_desktop, space.created_desktop
            ));
        }
        info
    }
}

impl Default for SpaceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
