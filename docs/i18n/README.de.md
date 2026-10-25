<div align="center">

<img src="../../assets/icon-256.png" width="128" alt="MAZ Launcher Logo">

# MAZ Launcher

**Alle deine Spiele. Ein Launcher.**

Steam, Epic Games, Roblox und deine eigenen Spiele in einer schnellen, schlanken und schönen App für **Windows, macOS und Linux**, geschrieben zu 100 % in Rust. Alles läuft lokal – kein Konto.

[![Neueste Version](https://img.shields.io/github/v/release/MikeAdamRGB/MAZ-Lauchner?label=Version&color=8b5cf6)](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/MikeAdamRGB/MAZ-Lauchner/total?label=Downloads&color=ec4899)](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases)
[![Build](https://github.com/MikeAdamRGB/MAZ-Lauchner/actions/workflows/build.yml/badge.svg)](https://github.com/MikeAdamRGB/MAZ-Lauchner/actions/workflows/build.yml)
![Windows](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?logo=windows&logoColor=white)
![Rust](https://img.shields.io/badge/100%25-Rust-DEA584?logo=rust&logoColor=white)
[![Lizenz: MIT](https://img.shields.io/badge/Lizenz-MIT-22c55e)](../../LICENSE)

[English](../../README.md) · **Deutsch** · [Français](README.fr.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português](README.pt-BR.md) · [Polski](README.pl.md) · [Türkçe](README.tr.md) · [Русский](README.ru.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [简体中文](README.zh-CN.md)

[**⬇️ Herunterladen**](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases/latest) · [**🌐 Webseite**](https://mikeadamrgb.github.io/MAZ-Lauchner/)

<br>

<img src="../screenshots/library-maz.webp" alt="MAZ Launcher Bibliothek im MAZ-Stil" width="900">

</div>

---

## Inhalt

- [Funktionen](#funktionen)
- [Screenshots](#screenshots)
- [Installation](#installation)
- [Benutzung](#benutzung)
- [Selbst bauen](#selbst-bauen)
- [Technik](#technik)
- [FAQ](#faq)
- [Mitmachen](#mitmachen)
- [Lizenz](#lizenz)

## Funktionen

| | |
|---|---|
| 🎮 **Steam** | Zeigt **alle** deine Steam-Spiele – installiert oder nicht – auf allen Festplatten, inklusive Spielzeit. Installieren und deinstallieren direkt im Launcher. |
| 🟦 **Epic Games** | Findet alle installierten Epic-Spiele automatisch. Einmal das Epic-Konto verbinden und die **komplette** Epic-Bibliothek erscheint – installieren oder deinstallieren mit einem Klick. Unter Linux über den [Heroic Games Launcher](https://heroicgameslauncher.com). |
| 🧱 **Roblox** | Erkennt und startet Roblox. Füg Links deiner Lieblings-Roblox-Spiele ein – jedes wird eine eigene Kachel, die direkt ins Spiel startet. Unter Linux über [Sober](https://flathub.org/apps/org.vinegarhq.Sober). |
| 🏪 **7 weitere Stores** | GOG (Windows: GOG Galaxy, Linux: Heroic), EA app, Ubisoft Connect, Battle.net, Amazon Games, itch.io und Xbox / Microsoft Store. Installierte Spiele werden automatisch gefunden und immer über das offizielle Programm gestartet. Fehlt ein Programm, erklärt der Launcher, wo es das gibt. Jeder Store lässt sich ausschalten. |
| 🕹️ **Emulatoren** | Profile für RetroArch, Dolphin, PCSX2, DuckStation, PPSSPP, mGBA, Snes9x oder jeden anderen Emulator. ROM-Ordner wählen – jede ROM wird ein eigenes Spiel. |
| 🗂️ **Bibliothek ordnen** | Suchen, sortieren (Name, zuletzt gespielt, Spielzeit, zuletzt hinzugefügt) und filtern (Store, installiert, Favoriten, Genre, Tags, Sammlungen). Eigene Tags und Sammlungen, Spielzeit-Messung, doppelte Spiele aus mehreren Stores werden zusammengefasst. |
| 📝 **Spiel-Infos** | Beschreibung, Genres, Entwickler und Erscheinungsdatum werden automatisch geladen (öffentliche Steam-Store-Daten, zwischengespeichert). Name, Cover, Hintergrund, Beschreibung und Genres selbst bearbeiten. |
| 🎁 **Gratis-Spiele** | Ein Tab mit allen Spielen, die es gerade bei Epic Games und Steam kostenlos gibt, mit Countdown, wie lange noch. „Zur Bibliothek hinzufügen“ öffnet den Store – ein Klick auf „Holen“ und das Spiel ist im MAZ Launcher. |
| 🚀 **Eigene Spiele** | Füge jedes Spiel hinzu: EXE, Verknüpfung oder Batch-Datei. Vor **jedem** Start fragt eine Checkliste: *„Hast du alles Wichtige deaktiviert?“* Bei **Nein** startet das Spiel nicht. |
| 🖼️ **Original-Icons & Cover** | Echte Spiel-Icons (direkt aus der EXE bzw. aus Steam) und Original-Cover. Für jedes Spiel kannst du ein eigenes Cover setzen. |
| 🎨 **3 Ansichten** | Sieht aus **wie die Steam-Bibliothek**, **wie Epic Games** oder im **eigenen MAZ-Stil**. |
| 🌈 **16 Designs + eigene** | 16 fertige Farbschemata, ein Design-Editor für eigene Designs mit Schrift, Abständen und Ansicht – teilbar als Code oder `.maztheme.json`-Datei –, eigene Akzentfarbe und eigenes Hintergrundbild. |
| 🎮 **Controller-Modus** | Optional: den ganzen Launcher mit dem Gamepad bedienen (Steuerkreuz/Stick, A, B, LB/RB). |
| 🌍 **9 Sprachen** | Deutsch, English, Español, Français, Italiano, Português, Polski, Türkçe und Русский – inklusive aller Fehlererklärungen. |
| 🛟 **Fehler einfach erklärt** | Startet ein Spiel nicht oder stürzt es direkt nach dem Start ab (Steam, Epic oder eigenes), erklärt der Launcher **Schritt für Schritt** in einfachen Worten, was los ist und wie man es behebt, mit Knöpfen, die direkt helfen (z. B. „Spieldateien überprüfen“, „Als Administrator starten“ oder „Visual C++ herunterladen“). Jedes Spiel hat außerdem einen Knopf „Startet nicht? Hilfe“. |
| 🔄 **Updates in der App** | Du bekommst Bescheid, wenn es eine neue Version gibt. „Jetzt updaten“ lädt sie, prüft die SHA256-Prüfsumme und installiert sie – ohne Umweg über GitHub. |
| 🔒 **Alles lokal** | Kein MAZ-Konto, kein Server, kein Tracking. Passwörter bleiben bei Steam, Epic und Roblox. |
| 🐧 **Alle Systeme** | Windows (x86_64, ARM64), macOS (Apple Silicon + Intel), Linux (x86_64, ARM64) als AppImage, .deb, .rpm, .pkg.tar.zst, Flatpak und Snap. |
| 📦 **Eingebauter Installer (Windows)** | Eine einzige EXE: Doppelklick öffnet ein Installationsmenü mit Desktop-Verknüpfung, Startmenü-Eintrag, „Jetzt öffnen“ und „App-Ordner öffnen“. Keine Admin-Rechte nötig. |
| 🪶 **Winzig** | Eine einzige, auf Größe optimierte EXE ohne Zusatzdateien. |

## Screenshots

<table>
  <tr>
    <td width="50%" align="center">
      <img src="../screenshots/library-maz.webp" alt="MAZ-Stil"><br>
      <sub><b>MAZ-Stil</b>: eigener Look mit „Weiterspielen“-Banner und allen Spielen als Raster</sub>
    </td>
    <td width="50%" align="center">
      <img src="../screenshots/installed.webp" alt="Installiert"><br>
      <sub><b>Installiert</b>: nur die Spiele, die auf deinem PC sind</sub>
    </td>
  </tr>
  <tr>
    <td width="50%" align="center">
      <img src="../screenshots/library-steam-style.webp" alt="Steam-Stil"><br>
      <sub><b>Steam-Stil</b>: Spieleliste links und alle Spiele als Raster</sub>
    </td>
    <td width="50%" align="center">
      <img src="../screenshots/details-steam-style.webp" alt="Details im Steam-Stil"><br>
      <sub><b>Details im Steam-Stil</b>: installieren, Spielzeit, Store-Seite und mehr</sub>
    </td>
  </tr>
  <tr>
    <td width="50%" align="center">
      <img src="../screenshots/library-epic-style.webp" alt="Epic-Games-Stil"><br>
      <sub><b>Epic-Games-Stil</b>: große Karten, Filter oben</sub>
    </td>
    <td width="50%" align="center">
      <img src="../screenshots/game-details.webp" alt="Spieldetails"><br>
      <sub><b>Spieldetails</b>: spielen oder installieren, Favorit, eigenes Cover, Hilfe und Deinstallieren</sub>
    </td>
  </tr>
  <tr>
    <td width="50%" align="center">
      <img src="../screenshots/free-games.webp" alt="Gratis-Spiele"><br>
      <sub><b>Gratis-Spiele</b>: was es gerade bei Epic und Steam gratis gibt, mit Countdown</sub>
    </td>
    <td width="50%" align="center">
      <img src="../screenshots/installed.webp" alt="Installiert"><br>
      <sub><b>Installiert</b>: nur die Spiele, die auf deinem Computer sind</sub>
    </td>
  </tr>
  <tr>
    <td width="50%" align="center">
      <img src="../screenshots/settings-design.webp" alt="Design"><br>
      <sub><b>Design</b>: 16 fertige Designs, eigene Designs und eigene Akzentfarbe</sub>
    </td>
    <td width="50%" align="center">
      <img src="../screenshots/settings-language.webp" alt="Sprachen"><br>
      <sub><b>Sprachen</b>: der ganze Launcher in 9 Sprachen</sub>
    </td>
  </tr>
</table>

## Installation

Alle Dateien gibt es unter [**Releases**](https://github.com/MikeAdamRGB/MAZ-Lauchner/releases/latest). In `SHA256SUMS.txt` steht zu jeder Datei eine Prüfsumme.

### Windows 10 / 11
1. **`MAZ-Launcher.exe`** herunterladen (Windows auf ARM: **`MAZ-Launcher-arm64.exe`**).
2. **Doppelklick** → **„Installieren“** → **„Jetzt öffnen“**.

> [!TIP]
> **„Der Computer wurde durch Windows geschützt“?** Das passiert bei neuen Programmen ohne bezahltes Zertifikat. Auf **„Weitere Informationen“** und dann **„Trotzdem ausführen“** klicken.

### macOS 11 oder neuer (Apple Silicon + Intel)
1. **`MAZ-Launcher-…-macOS.dmg`** herunterladen, öffnen und **MAZ Launcher** in **„Programme“** ziehen.
2. Beim ersten Mal: **Rechtsklick** auf die App → **„Öffnen“** → **„Öffnen“**. macOS fragt einmal, weil die App nicht aus dem App Store kommt.

### Linux (x86_64 und ARM64)

| Distribution | Datei | Installieren |
|---|---|---|
| Alle | `MAZ-Launcher-…-x86_64.AppImage` / `-aarch64.AppImage` | `chmod +x MAZ-Launcher-*.AppImage` und starten |
| Ubuntu, Debian, Mint, Pop!_OS | `maz-launcher_…_amd64.deb` / `_arm64.deb` | `sudo apt install ./maz-launcher_*.deb` |
| Fedora, openSUSE, Nobara | `maz-launcher-…-1.x86_64.rpm` / `.aarch64.rpm` | `sudo dnf install ./maz-launcher-*.rpm` |
| Arch, Manjaro, EndeavourOS, CachyOS | `maz-launcher-…-x86_64.pkg.tar.zst` | `sudo pacman -U maz-launcher-*.pkg.tar.zst` |
| Alle (Sandbox) | `maz-launcher-…-x86_64.flatpak` | `flatpak install --user maz-launcher-*.flatpak` |
| Ubuntu & Snap | `maz-launcher_…_amd64.snap` | `sudo snap install --dangerous --classic maz-launcher_*.snap` |

Der Launcher nutzt **webkit2gtk-4.1** vom System (auf den meisten Desktops schon da). Epic-Spiele laufen über den [Heroic Games Launcher](https://heroicgameslauncher.com), Roblox über [Sober](https://flathub.org/apps/org.vinegarhq.Sober).

### Updates
Der MAZ Launcher schaut bei GitHub nach neuen Versionen (abschaltbar unter **Einstellungen → Allgemein**). **„Jetzt updaten“** lädt die passende Datei, prüft die Prüfsumme und installiert sie. Bei Linux-Paketen fragt er nach deinem Passwort und nutzt den Paketmanager. Neue Versionen erscheinen weiterhin auch auf GitHub.

## Benutzung

- **Spielen:** Mit der Maus über ein Spiel fahren und auf ▶ klicken. Ein Klick auf das Cover öffnet die Details.
- **Eigenes Spiel hinzufügen:** Links auf **„Spiel hinzufügen“** → bei „Programm“ auf **„Durchsuchen…“** → die Start-Datei des Spiels wählen → **„Hinzufügen“**.
- **Aussehen ändern:** **Einstellungen → Design** → Ansicht (MAZ / Steam / Epic) und Farbschema wählen.
- **Alle Epic-Spiele sehen:** **Einstellungen → Verbindungen → Epic-Konto verbinden** und die 3 Schritte dort befolgen.
- **Genauere Steam-Liste (optional):** **Einstellungen → Verbindungen** → kostenlosen Steam-Web-API-Schlüssel eintragen. Die Anleitung steht direkt daneben.
- **Spiel deinstallieren:** Spiel öffnen → **Deinstallieren**.
- **Spiel startet nicht?** Spiel öffnen → **Startet nicht? Hilfe**. Stürzt ein Spiel direkt nach dem Start ab, erscheint die Erklärung automatisch.
- **Roblox:** **Einstellungen → Verbindungen → Roblox** → Link eines Roblox-Spiels einfügen (z. B. `https://www.roblox.com/games/920587237/…`).
- **Ordner durchsuchen:** leere Bibliothek oder **Meine Spiele** → **Ordner durchsuchen** → z. B. `D:\Spiele` wählen → Spiele anhaken → **Hinzufügen**.
- **Tags & Sammlungen:** Spiel öffnen → unten einen Tag eintippen oder eine Sammlung anklicken. Neue Sammlungen mit **+** in der Seitenleiste.
- **Emulatoren:** **Einstellungen → Verbindungen → Emulatoren** → Vorlage wählen → Emulator-Programm und ROM-Ordner auswählen.
- **Spiel-Infos ändern:** Spiel öffnen → **Infos bearbeiten**.
- **Design teilen:** **Einstellungen → Design** → eigenes Design bearbeiten → **Als Datei speichern**.
- **Sicherheitsfrage anpassen:** **Einstellungen → Eigene Spiele** → Punkte hinzufügen, ändern oder löschen. Pro Spiel kannst du zusätzliche Punkte festlegen.

### Diese Fehler erklärt der Launcher automatisch

Spiel stürzt direkt nach dem Start ab (Steam, Epic oder eigenes) · Spieldatei fehlt · Laufwerk oder USB-Platte nicht eingesteckt · Admin-Rechte nötig · Zugriff verweigert · falsche Datei gewählt · vom Virenschutz blockiert · von Windows / Smart App Control blockiert · Datei wird benutzt · fehlende DLLs (Visual C++ / DirectX) · Fehler **0xc000007b** · **0xc0000142** · Absturz direkt nach dem Start · Steam bzw. Epic nicht installiert · Steam/Epic reagiert nicht · falscher Steam-API-Schlüssel · WebView2 fehlt · Installation fehlgeschlagen.

## Selbst bauen

**Voraussetzungen:** [Rust](https://rustup.rs) (stable, ab 1.85) unter Windows. Sonst nichts: kein Node.js, kein Dioxus-CLI.

```powershell
git clone https://github.com/MikeAdamRGB/MAZ-Lauchner.git
cd MAZ-Lauchner
cargo run                 # Entwickeln
cargo build --release     # Fertige EXE: target\release\MAZ-Launcher.exe
.\scripts\build.ps1 -Mini # EXE nach dist\ + zusätzlich mit UPX gepackte Mini-Version
```

Das fertige CSS (`assets/maz.css`) liegt schon im Repo und wird in die EXE eingebettet. Nur wenn du Tailwind-Klassen änderst:

```powershell
.\scripts\build-css.ps1   # lädt einmalig die eigenständige Tailwind-CLI (kein Node.js)
```

**Release erstellen:** Tag pushen (`git tag v1.0.0 && git push --tags`). GitHub Actions baut die EXE und hängt sie automatisch an das Release.

## Technik

| Teil | Womit |
|---|---|
| Oberfläche | [Dioxus 0.7](https://dioxuslabs.com) (WebView2 / WKWebView / WebKitGTK) |
| Komponenten | im Stil von [rust-ui](https://github.com/rust-ui/ui) (Button, Card, Badge, Input, Switch, Dialog … mit gleichen Varianten und Klassen) |
| Styling | Tailwind CSS v4, Themes über CSS-Variablen |
| Steam | Registry → `libraryfolders.vdf` → `appmanifest_*.acf` (eigener VDF-Parser), Bilder aus dem Steam-Cache, optional Steam-Web-API |
| Epic Games | Launcher-Manifeste (`*.item`), Katalog-Cache, optional Epic-Konto; Linux: Heroic |
| Roblox | Roblox-Player bzw. Sober erkennen, offizielle `roblox://`-Links |
| Updates | GitHub-Releases, SHA256-Prüfung, Austausch der Datei oder Paketmanager |
| Icons | Original-Icons per Windows-API direkt aus der EXE (256 px) |
| HTTP | `ureq` mit Windows-eigenem TLS (SChannel), keine große TLS-Bibliothek |

<details>
<summary><b>Warum ist die EXE so klein?</b></summary>

- `opt-level = "z"`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = true`
- Dioxus nur mit minimalen Features, keine Router- oder Fullstack-Teile
- Windows-Funktionen über direkte API-Aufrufe statt großer Binding-Crates
- TLS über das in Windows eingebaute SChannel
- Bilder nur PNG/JPEG, CSS minifiziert und eingebettet
- Optional: mit UPX gepackte `MAZ-Launcher-mini.exe`

</details>

<details>
<summary><b>Projektstruktur</b></summary>

```
src/
├── main.rs        Start, Fenster, Absturz-Meldungen
├── install.rs     eingebauter Installer & Deinstallation
├── steam.rs       Steam-Erkennung (+ Web-API, Cover-Suche)
├── epic.rs        Epic-Games-Erkennung
├── library.rs     führt alles zur Bibliothek zusammen
├── launch.rs      Spiele starten + Absturz-Erkennung
├── problem.rs     alle Fehlererklärungen (einfache Sprache)
├── icons.rs       Original-Icons aus EXEs
├── vdf.rs         Parser für Valves VDF-Format
├── winffi.rs      direkte Windows-API-Aufrufe
├── config.rs / model.rs / sys.rs
└── ui/            Oberfläche (Bibliothek in 3 Stilen, Dialoge, Einstellungen, Installer)
```

Einstellungen liegen in `%APPDATA%\MAZ Launcher\config.json`, der Bilder-Cache in `%LOCALAPPDATA%\MAZ Launcher`.

</details>

## FAQ

**Lädt der Launcher Spiele herunter oder umgeht er Kopierschutz?**
Nein. Store-Spiele werden immer über das offizielle Programm gestartet (Steam, Epic, GOG Galaxy, EA app …). Es werden nie Passwörter abgefragt. Emulatoren nutzen nur ROMs, die du selbst auswählst – der Launcher lädt nie ROMs oder BIOS-Dateien herunter.

**Wie oft gibt es Updates?**
Neue Funktionen kommen gesammelt in größeren Updates statt vieler kleiner. Einzelne Fixes gibt es nur, wenn sie dringend sind (Sicherheit, Abstürze). Siehe [UPDATE-POLICY.md](../UPDATE-POLICY.md) und den [CHANGELOG](../../CHANGELOG.md).

**Warum sehe ich nicht alle meine Epic-Spiele?**
Epic gibt die Liste der gekauften Spiele nur nach einer Anmeldung heraus. Geh auf **Einstellungen → Verbindungen → Epic-Konto verbinden**, melde dich im Browser an und drück auf der Seite, die Epic dann zeigt, Strg + A und Strg + C – fertig. Deine ganze Epic-Bibliothek erscheint.

**Das Fenster bleibt weiß oder startet nicht.**
Der Launcher braucht Microsoft WebView2 (bei Windows 10/11 normalerweise dabei). Fehlt es, erklärt der Launcher beim Start, wie man es installiert.

**Werden meine Daten irgendwohin gesendet?**
Nein. Es gibt kein MAZ-Konto und keinen MAZ-Server. Internet wird nur für Cover-Bilder, die Gratis-Spiele-Liste, die Update-Prüfung und – falls verbunden – Steam und Epic genutzt. Siehe [Datenschutzerklärung](../DATENSCHUTZ.md).

## Mitmachen

Fehlermeldungen, Ideen und Pull Requests sind willkommen.

- 🐞 [Fehler melden](https://github.com/MikeAdamRGB/MAZ-Lauchner/issues/new/choose)
- 💡 [Idee vorschlagen](https://github.com/MikeAdamRGB/MAZ-Lauchner/issues/new/choose)
- 🛠️ Vor einem Pull Request bitte [CONTRIBUTING.md](../../CONTRIBUTING.md) lesen

## Lizenz

Veröffentlicht unter der [MIT-Lizenz](../../LICENSE) © MikeAdamRGB.

Komponenten-Design angelehnt an [rust-ui](https://github.com/rust-ui/ui) (MIT).
Steam ist eine Marke der Valve Corporation, Epic Games ist eine Marke von Epic Games, Inc. Dieses Projekt ist unabhängig und mit keinem der beiden Unternehmen verbunden.

<div align="center">
<br>
<sub>Gemacht mit 💜 und 🦀 Rust</sub>
</div>
