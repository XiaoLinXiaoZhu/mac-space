; ============================================================================
; MacSpaces - 系统事件监听
; 监听窗口关闭等系统事件
; ============================================================================
#Requires AutoHotkey v2.0

class Hooks {
    static pollTimer := 0
    static pollInterval := 500  ; 轮询间隔（毫秒）
    
    ; 初始化事件监听
    static Init() {
        ; 使用定时轮询方式检测窗口关闭
        ; Shell Hook 在 AHK v2 中实现较复杂，先用轮询作为 MVP
        this.pollTimer := SetTimer(ObjBindMethod(this, "PollWindowStatus"), this.pollInterval)
        
        if Config.Debug {
            ToolTip("事件监听已启动")
            SetTimer(() => ToolTip(), -1000)
        }
    }
    
    ; 停止事件监听
    static Stop() {
        if this.pollTimer {
            SetTimer(ObjBindMethod(this, "PollWindowStatus"), 0)
            this.pollTimer := 0
        }
    }
    
    ; 轮询检查窗口状态
    static PollWindowStatus() {
        ; 获取所有注册的窗口
        hwnds := SpaceRegistry.GetAllHwnds()
        
        for hwnd in hwnds {
            ; 检查窗口是否还存在
            if !WinExist(hwnd) {
                ; 窗口已关闭，处理清理逻辑
                this.HandleWindowClosed(hwnd)
            }
        }
    }
    
    ; 处理窗口关闭事件
    static HandleWindowClosed(hwnd) {
        info := SpaceRegistry.Get(hwnd)
        if !info {
            return
        }
        
        if Config.Debug {
            ToolTip("检测到窗口关闭: hwnd=" hwnd)
            SetTimer(() => ToolTip(), -1000)
        }
        
        createdDesktop := info.createdDesktop
        currentDesktop := VirtualDesktop.GetCurrent()
        
        ; 如果当前在即将删除的桌面上，先切换走
        if currentDesktop = createdDesktop {
            ; 切换到左边的桌面（如果有的话）
            targetDesktop := createdDesktop > 0 ? createdDesktop - 1 : 0
            
            ; 使用带动画的切换
            if targetDesktop < currentDesktop {
                VirtualDesktop.GoLeftAnimated()
            } else {
                VirtualDesktop.GoTo(targetDesktop)
            }
            
            Sleep(Config.SwitchDelay)
        }
        
        ; 删除空桌面
        try {
            ; 先检查桌面数量，确保不会删除最后一个桌面
            if VirtualDesktop.GetCount() > 1 {
                VirtualDesktop.Remove(createdDesktop)
                
                ; 更新其他空间的桌面索引
                SpaceRegistry.UpdateDesktopIndices(createdDesktop)
            }
        } catch Error as e {
            if Config.Debug {
                ToolTip("删除桌面失败: " e.Message)
                SetTimer(() => ToolTip(), -2000)
            }
        }
        
        ; 从注册表移除
        SpaceRegistry.Remove(hwnd)
    }
}
