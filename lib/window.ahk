; ============================================================================
; MacSpaces - 窗口操作模块
; ============================================================================
#Requires AutoHotkey v2.0

class WindowHelper {
    ; 获取当前活动窗口句柄
    static GetActive() {
        return WinGetID("A")
    }
    
    ; 获取窗口类名
    static GetClass(hwnd) {
        return WinGetClass(hwnd)
    }
    
    ; 获取窗口标题
    static GetTitle(hwnd) {
        return WinGetTitle(hwnd)
    }
    
    ; 获取窗口进程名
    static GetProcessName(hwnd) {
        return WinGetProcessName(hwnd)
    }
    
    ; 检查是否是 UWP 应用窗口
    static IsUWP(hwnd) {
        className := this.GetClass(hwnd)
        return className = "ApplicationFrameWindow"
    }
    
    ; 获取 UWP 应用的真实窗口句柄
    ; UWP 应用的窗口结构：ApplicationFrameWindow > ApplicationFrameInputSinkWindow > 真实窗口
    static GetRealUWPWindow(hwnd) {
        if !this.IsUWP(hwnd) {
            return hwnd
        }
        
        ; 尝试获取子窗口
        childHwnd := DllCall("GetWindow", "Ptr", hwnd, "UInt", 5, "Ptr")  ; GW_CHILD = 5
        if childHwnd {
            return childHwnd
        }
        
        return hwnd
    }
    
    ; 检查窗口是否有效（可操作）
    static IsValid(hwnd) {
        if !hwnd {
            return false
        }
        
        ; 检查窗口是否存在
        if !WinExist(hwnd) {
            return false
        }
        
        ; 排除桌面窗口
        className := this.GetClass(hwnd)
        if className = "Progman" || className = "WorkerW" {
            return false
        }
        
        ; 排除任务栏
        if className = "Shell_TrayWnd" || className = "Shell_SecondaryTrayWnd" {
            return false
        }
        
        return true
    }
    
    ; 最大化窗口
    static Maximize(hwnd) {
        WinMaximize(hwnd)
    }
    
    ; 还原窗口
    static Restore(hwnd) {
        WinRestore(hwnd)
    }
    
    ; 检查窗口是否最大化
    static IsMaximized(hwnd) {
        return WinGetMinMax(hwnd) = 1
    }
    
    ; 切换窗口最大化状态
    static ToggleMaximize(hwnd) {
        if this.IsMaximized(hwnd) {
            this.Restore(hwnd)
        } else {
            this.Maximize(hwnd)
        }
    }
}
