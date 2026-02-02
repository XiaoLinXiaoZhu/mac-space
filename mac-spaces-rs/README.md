# MacSpaces Rust 版本

这是 MacSpaces 的 Rust 重写版本，解决了原 AHK 版本的性能问题和屏幕闪烁问题。

## 主要改进

### 性能优化
- **移除轮询机制**：使用 `SetWinEventHook` 监听窗口关闭事件，零 CPU 占用
- **原生 API 调用**：直接调用 `GoToDesktopNumber`（内部使用 `SwitchDesktop`，自带动画）
- **事件驱动架构**：所有操作都是事件驱动，不再有定时器轮询

### 稳定性改进
- **移除模拟按键**：不再使用 `Send("^#{Left}")` 模拟按键切换桌面
- **类型安全**：Rust 的类型系统避免了许多运行时错误
- **单实例保护**：防止重复启动

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Win + ←` | 切换到左边的桌面 |
| `Win + →` | 切换到右边的桌面 |
| `Win + F` | 切换全屏空间（进入/退出） |

## 构建

```bash
cd mac-spaces-rs
cargo build --release
```

## 运行

```bash
# 确保 DLL 在正确位置
mkdir target\release\assets
copy ..\assets\VirtualDesktopAccessor.dll target\release\assets\

# 运行
target\release\mac_spaces.exe
```

或者直接双击 `target\release\mac_spaces.exe`（需要 DLL 在 `assets` 子目录）。

## 文件结构

```
mac-spaces-rs/
├── Cargo.toml          # 项目配置
├── src/
│   ├── main.rs         # 主入口、消息循环
│   ├── vda.rs          # VirtualDesktopAccessor DLL 封装
│   ├── hotkey.rs       # 全局快捷键（低级键盘钩子）
│   ├── hooks.rs        # 窗口事件监听（SetWinEventHook）
│   ├── desktop.rs      # 桌面操作逻辑
│   ├── registry.rs     # 空间注册表
│   ├── window.rs       # 窗口辅助函数
│   └── tray.rs         # 托盘图标
└── target/
    └── release/
        ├── mac_spaces.exe
        └── assets/
            └── VirtualDesktopAccessor.dll
```

## 技术细节

### 为什么移除轮询？

原 AHK 版本每 500ms 轮询检查窗口状态：
```ahk
SetTimer(ObjBindMethod(this, "PollWindowStatus"), 500)
```

这会导致：
1. 持续的 CPU 占用
2. 多显示器环境下可能触发窗口重绘，导致闪烁

Rust 版本使用 `SetWinEventHook`：
```rust
SetWinEventHook(
    EVENT_OBJECT_DESTROY,
    EVENT_OBJECT_DESTROY,
    None,
    Some(win_event_proc),
    0, 0,
    WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
)
```

这是事件驱动的，只在窗口关闭时才会触发回调，零 CPU 占用。

### 为什么移除模拟按键？

原 AHK 版本通过模拟 `Win+Ctrl+Left/Right` 实现带动画的切换：
```ahk
Send("^#{Left}")
```

这种方式不稳定，可能被其他程序拦截，也可能与系统产生冲突。

经过调研发现，`VirtualDesktopAccessor.dll` 的 `GoToDesktopNumber` 函数内部调用的是 `IVirtualDesktopManagerInternal::SwitchDesktop`，这个方法本身就是带动画的。原 AHK 版本使用模拟按键是历史遗留问题。

## 依赖

- [windows-rs](https://github.com/microsoft/windows-rs) - Windows API 绑定
- [libloading](https://crates.io/crates/libloading) - DLL 动态加载
- [tray-icon](https://crates.io/crates/tray-icon) - 托盘图标
- [VirtualDesktopAccessor.dll](https://github.com/Ciantic/VirtualDesktopAccessor) - 虚拟桌面 API
