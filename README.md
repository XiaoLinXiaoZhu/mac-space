# MacSpaces

**在 Windows 上复现 Mac 的空间化桌面体验**

将 Windows 虚拟桌面变成 Mac 风格的"空间"——每个全屏应用独占一个空间，左右滑动切换，关闭时自动清理。

## ✨ 功能

| 快捷键 | 功能 |
|--------|------|
| `Win+F` | 将当前窗口移到新空间并全屏（再按一次退出） |
| `Win+←` | 切换到左边的空间（带动画） |
| `Win+→` | 切换到右边的空间（带动画） |

### 核心体验

- **一键全屏**：`Win+F` 创建独立空间，窗口自动全屏（F11）
- **一键退出**：再按 `Win+F` 返回原桌面，空间自动删除
- **自动清理**：关闭全屏窗口时，对应空间自动删除
- **带动画切换**：与系统原生 `Win+Ctrl+←/→` 体验一致

## 🚀 快速开始

### 1. 安装 AutoHotkey v2

从 [AutoHotkey 官网](https://www.autohotkey.com/) 下载并安装 v2 版本。

### 2. 下载 VirtualDesktopAccessor.dll

从 [VirtualDesktopAccessor Releases](https://github.com/Ciantic/VirtualDesktopAccessor/releases) 下载最新版本，放入 `assets/` 目录。

### 3. 运行

双击 `mac-spaces.ahk` 启动。

## 📁 项目结构

```
mac-spaces/
├── mac-spaces.ahk          # 主脚本
├── lib/
│   ├── config.ahk          # 配置
│   ├── desktop.ahk         # 桌面 API
│   ├── window.ahk          # 窗口操作
│   ├── registry.ahk        # 空间注册表
│   └── hooks.ahk           # 事件监听
├── assets/
│   └── VirtualDesktopAccessor.dll
└── docs/
    └── TECHNICAL.md        # 技术文档
```

## ⚙️ 配置

编辑 `lib/config.ahk` 可自定义：

```autohotkey
class Config {
    static SwitchDelay := 150      ; 切换延迟（毫秒）
    static CyclicSwitch := false   ; 是否循环切换
    static Debug := false          ; 调试模式
}
```

## 🔧 托盘菜单

右键系统托盘图标：

- **桌面信息**：查看当前桌面数量和索引
- **空间注册表**：查看所有全屏空间
- **调试模式**：启用详细提示
- **重新加载**：重启脚本
- **退出**：关闭程序

## ⚠️ 注意事项

- 需要 Windows 10/11
- VirtualDesktopAccessor.dll 版本需与 Windows 版本兼容
- F11 全屏依赖应用支持（浏览器、VSCode 等大多支持）

## 📄 许可证

MIT

## 🙏 致谢

- [VirtualDesktopAccessor](https://github.com/Ciantic/VirtualDesktopAccessor) - Windows 虚拟桌面 API
- [AutoHotkey](https://www.autohotkey.com/) - 脚本运行环境
