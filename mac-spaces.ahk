; ============================================================================
; MacSpaces - Windows 虚拟桌面空间化体验工具
; 
; 功能：
;   Win+←     切换到左边的桌面（带动画）
;   Win+→     切换到右边的桌面（带动画）
;   Win+F     切换全屏空间（进入/退出）
;
; 作者：MacSpaces
; 版本：0.2.0
; ============================================================================
#Requires AutoHotkey v2.0
#SingleInstance Force

; 加载模块
#Include lib/config.ahk
#Include lib/desktop.ahk
#Include lib/window.ahk
#Include lib/registry.ahk
#Include lib/hooks.ahk

; ============================================================================
; 初始化
; ============================================================================

; 设置工作目录为脚本所在目录
SetWorkingDir(A_ScriptDir)

; 初始化虚拟桌面 API
try {
    VirtualDesktop.Init(Config.DllPath)
    if Config.Debug {
        MsgBox("VirtualDesktopAccessor.dll 加载成功！`n桌面数量: " VirtualDesktop.GetCount())
    }
} catch Error as e {
    MsgBox("初始化失败: " e.Message "`n`n请确保 VirtualDesktopAccessor.dll 位于 assets 目录中。", "MacSpaces 错误", "Icon!")
    ExitApp()
}

; 初始化事件监听
Hooks.Init()

; 创建托盘菜单
CreateTrayMenu()

; ============================================================================
; 快捷键绑定
; ============================================================================

; Win+← 切换到左边的桌面（带动画）
#Left::SwitchDesktopLeft()

; Win+→ 切换到右边的桌面（带动画）
#Right::SwitchDesktopRight()

; Win+F 切换全屏空间（进入/退出）
#f::ToggleFullscreenSpace()

; ============================================================================
; 功能实现
; ============================================================================

; 切换到左边的桌面
SwitchDesktopLeft() {
    if !VirtualDesktop.GoLeft(Config.CyclicSwitch, true) {
        ; 已经在最左边，可以播放提示音或显示通知
        if Config.Debug {
            ToolTip("已经是第一个桌面")
            SetTimer(() => ToolTip(), -1000)
        }
    }
}

; 切换到右边的桌面
SwitchDesktopRight() {
    if !VirtualDesktop.GoRight(Config.CyclicSwitch, true) {
        ; 已经在最右边
        if Config.Debug {
            ToolTip("已经是最后一个桌面")
            SetTimer(() => ToolTip(), -1000)
        }
    }
}

; 切换全屏空间（核心功能）
ToggleFullscreenSpace() {
    hwnd := WindowHelper.GetActive()
    
    ; 检查窗口是否有效
    if !WindowHelper.IsValid(hwnd) {
        if Config.Debug {
            ToolTip("无效的窗口")
            SetTimer(() => ToolTip(), -1000)
        }
        return
    }
    
    ; 检查是否已经是全屏空间
    if SpaceRegistry.IsFullscreenSpace(hwnd) {
        ; === 退出全屏空间 ===
        ExitFullscreenSpace(hwnd)
    } else {
        ; === 进入全屏空间 ===
        EnterFullscreenSpace(hwnd)
    }
}

; 进入全屏空间
EnterFullscreenSpace(hwnd) {
    try {
        ; 移动窗口到新桌面
        result := VirtualDesktop.MoveWindowToNewDesktop(hwnd, true)
        
        ; 注册到空间注册表
        SpaceRegistry.Register(hwnd, result.originalDesktop, result.newDesktop)
        
        if Config.Debug {
            ToolTip("进入全屏空间 #" (result.newDesktop + 1))
            SetTimer(() => ToolTip(), -1500)
        }
    } catch Error as e {
        if Config.Debug {
            MsgBox("进入全屏空间失败: " e.Message, "MacSpaces 错误", "Icon!")
        }
    }
}

; 退出全屏空间
ExitFullscreenSpace(hwnd) {
    info := SpaceRegistry.Get(hwnd)
    if !info {
        return
    }
    
    try {
        originalDesktop := info.originalDesktop
        createdDesktop := info.createdDesktop
        
        ; 移动窗口回原桌面并删除空桌面
        deleted := VirtualDesktop.MoveWindowBackToOriginal(hwnd, originalDesktop, createdDesktop)
        
        ; 如果成功删除了桌面，更新其他空间的索引
        if deleted {
            SpaceRegistry.UpdateDesktopIndices(createdDesktop)
        }
        
        ; 从注册表移除
        SpaceRegistry.Remove(hwnd)
        
        if Config.Debug {
            ToolTip("退出全屏空间，返回桌面 #" (originalDesktop + 1))
            SetTimer(() => ToolTip(), -1500)
        }
    } catch Error as e {
        if Config.Debug {
            MsgBox("退出全屏空间失败: " e.Message, "MacSpaces 错误", "Icon!")
        }
    }
}

; ============================================================================
; 托盘菜单
; ============================================================================

CreateTrayMenu() {
    ; 设置托盘图标提示
    A_IconTip := "MacSpaces - 虚拟桌面空间化"
    
    ; 创建托盘菜单
    tray := A_TrayMenu
    tray.Delete()  ; 清除默认菜单
    
    tray.Add("MacSpaces v0.2.0", (*) => {})
    tray.Disable("MacSpaces v0.2.0")
    tray.Add()  ; 分隔线
    
    tray.Add("桌面信息", ShowDesktopInfo)
    tray.Add("空间注册表", ShowSpaceRegistry)
    tray.Add()  ; 分隔线
    
    debugItem := "调试模式"
    tray.Add(debugItem, ToggleDebug)
    if Config.Debug {
        tray.Check(debugItem)
    }
    
    tray.Add()  ; 分隔线
    tray.Add("重新加载", (*) => Reload())
    tray.Add("退出", (*) => ExitMacSpaces())
}

; 显示桌面信息
ShowDesktopInfo(*) {
    count := VirtualDesktop.GetCount()
    current := VirtualDesktop.GetCurrent() + 1  ; 转为 1-based 显示
    
    MsgBox("桌面总数: " count "`n当前桌面: #" current, "MacSpaces 桌面信息", "Iconi")
}

; 显示空间注册表
ShowSpaceRegistry(*) {
    msg := SpaceRegistry.DebugPrint()
    MsgBox(msg, "MacSpaces 空间注册表", "Iconi")
}

; 切换调试模式
ToggleDebug(itemName, itemPos, menu) {
    Config.Debug := !Config.Debug
    if Config.Debug {
        menu.Check(itemName)
    } else {
        menu.Uncheck(itemName)
    }
}

; 退出程序
ExitMacSpaces() {
    Hooks.Stop()
    VirtualDesktop.Cleanup()
    ExitApp()
}

; ============================================================================
; 清理
; ============================================================================

OnExit(ExitFunc)

ExitFunc(exitReason, exitCode) {
    Hooks.Stop()
    VirtualDesktop.Cleanup()
}
