# MacSpaces 技术文档

## 架构概览

```
┌─────────────────────────────────────────────┐
│              mac-spaces.ahk                  │
│           (主脚本 - 入口点)                   │
├─────────────────────────────────────────────┤
│  快捷键绑定    托盘菜单    初始化/清理        │
└──────────────────┬──────────────────────────┘
                   │
    ┌──────────────┼──────────────┬────────────┐
    ▼              ▼              ▼            ▼
┌─────────┐  ┌─────────┐  ┌──────────┐  ┌─────────┐
│ desktop │  │ window  │  │ registry │  │  hooks  │
│  .ahk   │  │  .ahk   │  │   .ahk   │  │  .ahk   │
│ 桌面API │  │ 窗口操作│  │ 空间注册 │  │ 事件监听│
└────┬────┘  └─────────┘  └──────────┘  └─────────┘
     │
     ▼
┌───────────────────────────────────┐
│   VirtualDesktopAccessor.dll      │
│      (Windows 虚拟桌面 API)        │
└───────────────────────────────────┘
```

## 模块说明

### lib/config.ahk

配置文件，包含：
- DLL 路径
- 快捷键配置
- 切换延迟时间
- 调试开关

### lib/desktop.ahk

`VirtualDesktop` 类，封装 VirtualDesktopAccessor.dll 的调用：

| 方法 | 说明 |
|------|------|
| `Init(dllPath)` | 加载 DLL |
| `Cleanup()` | 卸载 DLL |
| `GetCount()` | 获取桌面总数 |
| `GetCurrent()` | 获取当前桌面索引 (0-based) |
| `GoTo(index)` | 切换到指定桌面（无动画） |
| `GoLeftAnimated()` | 切换到左边桌面（带动画） |
| `GoRightAnimated()` | 切换到右边桌面（带动画） |
| `GoToAnimated(index)` | 切换到指定桌面（带动画） |
| `Create()` | 创建新桌面 |
| `Remove(index)` | 删除指定桌面 |
| `MoveWindow(hwnd, index)` | 移动窗口到指定桌面 |
| `MoveWindowToNewDesktop(hwnd, maximize)` | 移动窗口到新桌面 |
| `MoveWindowBackToOriginal(hwnd, orig, created)` | 移动窗口回原桌面 |

### lib/window.ahk

`WindowHelper` 类，窗口操作辅助函数：

| 方法 | 说明 |
|------|------|
| `GetActive()` | 获取当前活动窗口 |
| `IsValid(hwnd)` | 检查窗口是否有效 |
| `IsUWP(hwnd)` | 检查是否是 UWP 应用 |
| `IsMaximized(hwnd)` | 检查窗口是否最大化 |
| `Maximize(hwnd)` | 最大化窗口 |
| `Restore(hwnd)` | 还原窗口 |

### lib/registry.ahk

`SpaceRegistry` 类，追踪窗口-桌面映射关系：

| 方法 | 说明 |
|------|------|
| `Register(hwnd, orig, created)` | 注册全屏空间 |
| `IsFullscreenSpace(hwnd)` | 检查是否是全屏空间窗口 |
| `Get(hwnd)` | 获取空间信息 |
| `Remove(hwnd)` | 移除注册 |
| `Cleanup()` | 清理无效条目 |
| `UpdateDesktopIndices(deleted)` | 更新桌面索引 |

### lib/hooks.ahk

`Hooks` 类，系统事件监听：

| 方法 | 说明 |
|------|------|
| `Init()` | 初始化事件监听 |
| `Stop()` | 停止事件监听 |
| `PollWindowStatus()` | 轮询检查窗口状态 |
| `HandleWindowClosed(hwnd)` | 处理窗口关闭事件 |

## 快捷键

| 快捷键 | 功能 | 实现函数 |
|--------|------|----------|
| `Win+←` | 切换到左边桌面（带动画） | `SwitchDesktopLeft()` |
| `Win+→` | 切换到右边桌面（带动画） | `SwitchDesktopRight()` |
| `Win+F` | 切换全屏空间（进入/退出） | `ToggleFullscreenSpace()` |

## 核心逻辑

### Win+F 切换逻辑

