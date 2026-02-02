; ============================================================================
; MacSpaces - 空间注册表
; 追踪窗口-桌面映射关系
; ============================================================================
#Requires AutoHotkey v2.0

class SpaceRegistry {
    ; 存储全屏空间信息: HWND -> SpaceInfo
    static spaces := Map()
    
    ; 注册全屏空间
    ; @param hwnd 窗口句柄
    ; @param originalDesktop 原始桌面索引
    ; @param createdDesktop 创建的桌面索引
    static Register(hwnd, originalDesktop, createdDesktop) {
        this.spaces[hwnd] := {
            hwnd: hwnd,
            originalDesktop: originalDesktop,
            createdDesktop: createdDesktop,
            createdAt: A_TickCount
        }
        
        if Config.Debug {
            ToolTip("注册空间: hwnd=" hwnd " orig=" originalDesktop " created=" createdDesktop)
            SetTimer(() => ToolTip(), -2000)
        }
    }
    
    ; 检查是否是全屏空间窗口
    static IsFullscreenSpace(hwnd) {
        return this.spaces.Has(hwnd)
    }
    
    ; 获取空间信息
    static Get(hwnd) {
        return this.spaces.Has(hwnd) ? this.spaces[hwnd] : false
    }
    
    ; 移除注册
    static Remove(hwnd) {
        if this.spaces.Has(hwnd) {
            this.spaces.Delete(hwnd)
            
            if Config.Debug {
                ToolTip("移除空间: hwnd=" hwnd)
                SetTimer(() => ToolTip(), -1000)
            }
        }
    }
    
    ; 清理无效条目（窗口已关闭但未触发事件）
    static Cleanup() {
        toRemove := []
        
        for hwnd, info in this.spaces {
            if !WinExist(hwnd) {
                toRemove.Push(hwnd)
            }
        }
        
        for hwnd in toRemove {
            this.Remove(hwnd)
        }
        
        return toRemove.Length
    }
    
    ; 更新桌面索引（当桌面被删除时，后面的索引需要减1）
    static UpdateDesktopIndices(deletedIndex) {
        for hwnd, info in this.spaces {
            if info.originalDesktop > deletedIndex {
                info.originalDesktop--
            }
            if info.createdDesktop > deletedIndex {
                info.createdDesktop--
            }
        }
    }
    
    ; 获取所有注册的空间数量
    static Count() {
        return this.spaces.Count
    }
    
    ; 获取所有注册的窗口句柄
    static GetAllHwnds() {
        hwnds := []
        for hwnd, info in this.spaces {
            hwnds.Push(hwnd)
        }
        return hwnds
    }
    
    ; 根据创建的桌面索引查找窗口
    static FindByCreatedDesktop(desktopIndex) {
        for hwnd, info in this.spaces {
            if info.createdDesktop = desktopIndex {
                return hwnd
            }
        }
        return 0
    }
    
    ; 调试：打印所有注册的空间
    static DebugPrint() {
        msg := "=== SpaceRegistry ===`n"
        msg .= "总数: " this.spaces.Count "`n"
        
        for hwnd, info in this.spaces {
            title := ""
            try {
                title := WinGetTitle(hwnd)
            }
            msg .= "- hwnd=" hwnd " title=" title "`n"
            msg .= "  orig=" info.originalDesktop " created=" info.createdDesktop "`n"
        }
        
        return msg
    }
}
