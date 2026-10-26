<div align="center">

<img src="../../assets/icon-256.png" width="128" alt="MAZ Launcher 图标">

# MAZ Launcher

**你的所有游戏,一个启动器搞定。**

将 Steam、Epic Games 和你自己的游戏汇集到一个快速、轻量、美观的 Windows 应用中,100% 使用 Rust 编写。

[![最新版本](https://img.shields.io/github/v/release/MikeAdamRGB/MAZ-Lauchner?label=Release&color=8b5cf6)](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases/latest)
[![下载量](https://img.shields.io/github/downloads/MikeAdamRGB/MAZ-Lauchner/total?label=Downloads&color=ec4899)](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases)
[![Build](https://github.com/MikeAdamRGB/MAZ-Lauchner/actions/workflows/build.yml/badge.svg)](https://github.com/MikeAdamRGB/MAZ-Lauchner/actions/workflows/build.yml)
![Windows](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?logo=windows&logoColor=white)
![Rust](https://img.shields.io/badge/100%25-Rust-DEA584?logo=rust&logoColor=white)
[![License: MIT](https://img.shields.io/badge/License-MIT-22c55e)](../../LICENSE)

[English](../../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português](README.pt-BR.md) · [Polski](README.pl.md) · [Türkçe](README.tr.md) · [Русский](README.ru.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · **简体中文**

[**⬇️ 下载 Windows 版**](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases/latest)

<br>

<img src="../screenshots/library-epic-style.webp" alt="Epic Games 风格的 MAZ Launcher 游戏库" width="900">

</div>

---

> [!NOTE]
> 目前应用界面仅提供**德语**。下文引用的按钮名称(例如 **“Spiel hinzufügen”**)与应用中显示的一致,并附有中文翻译。

## 功能

| | |
|---|---|
| 🎮 **Steam** | 自动识别所有磁盘上已安装的 Steam 游戏。填写可选的 Steam Web API 密钥后,还能显示你拥有的**全部**游戏及游玩时长。 |
| 🟦 **Epic Games** | 自动识别所有已安装的 Epic 游戏。 |
| 🚀 **自定义游戏** | 可添加任何游戏:`.exe`、快捷方式或批处理文件。**每次**启动前,检查清单都会询问 *“重要的东西都关掉了吗?”*,选择**否**则不会启动游戏。 |
| 🖼️ **原版图标与封面** | 直接从 EXE 或 Steam 获取真实图标和官方封面,也可以为任意游戏设置自定义封面。 |
| 🎨 **3 种布局** | 可以像 **Steam 库**、像 **Epic Games**,或使用启动器自己的 **MAZ 风格**。 |
| 🌈 **8 种配色** | MAZ 紫、午夜、霓虹、余烬、森林、樱花、OLED 黑和浅色,另可自定义强调色。 |
| 🛟 **通俗易懂的错误说明** | 无法启动时,启动器会**一步一步**说明发生了什么以及如何解决,并提供可直接处理问题的按钮(例如 *以管理员身份运行* 或 *下载 Visual C++*)。 |
| 📦 **内置安装程序** | 只有一个 EXE:双击即可打开安装菜单,包含桌面快捷方式、开始菜单项、*立即打开* 和 *打开应用文件夹*。无需管理员权限。 |
| 🪶 **极致小巧** | 单个经过体积优化的 EXE,没有额外文件。 |

## 截图

<table>
  <tr>
    <td width="50%" align="center">
      <img src="../screenshots/library-steam-style.webp" alt="Steam 风格的游戏库"><br>
      <sub><b>Steam 风格</b>:左侧游戏列表,“最近游玩”及全部游戏网格</sub>
    </td>
    <td width="50%" align="center">
      <img src="../screenshots/library-epic-style.webp" alt="Epic Games 风格的游戏库"><br>
      <sub><b>Epic Games 风格</b>:大卡片,顶部筛选</sub>
    </td>
  </tr>
  <tr>
    <td width="50%" align="center">
      <img src="../screenshots/game-details.webp" alt="游戏详情"><br>
      <sub><b>游戏详情</b>:开始游戏、收藏、打开文件夹、自定义封面</sub>
    </td>
    <td width="50%" align="center">
      <img src="../screenshots/settings-design.webp" alt="外观设置"><br>
      <sub><b>外观设置</b>:3 种布局、8 种配色和强调色</sub>
    </td>
  </tr>
</table>

## 安装

**系统要求:** Windows 10 或 11(64 位),并安装 [Microsoft WebView2](https://developer.microsoft.com/microsoft-edge/webview2/)(新版 Windows 已预装)。

1. 从 [**Releases**](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases/latest) 下载最新的 **`MAZ-Launcher.exe`**。
2. **双击**该文件。
3. 在安装菜单中点击 **“Installieren”**(*安装*)。
4. 完成!点击 **“Jetzt öffnen”**(*立即打开*)。之后通过桌面快捷方式启动即可。

> [!TIP]
> **Windows 提示“Windows 已保护你的电脑”?**
> 没有付费代码签名证书的新程序会出现此提示。点击 **更多信息**,再点击 **仍要运行**。

> [!TIP]
> **想用便携版?** 在安装菜单中点击 **“Ohne Installation starten”**(*免安装启动*),或在 EXE 旁边放一个名为 `portable.txt` 的空文件。

**`MAZ-Launcher-mini.exe`** 是用 UPX 压缩的同一应用。部分杀毒软件可能误报压缩过的 EXE,此时请使用普通版本。

**卸载:** Windows 设置 → 应用 → “MAZ Launcher” → 卸载,或在启动器的 **Einstellungen → Allgemein**(*设置 → 常规*)中卸载。你的游戏不会受到影响。

## 使用方法

- **开始游戏:** 将鼠标悬停在游戏上并点击 ▶。点击封面可打开详情。
- **添加自定义游戏:** 点击左侧 **“Spiel hinzufügen”**(*添加游戏*)→ 在“Programm”处点击 **“Durchsuchen…”**(*浏览…*)→ 选择游戏的可执行文件 → **“Hinzufügen”**(*添加*)。
- **更改外观:** **Einstellungen → Design** → 选择布局(MAZ / Steam / Epic)和配色。
- **显示所有 Steam 游戏(包括未安装的):** **Einstellungen → Verbindungen**(*连接*)→ 填写免费的 Steam Web API 密钥,说明就在旁边。
- **自定义检查清单:** **Einstellungen → Eigene Spiele**(*自定义游戏*)→ 添加、修改或删除条目,也可以按游戏单独设置。

### 启动器会自动解释的错误

游戏文件丢失 · 驱动器或 U 盘未连接 · 需要管理员权限 · 拒绝访问 · 选错文件 · 被杀毒软件拦截 · 被 Windows / Smart App Control 拦截 · 文件被占用 · 缺少 DLL(Visual C++ / DirectX)· 错误 **0xc000007b** · **0xc0000142** · 启动后立即崩溃 · 未安装 Steam 或 Epic · Steam 或 Epic 无响应 · Steam API 密钥无效 · 缺少 WebView2 · 安装失败。

## 从源码构建

**要求:** Windows 上的 [Rust](https://rustup.rs)(stable,1.85 或更高版本)。无需其他工具:不需要 Node.js,也不需要 Dioxus CLI。

```powershell
git clone https://github.com/MikeAdamRGB/MAZ-Lauchner.git
cd MAZ-Lauchner
cargo run                 # 开发
cargo build --release     # 发布版 EXE:target\release\MAZ-Launcher.exe
.\scripts\build.ps1 -Mini # 将 EXE 复制到 dist\,并额外生成 UPX 压缩的 mini 版本
```

编译好的样式表(`assets/maz.css`)已包含在仓库中。修改 Tailwind 类后,请运行 `.\scripts\build-css.ps1`。

技术细节(技术栈、项目结构、体积优化)请参阅 [英文 README](../../README.md#tech-stack)。

## 常见问题

**启动器会下载游戏或绕过版权保护吗?**
不会。Steam 和 Epic 游戏始终通过官方启动器启动,也从不索要密码。

**为什么看不到我所有的 Epic 游戏?**
Epic 只有在登录后才提供已购游戏列表。MAZ Launcher 会显示所有**已安装**的 Epic 游戏。

**窗口一直是白色或无法打开。**
启动器需要 Microsoft WebView2(Windows 10/11 通常自带)。如果缺失,启动器会在启动时说明如何安装。

**我的数据会被发送到其他地方吗?**
没有账户,也没有遥测。网络仅用于获取封面图片,以及在你填写密钥时访问 Steam Web API。

## 参与贡献

欢迎提交错误报告、想法和 Pull Request:[提交 Issue](https://github.com/MikeAdamRGB/MAZ-Lauchner/issues/new/choose) · [CONTRIBUTING.md](../../CONTRIBUTING.md)


## 扩展与 Minecraft

**Minecraft 与 NoRisk Client:** 官方 Minecraft Launcher 和每个 NoRisk 配置都会显示为单独的磁贴。在“Minecraft”标签页中可以安装来自 Modrinth 和 CurseForge 的模组、资源包、光影和数据包。

**扩展**(默认关闭):添加新游戏来源、信息或按钮的小型外部程序。安装前会显示全部权限和校验和。如实说明:扩展以你的用户权限运行,请只安装你信任的扩展。详情:[docs/EXTENSIONS.md](../EXTENSIONS.md)。

## 许可证

基于 [MIT 许可证](../../LICENSE) 发布 © MikeAdamRGB。

组件设计参考了 [rust-ui](https://github.com/rust-ui/ui)(MIT)。
Steam 是 Valve Corporation 的商标,Epic Games 是 Epic Games, Inc. 的商标。本项目为独立项目,与上述两家公司均无关联。