```
┌─────────────────────────────────────────────────────────────────┐
│  Win+F 行为                                                      │
│                                                                 │
│  IF 窗口不在 SpaceRegistry 中（普通窗口）:                       │
│    → 进入全屏空间                                               │
│    1. 记录当前桌面为 originalDesktop                            │
│    2. 创建新桌面                                                │
│    3. 移动窗口到新桌面                                          │
│    4. 切换到新桌面（带动画）                                    │
│    5. 最大化窗口                                                │
│    6. 注册到 SpaceRegistry                                      │
│                                                                 │
│  ELSE（全屏空间窗口）:                                          │
│    → 退出全屏空间                                               │
│    1. 还原窗口（取消最大化）                                    │
│    2. 移动窗口回 originalDesktop                                │
│    3. 切换到 originalDesktop（带动画）                          │
│    4. 删除创建的空桌面                                          │
│    5. 更新其他空间的桌面索引                                    │
│    6. 从 SpaceRegistry 移除                                     │
└─────────────────────────────────────────────────────────────────┘
```

### 窗口关闭自动清理

```
┌─────────────────────────────────────────────────────────────────┐
│  窗口关闭监听（轮询方式，每 500ms）                              │
│                                                                 │
│  FOR EACH hwnd IN SpaceRegistry:                                │
│    IF 窗口不存在:                                               │
│      1. 获取 createdDesktop                                     │
│      2. 如果当前在 createdDesktop，先切换到相邻桌面             │
│      3. 删除 createdDesktop                                     │
│      4. 更新其他空间的桌面索引                                  │
│      5. 从 SpaceRegistry 移除                                   │
└─────────────────────────────────────────────────────────────────┘
```

### 动画切换实现

```
┌─────────────────────────────────────────────────────────────────┐
│  动画切换原理                                                    │
│                                                                 │
│  问题：VirtualDesktopAccessor 的 GoToDesktopNumber 无动画       │
│                                                                 │
│  解决方案：模拟系统快捷键                                        │
│  - GoLeftAnimated()  → Send("^#{Left}")   ; Win+Ctrl+Left       │
│  - GoRightAnimated() → Send("^#{Right}")  ; Win+Ctrl+Right      │
│                                                                 │
│  多步切换：                                                      │
│  - GoToAnimated(target) 通过多次 Left/Right 实现                │
│  - 每次按键间隔 50ms，让动画有时间开始                          │
└─────────────────────────────────────────────────────────────────┘
```

## 状态管理

### SpaceRegistry 数据结构

```
Map<HWND, SpaceInfo> = {
  0x12345: {
    hwnd: 0x12345,
    originalDesktop: 0,      // 原始桌面（用于退出时返回）
    createdDesktop: 2,       // 创建的桌面索引（用于删除）
    createdAt: 123456789     // 创建时间戳
  },
  ...
}
```

### 桌面索引更新

当删除桌面时，后面的桌面索引会减 1：

```
删除前: [Desktop 0] [Desktop 1] [Desktop 2] [Desktop 3]
删除 Desktop 1 后: [Desktop 0] [Desktop 1] [Desktop 2]
                                    ↑           ↑
                              原 Desktop 2  原 Desktop 3
```

`SpaceRegistry.UpdateDesktopIndices(deletedIndex)` 会更新所有受影响的索引。

## 已知问题

### 1. UWP 应用处理

UWP 应用（如 Windows 设置、Microsoft Store 应用）使用 `ApplicationFrameWindow` 作为宿主窗口。移动这类窗口时可能需要特殊处理。

### 2. 切换延迟

Windows 虚拟桌面切换有动画效果，切换后需要等待约 150ms 才能进行后续操作。

### 3. DLL 兼容性

VirtualDesktopAccessor.dll 依赖 Windows 内部 API，Windows 大版本更新可能导致 DLL 失效。需要关注项目更新。

### 4. 轮询延迟

窗口关闭检测使用 500ms 轮询，可能有最多 500ms 的延迟。

## 调试

1. 在托盘菜单中启用"调试模式"
2. 或在 `lib/config.ahk` 中设置 `Debug := true`

调试模式下会显示：
- 操作提示（ToolTip）
- 错误信息（MsgBox）

托盘菜单还提供：
- "桌面信息"：显示当前桌面数量和索引
- "空间注册表"：显示所有注册的全屏空间

## 参考资料

- [VirtualDesktopAccessor](https://github.com/Ciantic/VirtualDesktopAccessor)
- [AutoHotkey v2 文档](https://www.autohotkey.com/docs/v2/)
- [Windows 虚拟桌面 API](https://docs.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-ivirtualdesktopmanager)
