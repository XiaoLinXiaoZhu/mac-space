; ============================================================================
; MacSpaces - 配置与常量
; ============================================================================
#Requires AutoHotkey v2.0

class Config {
    ; DLL 路径（相对于主脚本）
    static DllPath := A_ScriptDir "\assets\VirtualDesktopAccessor.dll"
    
    ; 切换桌面后的等待时间（毫秒），用于等待动画完成
    static SwitchDelay := 150
    
    ; 快捷键配置
    static Hotkeys := {
        SwitchLeft: "#Left",      ; Win+← 切换到左边桌面
        SwitchRight: "#Right",    ; Win+→ 切换到右边桌面
        FullscreenNew: "#f"       ; Win+F 全屏到新桌面
    }
    
    ; 是否启用循环切换（最后一个桌面按右切换到第一个）
    static CyclicSwitch := false
    
    ; 调试模式
    static Debug := false
}
