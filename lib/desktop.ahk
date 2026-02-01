; ============================================================================
; MacSpaces - 桌面管理模块
; 封装 VirtualDesktopAccessor.dll 的调用
; ============================================================================
#Requires AutoHotkey v2.0

class VirtualDesktop {
    static hModule := 0
    static dllPath := ""
    
    ; 初始化：加载 DLL
    static Init(dllPath) {
        this.dllPath := dllPath
        
        if !FileExist(dllPath) {
            throw Error("VirtualDesktopAccessor.dll 未找到: " dllPath)
        }
        
        this.hModule := DllCall("LoadLibrary", "Str", dllPath, "Ptr")
        if !this.hModule {
            throw Error("无法加载 VirtualDesktopAccessor.dll")
        }
        
        return true
    }
    
    ; 清理：卸载 DLL
    static Cleanup() {
        if this.hModule {
            DllCall("FreeLibrary", "Ptr", this.hModule)
            this.hModule := 0
        }
    }
    
    ; 获取桌面总数
    static GetCount() {
        return DllCall(this.dllPath "\GetDesktopCount", "Int")
    }
    
    ; 获取当前桌面索引（0-based）
    static GetCurrent() {
        return DllCall(this.dllPath "\GetCurrentDesktopNumber", "Int")
    }
    
    ; 切换到指定桌面（0-based）- 无动画
    static GoTo(index) {
        DllCall(this.dllPath "\GoToDesktopNumber", "Int", index)
    }
    
    ; 创建新桌面，返回新桌面的索引
    static Create() {
        ; 先获取当前桌面数
        countBefore := this.GetCount()
        
        ; 创建新桌面
        DllCall(this.dllPath "\CreateDesktop")
        
        ; 返回新桌面的索引（最后一个）
        return countBefore
    }
    
    ; 删除指定桌面
    static Remove(index) {
        ; 第二个参数是 fallback 桌面索引（窗口会被移动到这个桌面）
        fallback := index > 0 ? index - 1 : 0
        DllCall(this.dllPath "\RemoveDesktop", "Int", index, "Int", fallback)
    }
    
    ; 将窗口移动到指定桌面
    static MoveWindow(hwnd, desktopIndex) {
        DllCall(this.dllPath "\MoveWindowToDesktopNumber", "Ptr", hwnd, "Int", desktopIndex)
    }
    
    ; 获取窗口所在的桌面索引
    static GetWindowDesktop(hwnd) {
        return DllCall(this.dllPath "\GetWindowDesktopNumber", "Ptr", hwnd, "Int")
    }
    
    ; ========== 带动画的切换（模拟系统快捷键）==========
    
    ; 切换到左边的桌面（带动画）
    static GoLeftAnimated() {
        current := this.GetCurrent()
        if current > 0 {
            ; 模拟 Win+Ctrl+Left 按键
            Send("^#{Left}")
            return true
        }
        return false
    }
    
    ; 切换到右边的桌面（带动画）
    static GoRightAnimated() {
        current := this.GetCurrent()
        count := this.GetCount()
        if current < count - 1 {
            ; 模拟 Win+Ctrl+Right 按键
            Send("^#{Right}")
            return true
        }
        return false
    }
    
    ; 切换到指定桌面（带动画，通过多次左右切换实现）
    static GoToAnimated(targetIndex) {
        current := this.GetCurrent()
        
        if targetIndex = current {
            return true
        }
        
        ; 计算需要切换的次数和方向
        diff := targetIndex - current
        
        if diff > 0 {
            ; 向右切换
            Loop diff {
                Send("^#{Right}")
                Sleep(50)  ; 短暂延迟，让动画有时间开始
            }
        } else {
            ; 向左切换
            Loop Abs(diff) {
                Send("^#{Left}")
                Sleep(50)
            }
        }
        
        return true
    }
    
    ; ========== 高级操作 ==========
    
    ; 切换到左边的桌面（可选是否带动画）
    static GoLeft(cyclic := false, animated := true) {
        current := this.GetCurrent()
        if current > 0 {
            if animated {
                this.GoLeftAnimated()
            } else {
                this.GoTo(current - 1)
            }
            return true
        } else if cyclic {
            if animated {
                ; 循环切换需要多次按键，可能体验不好，直接跳转
                this.GoTo(this.GetCount() - 1)
            } else {
                this.GoTo(this.GetCount() - 1)
            }
            return true
        }
        return false
    }
    
    ; 切换到右边的桌面（可选是否带动画）
    static GoRight(cyclic := false, animated := true) {
        current := this.GetCurrent()
        count := this.GetCount()
        if current < count - 1 {
            if animated {
                this.GoRightAnimated()
            } else {
                this.GoTo(current + 1)
            }
            return true
        } else if cyclic {
            if animated {
                ; 循环切换直接跳转
                this.GoTo(0)
            } else {
                this.GoTo(0)
            }
            return true
        }
        return false
    }
    
    ; 将窗口移动到新桌面并切换过去
    static MoveWindowToNewDesktop(hwnd, maximize := true, sendF11 := true) {
        ; 1. 记录原始桌面
        originalDesktop := this.GetCurrent()
        
        ; 2. 如果窗口是最大化的，先还原
        wasMaximized := WinGetMinMax(hwnd) = 1
        if wasMaximized {
            WinRestore(hwnd)
            Sleep(50)
        }
        
        ; 3. 创建新桌面
        newIndex := this.Create()
        Sleep(50)
        
        ; 4. 移动窗口到新桌面
        this.MoveWindow(hwnd, newIndex)
        Sleep(50)
        
        ; 5. 切换到新桌面（带动画）
        this.GoToAnimated(newIndex)
        Sleep(Config.SwitchDelay)
        
        ; 6. 最大化窗口
        if maximize {
            WinMaximize(hwnd)
        }
        
        ; 7. 发送 F11 进入应用全屏模式
        if sendF11 {
            Sleep(100)
            Send("{F11}")
        }
        
        return {
            originalDesktop: originalDesktop,
            newDesktop: newIndex
        }
    }
    
    ; 将窗口从全屏空间移回原桌面
    static MoveWindowBackToOriginal(hwnd, originalDesktop, createdDesktop, sendF11 := true) {
        ; 1. 先发送 F11 退出应用全屏模式
        if sendF11 {
            Send("{F11}")
            Sleep(100)
        }
        
        ; 2. 还原窗口
        WinRestore(hwnd)
        Sleep(50)
        
        ; 3. 移动窗口回原桌面
        this.MoveWindow(hwnd, originalDesktop)
        Sleep(50)
        
        ; 4. 切换到原桌面（带动画）
        this.GoToAnimated(originalDesktop)
        Sleep(Config.SwitchDelay)
        
        ; 5. 删除空桌面（如果桌面数量大于1）
        if this.GetCount() > 1 {
            ; 注意：删除桌面后，索引会变化
            ; 如果 createdDesktop > originalDesktop，删除后不影响 originalDesktop
            ; 如果 createdDesktop < originalDesktop，这种情况不应该发生
            this.Remove(createdDesktop)
            return true
        }
        
        return false
    }
}
