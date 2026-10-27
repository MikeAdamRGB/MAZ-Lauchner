# Erweiterungen für den Open Game Launcher – Entwickler-Doku

Erweiterungen sind **eigene Programme**, die über eine sehr einfache Schnittstelle mit dem
Launcher reden: Der Launcher schreibt pro Zeile eine Anfrage (JSON) auf **stdin**, die
Erweiterung antwortet pro Zeile auf **stdout**. Jede Programmiersprache geht.

Beispiele im Projekt:

| Ordner | Inhalt |
|---|---|
| `examples/extension-hello/` | Rust: zwei Test-Spiele, Spiel-Infos und eine Aktion „Ordner öffnen“ |
| `examples/extension-python/` | Python (nur Standardbibliothek): ein Test-Spiel |
| `crates/og-extension-sdk/` | Rust-SDK mit allen Protokoll-Typen und einer fertigen Hauptschleife |

## Verfügbare Erweiterungen

Fertige Pakete (`.ogext`) hängen an jedem [Release](https://github.com/MikeAdamRGB/Open-Game-Launcher/releases/latest).
Installieren: **Einstellungen → Erweiterungen → Installieren** und die heruntergeladene Datei wählen.
Der Launcher zeigt vorher an, was die Erweiterung lesen darf, und fragt nach deiner Zustimmung.
Es gibt bewusst **keinen** Online-Katalog im Launcher.

| Erweiterung | System | Was sie macht | Rechte | Stand |
|---|---|---|---|---|
| **Humble App** (`Humble-App.ogext`, Quelltext `extensions/humble-app/`) | Windows | Liest die Spieleliste der Humble App (installierte und gekaufte Spiele) und startet Spiele über `humble://` | liest `%APPDATA%\Humble App`, kein Netzwerk | neu – mit echten Installationen noch wenig getestet |
| **Legacy Games** (`Legacy-Games.ogext`, Quelltext `extensions/legacy-games/`) | Windows | Findet installierte Spiele des Legacy Games Launchers (Windows-Programmliste) und gekaufte Spiele aus dessen Speicher | liest `%APPDATA%\legacy-games-launcher`, kein Netzwerk | neu – mit echten Installationen noch wenig getestet |

Beide lesen **keine** Anmeldedaten: Felder wie `token`, `auth`, `user`, `password` werden übersprungen.

Eigene Erweiterung veröffentlicht? Gern per Issue oder Pull-Request in diese Liste eintragen lassen
(Quelltext öffentlich, Rechte im Manifest so klein wie möglich).

## Grundregeln

1. **Kein fremder Code im Launcher.** Keine DLL/.so/.dylib, kein Skripting im Launcher-Prozess.
2. **Keine Passwörter, keine Tokens.** Erweiterungen bekommen nie Store-Anmeldungen, den
   Epic-Schlüssel oder die `config.json`.
3. **Keine Spiele-Downloads aus inoffiziellen Quellen, kein Umgehen von Kopierschutz.**
   Erweiterungen melden Spiele, liefern Infos und lösen erlaubte Aktionen aus – mehr nicht.
4. **Kein Online-Katalog.** Installiert wird nur aus einer Datei oder einem Ordner, den der
   Nutzer selbst auswählt.
5. **Standardmäßig aus.** Der Nutzer muss „Erweiterungen erlauben“ einschalten und jeder
   Erweiterung einzeln zustimmen.

## Ehrlicher Hinweis zur Sicherheit

Erweiterungen laufen in Version 1 **mit den Rechten des Nutzers**. `read_paths` und `network`
im Manifest sind eine **Erklärung** für den Nutzer und werden vom Launcher bei allen Antworten
geprüft – sie sind aber **keine echte Sperre** im Betriebssystem. Deshalb:

- Vor der Installation zeigt der Launcher Name, Autor, Version, alle Rechte und die
  SHA-256-Prüfsummen. Ohne Zustimmung keine Installation.
- **Hash-Pinning:** Die Prüfsumme über *alle* Dateien wird gespeichert. Ändert sich danach auch
  nur eine Datei, wird die Erweiterung ausgeschaltet, bis der Nutzer erneut zustimmt.
- Der Prozess startet mit **minimaler Umgebung** (fast keine Umgebungsvariablen) und dem
  Erweiterungsordner als Arbeitsordner.
- **Flatpak:** Dort läuft die Erweiterung ohnehin in der Sandbox der App. Nächster Schritt:
  `flatpak-spawn --sandbox` für eine eigene, noch strengere Sandbox pro Erweiterung.
- Fehler werden nur lokal angezeigt, nichts wird ins Internet geschickt.

## Ordner

| System | Ordner |
|---|---|
| Windows | `%APPDATA%\Open Game Launcher\extensions\<id>\` |
| macOS | `~/Library/Application Support/Open Game Launcher/extensions/<id>/` |
| Linux | `~/.config/open-game-launcher/extensions/<id>/` |

Der Ordnername muss genau der `id` aus dem Manifest entsprechen. Bis Version 1.7 hieß der Ordner
„MAZ Launcher“ bzw. `maz-launcher` – er wird beim ersten Start automatisch umbenannt.

## manifest.json

```json
{
  "id": "hello-example",
  "name": "Hello Extension",
  "version": "1.0.0",
  "author": "Dein Name",
  "description": "@description",
  "homepage": "https://example.com",
  "run": { "windows": "hello.exe", "linux": "hello", "macos": "hello" },
  "api_version": 1,
  "kinds": ["library", "metadata", "action"],
  "permissions": {
    "read_paths": ["~/Games", "%APPDATA%\\MyStore"],
    "network": false,
    "uri_schemes": ["mystore"]
  },
  "languages": ["de", "en"],
  "signature": ""
}
```

| Feld | Pflicht | Regeln |
|---|---|---|
| `id` | ja | 2–40 Zeichen, nur `a-z`, `0-9`, `-` |
| `name`, `author` | ja | max. 60 Zeichen |
| `version` | ja | SemVer, z. B. `1.2.3` oder `1.2.3-beta.1` |
| `description` | nein | max. 500 Zeichen |
| `homepage` | nein | nur `https://` |
| `run` | ja | Pfad **relativ** zum Erweiterungsordner, entweder ein Text für alle Systeme oder je System (`windows`, `macos`, `linux`). Kein `..`, kein absoluter Pfad, keine Symlinks nach außen. |
| `api_version` | ja | `1`. Andere Werte → Erweiterung wird mit Erklärung deaktiviert. |
| `kinds` | ja | `library`, `metadata`, `action` |
| `permissions.read_paths` | nein | Nur mit Platzhalter: `~/…`, `%APPDATA%`, `%LOCALAPPDATA%`, `%PROGRAMDATA%`, `%XDG_DATA_HOME%`, `%XDG_CONFIG_HOME%`. Max. 20. |
| `permissions.network` | nein | `true`, wenn die Erweiterung ins Internet geht |
| `permissions.uri_schemes` | nein | Zusätzliche Link-Arten (z. B. `mystore` für `mystore://…`). `file`, `javascript`, `data` u. ä. sind nie erlaubt. |
| `languages` | nein | Mitgelieferte Sprachen unter `lang/` |
| `signature` | nein | Vorbereitet für minisign/ed25519, in Version 1 nicht geprüft |

Unbekannte Felder sind ein Fehler (`deny_unknown_fields`).

### Übersetzungen

Texte, die mit `@` beginnen (Name, Beschreibung, Knopf-Beschriftungen, Meldungen), werden in
`lang/<sprache>.txt` nachgeschlagen – im selben Format wie die Launcher-Sprachdateien
(`schlüssel = Text`). Fehlt die Sprache, wird `lang/en.txt` genutzt, sonst der Schlüssel selbst.

## Protokoll (Version 1)

- Eine Anfrage pro Zeile, eine Antwort pro Zeile, UTF-8-JSON.
- Jede Anfrage hat eine `id`; die Antwort gibt sie zurück.
- Erfolg: `{"id": 7, "result": …}` · Fehler: `{"id": 7, "error": {"code": "…", "message": "…"}}`
- **Beenden:** Sobald stdin geschlossen wird (EOF), muss sich die Erweiterung sofort beenden.
- Nicht unterstützter Aufruf → Fehler mit `code: "unsupported"`.

### hello (immer zuerst)

```json
→ {"id":1,"call":"hello","api_version":1,"os":"windows","lang":"de","network":false}
← {"id":1,"result":{"name":"Hello Extension","version":"1.0.0","kinds":["library","action"]}}
```

### games / owned

`games` = installierte Spiele, `owned` (optional) = gekaufte, nicht installierte.

```json
→ {"id":2,"call":"games"}
← {"id":2,"result":{"games":[
    {"id":"witcher3","name":"The Witcher 3","install_dir":"D:\\Games\\Witcher3",
     "launch":{"exe":{"path":"D:\\Games\\Witcher3\\bin\\witcher3.exe","args":["-windowed"]}},
     "cover":"https://example.com/cover.webp","icon":"icons/witcher.png","size":50000000000}
  ]}}
```

Startarten (`launch`), genau eine:

| Art | Regeln |
|---|---|
| `{"uri": "…"}` | Nur Link-Arten der eingebauten Positivliste (z. B. `https`, `steam`) oder aus `uri_schemes`, denen der Nutzer zugestimmt hat |
| `{"exe": {"path": "…", "args": […]}}` | Datei muss existieren und **innerhalb von `install_dir`** liegen. Vor jedem Start kommt – wie bei eigenen Spielen – die Sicherheitsfrage. Max. 20 Argumente, keine Shell. |
| `{"appx": {"family": "…", "app": "…"}}` | Nur Windows, nur einfache Kennungen |

Spiel-IDs bekommen im Launcher automatisch das Präfix `ext:<id>:`. Erlaubt: `a-z A-Z 0-9 - _ .`,
max. 100 Zeichen. Bilder: `https://…` oder eine Datei im Erweiterungsordner, nur PNG/JPEG/WebP,
Dateien max. 5 MB.

### metadata

Wird nur gefragt, wenn die eingebauten Quellen (Steam-Store, Wikipedia) nichts gefunden haben.
Eigene Änderungen des Nutzers haben immer Vorrang.

```json
→ {"id":3,"call":"metadata","name":"Hello Game One","id":"ext:hello-example:hello-1"}
← {"id":3,"result":{"description":"…","genres":["Demo"],"developer":"OG","release":"2026","cover":"https://…/c.png"}}
```

Leeres Objekt `{}` = nichts gefunden.

### actions / run_action

```json
→ {"id":4,"call":"actions","game":{"id":"…","name":"…","source":"steam","install_dir":"…"}}
← {"id":4,"result":{"actions":[{"id":"open-folder","label":"@open_folder"}]}}

→ {"id":5,"call":"run_action","action":"open-folder","game":{…}}
← {"id":5,"result":{"type":"open_folder","path":"D:\\Games\\Witcher3\\mods"}}
```

Erlaubte Ergebnisse – alles andere wird blockiert:

| `type` | Bedeutung | Prüfung |
|---|---|---|
| `open_folder` | Ordner öffnen | muss in `read_paths` oder im Spielordner liegen |
| `open_url` | Link öffnen | wie bei `launch.uri` |
| `message` | Meldung anzeigen (`text`, max. 1000 Zeichen) | – |

Max. 8 Aktionen pro Erweiterung und Spiel.

## Grenzen

| Grenze | Wert |
|---|---|
| Zeit pro Aufruf | 10 Sekunden |
| Größe einer Antwort | 1 MB |
| Spiele pro Erweiterung | 5000 |
| Name / Beschreibung | 120 / 4000 Zeichen |
| Paketgröße | 200 MB, max. 2000 Dateien |

Antwortet eine Erweiterung nicht, schickt kaputtes JSON, zu große Antworten oder stürzt ab, wird
sie **einmal neu gestartet**. Passiert es wieder, wird sie bis zum nächsten Launcher-Start
deaktiviert und in den Einstellungen mit Erklärung angezeigt. Die Bibliothek lädt trotzdem.

Prozesse enden mit dem Launcher: unter Windows über ein Job-Objekt, unter Linux über
`PR_SET_PDEATHSIG`, unter macOS über das geschlossene stdin (siehe „Beenden“ oben).

## Fehlercodes (Vorschlag)

`unsupported`, `not_found`, `bad_request`, `internal`. Der Launcher zeigt `code` und `message` an.

## Paketieren als .ogext

Eine `.ogext`-Datei ist ein ZIP mit `manifest.json` **im Hauptordner** und allen Dateien.
Ältere Pakete mit der Endung `.mazext` funktionieren weiter.

```sh
cd my-extension
zip -r ../my-extension.ogext manifest.json hello hello.exe lang icons
```

Beim Entpacken prüft der Launcher: keine Pfade mit `..`, keine absoluten Pfade oder Laufwerke,
keine Symlinks, keine verschlüsselten Einträge, Größengrenzen (Schutz vor Zip-Slip und Zip-Bomben).

## Testen

1. Launcher → Einstellungen → Erweiterungen → „Erweiterungen erlauben“ einschalten.
2. „Aus Ordner …“ und den Ordner der Erweiterung wählen (z. B. `examples/extension-hello`
   nach dem Bauen – die fertige Datei `extension-hello` bzw. `.exe` neben `manifest.json` legen).
3. Rechte prüfen und zustimmen. Die Spiele erscheinen nach dem Neu-Laden in der Bibliothek.

Von Hand testen geht auch ohne Launcher:

```sh
printf '{"id":1,"call":"hello","api_version":1,"os":"linux","lang":"de"}\n{"id":2,"call":"games"}\n' | ./hello
```

Der Launcher selbst hat Tests für Manifest, Protokoll (Zeitüberschreitung, zu große Antwort,
kaputtes JSON, Absturz), Start-Prüfungen, Hash-Pinning, Zip-Slip und einen Test, der
`examples/extension-hello` baut, startet und seine Spiele lädt.
