# Changelog

Alle wichtigen Änderungen am MAZ Launcher. Jede Version hat bis zu vier Abschnitte:
**Neu**, **Verbessert**, **Behoben** und **Sicherheit**. Wie oft Updates kommen, steht in
[docs/UPDATE-POLICY.md](docs/UPDATE-POLICY.md).

Der Abschnitt der aktuellen Version wird automatisch als Text für das GitHub-Release
und für das Update-Fenster in der App verwendet.

## [1.7.0] – 2026-10-26 (Test)

### Neu
- **NoRisk Client:** Jedes Profil aus dem NoRisk Launcher wird eine eigene Kachel. „Spielen“ startet genau dieses Profil über den offiziellen Startbefehl des NoRisk Launchers – dein Microsoft-Konto bleibt dort.
- **Minecraft-Kachel:** Der offizielle Minecraft Launcher wird erkannt (Microsoft Store/Xbox-App, klassischer Launcher, macOS, Linux inkl. Flatpak).
- **Minecraft-Inhalte:** Neuer Tab „Minecraft“ – Mods, Texturenpakete, Shader und Datenpakete von **Modrinth** (ohne Schlüssel) und **CurseForge** (mit eigenem, kostenlosem Schlüssel und Schritt-für-Schritt-Anleitung) suchen und mit einem Klick installieren. Ziel: `.minecraft` des offiziellen Launchers oder ein NoRisk-Profil. Benötigte Zusatz-Mods kommen automatisch mit.
- **Fabric einrichten** mit einem Klick (wie der offizielle Fabric-Installer).
- **Erweiterungen** (Einstellungen → Erweiterungen, standardmäßig aus): kleine externe Programme können neue Spielquellen, Spiel-Infos und Knöpfe liefern. Mit Rechte-Dialog, Prüfsumme, Hash-Pinning, Beispielen in Rust und Python und ausführlicher Entwickler-Doku (docs/EXTENSIONS.md).

### Verbessert
- Test-Versionen werden auf GitHub als Vorabversion veröffentlicht und in der App nicht automatisch angeboten.

### Behoben
- **Xbox/Microsoft Store:** Statt des Spiels bzw. des Minecraft Launchers öffnete sich nur der Explorer. Die richtige App-Kennung wird jetzt aus der Paketbeschreibung gelesen.
- Einige Schritt-Anleitungen in der Fehlerhilfe (Store fehlt, Deinstallieren in anderen Stores, Wine/Proton fehlt) wurden nach dem ersten Schritt abgeschnitten.

### Sicherheit
- Minecraft-Downloads nur von den offiziellen Servern (Modrinth-CDN, CurseForge-CDN), jede Datei wird per SHA-512/SHA-1 geprüft; Dateinamen werden gesäubert, nur .jar/.zip, Größe begrenzt. CurseForge-Mods, deren Autor fremde Downloads verbietet, werden nicht geladen.
- Erweiterungen: kein fremder Code im Launcher, minimale Umgebung, 10-Sekunden- und 1-MB-Grenzen, strenge Prüfung aller Antworten (Pfade, Link-Arten, Bilder), Schutz gegen Zip-Slip, Prozesse enden garantiert mit dem Launcher.
- Der CurseForge-Schlüssel wird wie alle Schlüssel nur lokal gespeichert (unter Windows verschlüsselt, unter macOS/Linux nur für dich lesbar).

## [1.6.0] – 2026-10-25

### Neu
- **Komplette Store-Bibliotheken:** GOG, Amazon Games und itch.io zeigen jetzt auch gekaufte, nicht installierte Spiele (lokal aus deren Programmen gelesen). Ein Klick öffnet die Installation im offiziellen Programm.
- **PC-Anforderungen:** In den Spiel-Details stehen Mindest- und empfohlene Anforderungen für dein System.
- **Steam-Deck-Bewertung:** Verifiziert / Spielbar / Nicht unterstützt – direkt in den Details.
- **Zweite Quelle für Spiel-Infos:** Für Spiele ohne Steam-Seite (z. B. ROMs, GOG-, Xbox- oder eigene Spiele) kommen Beschreibung, Genres, Entwickler und Jahr aus Wikipedia/Wikidata – mit Quellenangabe. ROMs bekommen so auch ein Bild.
- **Wine/Proton pro Spiel (Linux):** Für jedes Windows-Spiel wählbar – System-Wine, Steam-Proton, GE-Proton, Lutris- und Heroic-Versionen. Proton bekommt einen eigenen Prefix pro Spiel.
- **Hinweis „Fehlen Epic-Spiele?“** oben in der Bibliothek, mit Knopf zum Verbinden.

### Verbessert
- **Epic-Konto verbinden:** deutlich schneller (Spiele werden parallel geladen) und übersichtlicher – drei Schritte mit Anzeige, wo du gerade bist, den richtigen Tastenkürzeln (auch ⌘ auf dem Mac) und einem Fortschrittsbalken „12 von 230 Spielen“.
- **Neues Logo:** schwarz mit weißer Schrift – in der App, als Programm-Symbol und auf der Webseite.
- **macOS:** echte Fenster-Knöpfe (Ampel) oben links, die eigene Titelleiste lässt Platz dafür.
- **Webseite:** OLED-Schwarz, Screenshots in Fenster-Rahmen mit Vergrößern, bessere Suchmaschinen-Daten (Beschreibung, Vorschaubild, Sprachen, Sitemap).
- Windows: Der eigene Roblox-Eintrag in der Seitenleiste ist weg – Roblox-Spiele stehen bei „Alle Spiele“.

### Sicherheit
- Einstellungen mit Schlüsseln/Anmeldung sind unter macOS und Linux nur noch für den eigenen Benutzer lesbar (0600).
- Linux-Updates werden in einem privaten Ordner geprüft und installiert statt in „Downloads“.
- Systemprogramme (xdg-open, Zwischenablage, rpm …) werden mit festem Pfad gestartet.
- Fremde Datenbanken (GOG, Amazon, itch.io) werden nur lesend und ohne ausführbare Inhalte geöffnet; Links und Kennungen daraus werden streng geprüft.
- Wine/Proton: Es startet nur eine Version, die der Launcher selbst gefunden hat.
- Epic: Die Zwischenablage wird höchstens 10 Minuten beobachtet; Antworten sind in der Größe begrenzt.
- Flatpak braucht keinen Zugriff mehr auf den Download-Ordner. Webseite mit Content-Security-Policy.

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
