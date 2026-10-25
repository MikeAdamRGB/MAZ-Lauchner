# Erweiterungen – Konzept (noch nicht eingebaut)

Dieses Dokument beschreibt, wie Erweiterungen für den MAZ Launcher später funktionieren
könnten. **Es gibt sie noch nicht.** Ziel ist, vorher festzulegen, was Erweiterungen dürfen
und was nicht, damit der Launcher sicher und lokal bleibt.

## Ziele

- Neue Spielquellen (weitere Stores, Emulator-Frontends) ohne Änderung am Launcher selbst.
- Neue Infos zu Spielen (z. B. andere Metadaten-Quellen).
- Eigene Aktionen in den Spiel-Details (z. B. „Mods-Ordner öffnen“).

## Grundregeln

1. **Kein fremder Code im Launcher-Prozess.** Erweiterungen laufen als eigenes Programm und
   reden über eine einfache Schnittstelle (JSON über stdin/stdout) mit dem Launcher.
2. **Nur was erlaubt ist.** Jede Erweiterung beschreibt in einer `manifest.json`, was sie
   braucht. Der Nutzer sieht das vor der Installation und muss zustimmen.
3. **Keine Passwörter.** Erweiterungen bekommen nie Zugriff auf Store-Konten oder gespeicherte
   Schlüssel des Launchers.
4. **Keine Downloads aus inoffiziellen Quellen, kein Umgehen von Kopierschutz** – genau wie
   beim Launcher selbst. Erweiterungen, die das tun, werden nicht unterstützt.
5. **Lokal zuerst.** Internet nur, wenn die Erweiterung es im Manifest angibt.

## Aufbau einer Erweiterung

```
%APPDATA%\MAZ Launcher\extensions\<id>\
    manifest.json
    <programm>            (z. B. extension.exe, extension.py)
```

### manifest.json

```json
{
  "id": "beispiel-store",
  "name": "Beispiel-Store",
  "version": "1.0.0",
  "author": "Name",
  "run": "extension.exe",
  "kinds": ["library"],
  "permissions": {
    "read_paths": ["%LOCALAPPDATA%\\BeispielStore"],
    "network": false
  }
}
```

`kinds` legt fest, was die Erweiterung liefern darf:

| Art | Aufgabe |
|---|---|
| `library` | Liste installierter Spiele (wie die eingebauten Stores) |
| `metadata` | Beschreibung, Genres, Bilder zu einem Spiel |
| `action` | Zusätzlicher Knopf in den Spiel-Details |

## Schnittstelle

Der Launcher startet das Programm der Erweiterung und schickt **eine** Anfrage pro Zeile:

```json
{"call": "games"}
```

Antwort (eine Zeile):

```json
{"games": [{"id": "bsp:123", "name": "Spiel", "install_dir": "C:\\Spiele\\Spiel", "launch": {"uri": "beispiel://launch/123"}}]}
```

Erlaubte Startarten entsprechen den eingebauten: `uri` (nur Schemata aus der Positivliste),
`exe` (Datei muss innerhalb von `install_dir` liegen) oder `appx`.

Weitere Aufrufe: `{"call": "metadata", "name": "..."}`, `{"call": "action", "game": "..."}`.
Zeitlimit pro Aufruf: 10 Sekunden. Antworten über 1 MB werden verworfen.

## Sicherheit

- Der Launcher prüft jede Antwort (Länge, Pfade, Link-Schemata) wie bei den eingebauten Stores.
- Erweiterungen lassen sich einzeln ausschalten; eine defekte Erweiterung bremst den Rest nicht.
- Die Installation läuft nur aus einem Ordner, den der Nutzer auswählt – es gibt keinen
  eingebauten Online-Katalog.

## Offene Fragen

- Signierte Erweiterungen?
- Sandbox unter Linux (Flatpak-Portal) und macOS.
- Wie Erweiterungen eigene Texte in allen 9 Sprachen mitbringen.

Die eingebauten Stores (`src/stores.rs`, Trait `StoreLibrary`) sind bewusst so gebaut, dass
eine Erweiterung später denselben Weg nehmen kann.
