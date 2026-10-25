# Changelog

Alle wichtigen Änderungen am MAZ Launcher. Jede Version hat bis zu vier Abschnitte:
**Neu**, **Verbessert**, **Behoben** und **Sicherheit**. Wie oft Updates kommen, steht in
[docs/UPDATE-POLICY.md](docs/UPDATE-POLICY.md).

Der Abschnitt der aktuellen Version wird automatisch als Text für das GitHub-Release
und für das Update-Fenster in der App verwendet.

## [1.5.0] – 2026-10-25

### Neu
- **Sieben weitere Stores:** GOG (Windows: GOG Galaxy, Linux: Heroic), EA app, Ubisoft Connect, Battle.net, Amazon Games, itch.io und Xbox / Microsoft Store. Installierte Spiele werden automatisch gefunden und immer über das offizielle Programm gestartet. Fehlt ein Programm, erklärt der Launcher, wo es das gibt.
- **Emulatoren:** Profile für RetroArch, Dolphin, PCSX2, DuckStation, PPSSPP, mGBA, Snes9x oder jeden anderen Emulator. ROM-Ordner wählen – jede ROM wird ein eigenes Spiel.
- **Ordner durchsuchen:** „D:\Spiele“ auswählen und alle gefundenen Spiele mit einem Klick hinzufügen.
- **Tags und Sammlungen:** Spiele selbst ordnen (z. B. „Mit Freunden“, „Durchgespielt“) – in der Seitenleiste filterbar.
- **Spielzeit:** Der Launcher misst, wie lange du spielst (nur lokal). Sortieren nach Spielzeit und „Zuletzt hinzugefügt“.
- **Spiel-Infos:** Beschreibung, Genres, Entwickler und Erscheinungsdatum werden automatisch geladen (öffentliche Steam-Store-Daten, zwischengespeichert). Genre-Filter in der Bibliothek.
- **Infos bearbeiten:** Name, Cover, Hintergrundbild, Beschreibung und Genres für jedes Spiel selbst ändern.
- **Doppelte Spiele:** Hast du ein Spiel in mehreren Stores, erscheint es nur einmal – die anderen Versionen startest du in den Details.
- **Design-Dateien:** Designs als `.maztheme.json` speichern und teilen – jetzt mit Schrift, Abständen und Ansicht.
- **Controller-Modus** (Einstellungen → Allgemein): Bedienung mit Gamepad.

### Verbessert
- Update-Fenster zeigt die Neuerungen übersichtlich nach Abschnitten.
- Neue Update-Regel: Mehrere Neuerungen kommen gesammelt in einem Update statt vieler kleiner Updates.
- `Esc` schließt Fenster.

### Behoben
- **Roblox:** Ein Klick auf die Roblox-Kachel öffnet jetzt die Roblox-App statt der Webseite – auch bei der Microsoft-Store-Version von Roblox.

## [1.4.3]

### Neu
- Linux: jetzt auch als **Flatpak**.

## [1.4.0]

### Neu
- **Linux und macOS:** Steam (auch Flatpak/Snap-Steam), Epic (macOS: Epic Games Launcher, Linux: Heroic), eigene Spiele (Linux: auch .exe über Wine).
- **Roblox:** Roblox wird erkannt und gestartet. Eigene Roblox-Spiele als Kacheln (Linux: über Sober).
- **Updates in der App:** Hinweis bei neuer Version und Knopf „Jetzt updaten“, Download wird per SHA256 geprüft.
- **ARM64** für Windows und Linux.

### Verbessert
- **Alles lokal:** Konto, Freunde und Chat wurden entfernt. Kein MAZ-Server mehr, kein Konto nötig.

## [1.3.2]

### Sicherheit
- Schutz gegen untergeschobene DLLs/Programme, Bestätigung vor dem Deinstallieren, strengere Prüfung von Links, CI mit minimalen Rechten.

## [1.3.1]

### Sicherheit
- Deinstallation löscht nur eigene Dateien, Links werden geprüft, gespeicherte Schlüssel verschlüsselt, lokale Dateien nur noch als Bilder.

## [1.3.0]

### Neu
- Spiele deinstallieren, komplette Epic-Bibliothek, Absturzhilfe für Steam- und Epic-Spiele.

## [1.2.0]

### Neu
- Gratis-Spiele, nicht installierte Spiele installieren.

## [1.1.0]

### Neu
- 9 Sprachen, Design-Editor, 16 Designs, Hintergrundbild.

## [1.0.0]

### Neu
- Erste Version: Steam, Epic und eigene Spiele, drei Ansichten, Installer.
