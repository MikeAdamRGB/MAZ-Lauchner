<div align="center">

<img src="assets/icon-256.png" width="128" alt="MAZ Launcher logo">

# MAZ Launcher

**All your games. One launcher.**

Steam, Epic Games, Roblox and your own games in one fast, lightweight and good-looking app for **Windows, macOS and Linux**, written in 100% Rust. Everything runs locally – no account.

[![Latest release](https://img.shields.io/github/v/release/MikeAdamRGB/MAZ-Lauchner?label=Release&color=8b5cf6)](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/MikeAdamRGB/MAZ-Lauchner/total?label=Downloads&color=ec4899)](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases)
[![Build](https://github.com/MikeAdamRGB/MAZ-Lauchner/actions/workflows/build.yml/badge.svg)](https://github.com/MikeAdamRGB/MAZ-Lauchner/actions/workflows/build.yml)
![Windows](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?logo=windows&logoColor=white)
![macOS](https://img.shields.io/badge/macOS-11%2B-000000?logo=apple&logoColor=white)
![Linux](https://img.shields.io/badge/Linux-x86__64%20%7C%20ARM64-FCC624?logo=linux&logoColor=black)
![Rust](https://img.shields.io/badge/100%25-Rust-DEA584?logo=rust&logoColor=white)
[![License: MIT](https://img.shields.io/badge/License-MIT-22c55e)](LICENSE)

**English** · [Deutsch](docs/i18n/README.de.md) · [Français](docs/i18n/README.fr.md) · [Español](docs/i18n/README.es.md) · [Italiano](docs/i18n/README.it.md) · [Português](docs/i18n/README.pt-BR.md) · [Polski](docs/i18n/README.pl.md) · [Türkçe](docs/i18n/README.tr.md) · [Русский](docs/i18n/README.ru.md) · [日本語](docs/i18n/README.ja.md) · [한국어](docs/i18n/README.ko.md) · [简体中文](docs/i18n/README.zh-CN.md)

[**⬇️ Download**](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases/latest) · [**🌐 Website**](https://mikeadamrgb.github.io/MAZ-Lauchner/)

<br>

<img src="docs/screenshots/library-maz.webp" alt="MAZ Launcher library in its own MAZ style" width="900">

</div>

---

## Table of contents

- [Features](#features)
- [Screenshots](#screenshots)
- [Installation](#installation)
- [Usage](#usage)
- [Building from source](#building-from-source)
- [Tech stack](#tech-stack)
- [FAQ](#faq)
- [Contributing](#contributing)
- [License](#license)

## Features

| | |
|---|---|
| 🎮 **Steam** | Shows **all** your Steam games – installed or not – across all drives, including playtime. Install and uninstall them right from the launcher. Works on Windows, macOS and Linux (incl. Flatpak/Snap Steam). |
| 🟦 **Epic Games** | Detects every installed Epic Games title automatically. Connect your Epic account once to see your **complete** Epic library, then install or uninstall with one click. On Linux, Epic games come from the [Heroic Games Launcher](https://heroicgameslauncher.com). |
| 🧱 **Roblox** | Detects Roblox and starts it. Paste links of your favorite Roblox games – each becomes its own tile that starts straight into the game. On Linux via [Sober](https://flathub.org/apps/org.vinegarhq.Sober). |
| 🏪 **7 more stores** | GOG (Windows: GOG Galaxy, Linux: Heroic), EA app, Ubisoft Connect, Battle.net, Amazon Games, itch.io and Xbox / Microsoft Store. Installed games are found automatically and always started through the official app. GOG, Amazon and itch.io also show your **whole library** (read locally from their apps), so you can install games with one click. Every store can be turned off. |
| 🕹️ **Emulators** | Profiles for RetroArch, Dolphin, PCSX2, DuckStation, PPSSPP, mGBA, Snes9x or any other emulator. Pick your ROM folders – every ROM becomes its own game. |
| 🗂️ **Organize your library** | Search, sort (name, last played, playtime, recently added) and filter (store, installed, favorites, genre, tags, collections). Your own tags and collections, playtime tracking, duplicates across stores merged into one tile. |
| 📝 **Game info** | Description, genres, developer, release date, **minimum and recommended PC specs** and the **Steam Deck rating** – from the public Steam store, and from **Wikipedia/Wikidata** for everything else (e.g. ROMs, GOG or Xbox games). Cached locally; edit everything yourself. |
| ⛏️ **Minecraft & NoRisk Client** | The official Minecraft Launcher (Store, classic, macOS, Linux) and **every NoRisk Client profile** as its own tile – Play starts that exact profile. A built-in browser for **mods, resource packs, shaders and data packs** from Modrinth (no key) and CurseForge (your own free key, with a step-by-step guide) installs them into `.minecraft` or a NoRisk profile – including required dependencies and one-click Fabric setup. |
| 🧩 **Extensions** | Optional and off by default: add new game sources, game info or buttons through small external programs (any language, JSON over stdin/stdout). Every extension shows its permissions and checksum before you agree, is pinned by hash and is checked strictly. **Honest note:** extensions run with your user rights – only install ones you trust. See [docs/EXTENSIONS.md](docs/EXTENSIONS.md). |
| 🐧 **Wine & Proton per game** | On Linux, pick the Wine or Proton version for each Windows game (system Wine, Steam Proton, GE-Proton, Lutris and Heroic versions). |
| 🎁 **Free games** | A tab with every game that is currently free on Epic Games and Steam, with a countdown of how long the offer lasts. *Add to library* opens the store page – one click on *Get* and it shows up in MAZ Launcher. |
| 🚀 **Your own games** | Add any game: an `.exe`, a shortcut or a batch file. Before **every** launch a safety checklist asks *"Have you turned off everything important?"*. If you answer **No**, the game does not start. |
| 🖼️ **Original icons & covers** | Real game icons (taken straight from the EXE or from Steam) and original cover art. You can set a custom cover for any game. |
| 🎨 **3 library layouts** | Looks like **the Steam library**, like **Epic Games**, or uses the launcher's own **MAZ style**. |
| 🌈 **16 designs + your own** | 16 ready-made color themes, a design editor for your own themes with font, spacing and layout, shareable as a code or a `.maztheme.json` file, a custom accent color and your own background image. |
| 🎮 **Controller mode** | Optional: navigate the whole launcher with a gamepad (D-pad/stick, A, B, LB/RB). |
| 🌍 **9 languages** | English, Deutsch, Español, Français, Italiano, Português, Polski, Türkçe and Русский – including every error explanation. |
| 🛟 **Errors explained in plain words** | If a game won't start or crashes right after launch (Steam, Epic or your own), the launcher explains **step by step** what happened and how to fix it, with buttons that fix it directly (for example *Verify game files*, *Run as administrator* or *Download Visual C++*). Every game also has a *Won't start? Help* button. |
| 🔄 **Updates in the app** | You get a message when a new version is out. *Update now* downloads it, checks the SHA256 checksum and installs it – no trip to GitHub needed. |
| 🔒 **Everything local** | No MAZ account, no server, no tracking. Passwords stay with Steam, Epic and Roblox. |
| 📦 **Built-in installer (Windows)** | One single EXE: double-click it to get an install menu with a desktop shortcut, a Start menu entry, *Open now* and *Open app folder*. No admin rights required. |
| 🐧 **Every platform** | Windows (x86_64, ARM64), macOS (Apple Silicon + Intel), Linux (x86_64, ARM64) as AppImage, .deb, .rpm, .pkg.tar.zst, Flatpak and Snap. |
| 🪶 **Tiny footprint** | One size-optimized program with no extra files. |

## Screenshots

<table>
  <tr>
    <td width="50%" align="center">
      <img src="docs/screenshots/library-maz.webp" alt="MAZ style"><br>
      <sub><b>MAZ style</b>: its own look with a "Continue playing" banner and all games in a grid</sub>
    </td>
    <td width="50%" align="center">
      <img src="docs/screenshots/installed.webp" alt="Installed"><br>
      <sub><b>Installed</b>: only the games that are on your PC</sub>
    </td>
  </tr>
  <tr>
    <td width="50%" align="center">
      <img src="docs/screenshots/library-steam-style.webp" alt="Steam style"><br>
      <sub><b>Steam style</b>: game list on the left and all games in a grid</sub>
    </td>
    <td width="50%" align="center">
      <img src="docs/screenshots/details-steam-style.webp" alt="Steam-style details"><br>
      <sub><b>Steam-style details</b>: install, playtime, store page and more</sub>
    </td>
  </tr>
  <tr>
    <td width="50%" align="center">
      <img src="docs/screenshots/library-epic-style.webp" alt="Epic Games style"><br>
      <sub><b>Epic Games style</b>: large cards with filters along the top</sub>
    </td>
    <td width="50%" align="center">
      <img src="docs/screenshots/game-details.webp" alt="Game details"><br>
      <sub><b>Game details</b>: play or install, favorite, custom cover, help and uninstall</sub>
    </td>
  </tr>
  <tr>
    <td width="50%" align="center">
      <img src="docs/screenshots/free-games.webp" alt="Free games"><br>
      <sub><b>Free games</b>: what is free on Epic and Steam right now, with a countdown</sub>
    </td>
    <td width="50%" align="center">
      <img src="docs/screenshots/installed.webp" alt="Installed"><br>
      <sub><b>Installed</b>: only the games that are on your computer</sub>
    </td>
  </tr>
  <tr>
    <td width="50%" align="center">
      <img src="docs/screenshots/settings-design.webp" alt="Design settings"><br>
      <sub><b>Design settings</b>: 16 ready-made designs, your own designs and a custom accent color</sub>
    </td>
    <td width="50%" align="center">
      <img src="docs/screenshots/settings-language.webp" alt="Languages"><br>
      <sub><b>Languages</b>: the whole launcher in 9 languages</sub>
    </td>
  </tr>
</table>

## Installation

All files are on the [**Releases**](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases/latest) page. `SHA256SUMS.txt` lists a checksum for every file.

### Windows 10 / 11

1. Download **`MAZ-Launcher.exe`** (Windows on ARM: **`MAZ-Launcher-arm64.exe`**).
2. **Double-click** the file and click **Install**, then **Open now**.

> [!TIP]
> **Windows shows "Windows protected your PC"?** This happens with new programs that aren't signed with a paid certificate. Click **More info**, then **Run anyway**.

Prefer a portable version? Click **Start without installing**, or put an empty file named `portable.txt` next to the EXE. `MAZ-Launcher-mini.exe` is the same app packed with UPX.

### macOS 11 or newer (Apple Silicon + Intel)

1. Download **`MAZ-Launcher-…-macOS.dmg`**, open it and drag **MAZ Launcher** into **Applications**.
2. The first time: **right-click** the app → **Open** → **Open**. macOS asks once because the app isn't from the App Store.

### Linux (x86_64 and ARM64)

| Distribution | File | Install |
|---|---|---|
| Any | `MAZ-Launcher-…-x86_64.AppImage` / `-aarch64.AppImage` | `chmod +x MAZ-Launcher-*.AppImage` and start it |
| Ubuntu, Debian, Mint, Pop!_OS | `maz-launcher_…_amd64.deb` / `_arm64.deb` | `sudo apt install ./maz-launcher_*.deb` |
| Fedora, openSUSE, Nobara | `maz-launcher-…-1.x86_64.rpm` / `.aarch64.rpm` | `sudo dnf install ./maz-launcher-*.rpm` |
| Arch, Manjaro, EndeavourOS, CachyOS | `maz-launcher-…-x86_64.pkg.tar.zst` | `sudo pacman -U maz-launcher-*.pkg.tar.zst` |
| Any (sandboxed) | `maz-launcher-…-x86_64.flatpak` | `flatpak install --user maz-launcher-*.flatpak` |
| Ubuntu & Snap | `maz-launcher_…_amd64.snap` | `sudo snap install --dangerous --classic maz-launcher_*.snap` |

The launcher uses the system's **webkit2gtk-4.1** (preinstalled on most desktops; otherwise e.g. `sudo apt install libwebkit2gtk-4.1-0`). Epic games run through the [Heroic Games Launcher](https://heroicgameslauncher.com), Roblox through [Sober](https://flathub.org/apps/org.vinegarhq.Sober).

### Updates

MAZ Launcher checks GitHub for new versions (can be turned off in **Settings → General**). Click **Update now** and it downloads the right file, checks its checksum and installs it. For the Linux packages it asks for your password and uses the package manager. New versions are always published on GitHub as well.

**Uninstall:** Windows: Settings → Apps → "MAZ Launcher" (or **Settings → General** in the launcher). macOS: drag the app to the bin. Linux: with your package manager. Your games are never touched.

## Usage

- **Play:** hover over a game and click ▶. Clicking the cover opens the game details.
- **Add your own game:** click **Add game** on the left → **Browse…** next to "Program" → pick the game's executable → **Add**.
- **Change the look:** **Settings → Design** → choose a layout (MAZ / Steam / Epic) and a color theme.
- **See all Epic games:** click **Connect** in the hint at the top of the library (or **Settings → Connections → Connect Epic account**). Three steps: sign in at Epic in your browser → press Ctrl + A and Ctrl + C on the page Epic shows (⌘ on Mac) → done. A progress bar shows how many games are loaded.
- **More accurate Steam list (optional):** **Settings → Connections** → enter a free Steam Web API key. Instructions are shown right next to the field.
- **Uninstall a game:** open the game → **Uninstall**.
- **A game won't start?** open the game → **Won't start? Help**. If a game crashes right after launch, the explanation appears automatically.
- **Roblox:** **Settings → Connections → Roblox** → paste the link of a Roblox game (e.g. `https://www.roblox.com/games/920587237/…`).
- **Scan a folder:** empty library or **My games** → **Scan a folder** → pick e.g. `D:\Games` → tick the games → **Add**.
- **Tags & collections:** open a game → type a tag at the bottom or click a collection. Create collections with **+** in the sidebar.
- **Minecraft mods:** click the **Minecraft** tab at the top (or open a Minecraft tile → **Mods & textures**) → choose where to install → search → **Install**. For CurseForge, add your free key once (the guide is right there).
- **Extensions:** **Settings → Extensions** → turn on "Allow extensions" → **Install extension…** → check the permissions → agree.
- **Wine/Proton (Linux):** open a Windows game you added → **Edit** → pick a version under "Wine / Proton".
- **Emulators:** **Settings → Connections → Emulators** → pick a preset → choose the emulator program and your ROM folders.
- **Edit game info:** open a game → **Edit info**.
- **Share a design:** **Settings → Design** → edit your design → **Save as file**.
- **Customize the safety checklist:** **Settings → My games** → add, edit or delete items. You can also add extra items per game.

### Errors the launcher explains for you

Game crashed right after launch (Steam, Epic or own games) · missing game file · drive or USB disk not connected · administrator rights required · access denied · wrong file selected · blocked by antivirus · blocked by Windows / Smart App Control · file in use · missing DLLs (Visual C++ / DirectX) · error **0xc000007b** · **0xc0000142** · crash right after launch · Steam or Epic not installed · Steam or Epic not responding · invalid Steam API key · WebView2 missing · installation failed.

## Building from source

**Requirements:** [Rust](https://rustup.rs) (stable, 1.85 or newer). No Node.js and no Dioxus CLI.

- **Windows / macOS:** nothing else.
- **Linux:** `sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev libssl-dev pkg-config` (or the same packages for your distribution).

```powershell
git clone https://github.com/MikeAdamRGB/MAZ-Lauchner.git
cd MAZ-Lauchner
cargo run                 # development build
cargo build --release     # release build: target/release/MAZ-Launcher(.exe)
```

Packages: `scripts/package-linux.sh` (AppImage, .deb, .rpm, .pkg.tar.zst, Flatpak, Snap) and `scripts/package-macos.sh` (.app, .dmg). GitHub Actions builds all of them for every release.

The compiled stylesheet (`assets/maz.css`) is checked into the repository and embedded into the EXE. You only need to rebuild it after changing Tailwind classes:

```powershell
.\scripts\build-css.ps1   # downloads the standalone Tailwind CLI once (no Node.js needed)
```

**Publishing a release:** push a tag (`git tag v1.0.0 && git push --tags`). GitHub Actions builds the EXE and attaches it to the release automatically.

## Tech stack

| Part | Technology |
|---|---|
| UI | [Dioxus 0.7](https://dioxuslabs.com) (desktop: WebView2 on Windows, WKWebView on macOS, WebKitGTK on Linux) |
| Components | Modeled on [rust-ui](https://github.com/rust-ui/ui) (Button, Card, Badge, Input, Switch, Dialog… with the same variants and classes) |
| Styling | Tailwind CSS v4, themes via CSS custom properties |
| Steam | Registry → `libraryfolders.vdf` → `appmanifest_*.acf` (custom VDF parser), artwork from the Steam cache, optional Steam Web API |
| Epic Games | Registry → launcher manifests (`*.item`), launcher catalog cache, optional Epic account for the full library |
| Roblox | Roblox player / Microsoft Store app / Sober detection, official `roblox-player:` and `roblox://experiences/start?placeId=` links |
| More stores | One `StoreLibrary` trait per store: registry / manifest files (GOG, EA, Ubisoft, Battle.net, Amazon), itch receipts, Xbox `MicrosoftGame.config` + AppX packages |
| Game info | Public Steam store `appdetails` + Steam Deck compatibility endpoints, Wikipedia REST API and Wikidata as second source; cached locally |
| Store libraries | Read-only SQLite access (`rusqlite`, bundled) to the local GOG Galaxy, Amazon Games and itch.io databases |
| Wine / Proton | Detects system Wine, Steam Proton, `compatibilitytools.d`, Lutris and Heroic runners; one prefix per game |
| Updates | GitHub Releases API, SHA256 verification, in-place replacement or package manager |
| Icons | Original icons extracted from the EXE through the Windows API (256 px) |
| HTTP | `ureq` with the system TLS (SChannel / Security.framework / OpenSSL) |

<details>
<summary><b>Why is the EXE so small?</b></summary>

- `opt-level = "z"`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = true`
- Dioxus with only the minimal feature set, no router or fullstack parts
- Windows functionality through direct API calls instead of large binding crates
- TLS through the SChannel stack built into Windows
- Only PNG/JPEG image decoding; CSS minified and embedded
- Optional UPX-packed `MAZ-Launcher-mini.exe`

</details>

<details>
<summary><b>Project structure</b></summary>

```
src/
├── main.rs        entry point, window, crash reporting
├── install.rs     built-in installer and uninstaller
├── steam.rs       Steam detection (+ Web API, cover lookup)
├── epic.rs        Epic Games detection (Windows/macOS launcher, Heroic on Linux)
├── epicacct.rs    optional Epic account for the full library
├── roblox.rs      Roblox detection, start and favorite experiences
├── stores.rs      GOG, EA, Ubisoft, Battle.net, Amazon, itch.io, Xbox
├── appx.rs        Microsoft Store / Xbox apps (Windows)
├── emulators.rs   emulator profiles, ROM scanner, folder scan
├── metadata.rs    automatic game info (cached)
├── playtime.rs    local playtime tracking
├── runners.rs     Wine / Proton versions on Linux
├── minecraft.rs   Minecraft Launcher + NoRisk Client profiles
├── mccontent.rs   Modrinth / CurseForge search and install
├── extensions/    extension system (manifest, protocol, processes, install)
├── notes.rs       changelog rendering for the update window
├── updater.rs     update check and in-app update
├── freegames.rs   free games on Epic and Steam
├── library.rs     merges everything into one library
├── launch.rs      game launching + crash detection
├── problem.rs     all error explanations (plain language)
├── icons.rs       original icons from EXEs
├── vdf.rs         parser for Valve's VDF format
├── winffi.rs      direct Windows API calls
├── config.rs / model.rs / sys.rs
└── ui/            interface (library in 3 styles, dialogs, settings, installer)
```

Settings: `%APPDATA%\MAZ Launcher` (Windows), `~/Library/Application Support/MAZ Launcher` (macOS), `~/.config/maz-launcher` (Linux). Image cache: `%LOCALAPPDATA%\MAZ Launcher`, `~/Library/Caches/MAZ Launcher`, `~/.cache/maz-launcher`.

</details>

## FAQ

**Does the launcher download games or bypass copy protection?**
No. Store games are always started through their official apps (Steam, Epic, GOG Galaxy, EA app …). The launcher never asks for passwords. Emulators only use ROMs you point it to – it never downloads ROMs or BIOS files.

**How often are there updates?**
New features are bundled into larger updates instead of many small ones. Quick fixes only ship on their own when they are urgent (security, crashes). See [docs/UPDATE-POLICY.md](docs/UPDATE-POLICY.md) and the [CHANGELOG](CHANGELOG.md).

**Why don't I see all of my Epic games?**
Epic only provides the list of owned games after signing in. Go to **Settings → Connections → Connect Epic account**, sign in in your browser, press Ctrl + A and Ctrl + C on the page Epic shows – done. Your whole Epic library appears.

**The window stays white or doesn't open.**
On Windows the launcher needs Microsoft WebView2, which is normally included with Windows 10 and 11. If it is missing, the launcher explains on startup how to install it. On Linux it needs webkit2gtk-4.1.

**Is my data sent anywhere?**
No. There is no MAZ account and no MAZ server. Network access is only used for cover art, the free-games list, the update check and – if you connect them – Steam and Epic. See the [privacy policy](docs/DATENSCHUTZ.md).

## Contributing

Bug reports, ideas and pull requests are welcome.

- 🐞 [Report a bug](https://github.com/MikeAdamRGB/MAZ-Lauchner/issues/new/choose)
- 💡 [Suggest an idea](https://github.com/MikeAdamRGB/MAZ-Lauchner/issues/new/choose)
- 🛠️ Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request

## License

Released under the [MIT License](LICENSE) © MikeAdamRGB.

Component design inspired by [rust-ui](https://github.com/rust-ui/ui) (MIT).
Steam is a trademark of Valve Corporation. Epic Games is a trademark of Epic Games, Inc. This project is independent and not affiliated with either company.

<div align="center">
<br>
<sub>Made with 💜 and 🦀 Rust</sub>
</div>
