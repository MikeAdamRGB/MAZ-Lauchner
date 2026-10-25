# Update-Regel

Damit du nicht jeden zweiten Tag ein Update installieren musst, kommen Neuerungen
**gesammelt**.

## Feature-Updates (1.5.0, 1.6.0 …)

- Mehrere große **und** kleine Neuerungen werden zusammen veröffentlicht.
- Ein einzelnes neues Feature ist **kein** Grund für ein neues Update – es wartet auf das nächste Paket.
- Richtwert: höchstens etwa ein Feature-Update pro Monat.
- Jedes Update hat einen Eintrag in [CHANGELOG.md](../CHANGELOG.md) mit den Abschnitten
  **Neu**, **Verbessert**, **Behoben** (und bei Bedarf **Sicherheit**).

## Fehlerbehebungen (1.5.1, 1.5.2 …)

Ein Update mit nur einer Änderung gibt es **nur**, wenn es dringend ist:

- Sicherheitslücke
- Der Launcher startet nicht oder stürzt ab
- Spiele lassen sich für viele Nutzer nicht mehr starten
- Einstellungen oder Daten könnten verloren gehen

Alles andere (kleine Fehler, Texte, Optik) kommt ins nächste Feature-Update.

## Für Entwickler

- Auf `main` darf jederzeit gearbeitet werden. Ein Release entsteht **nur**, wenn die Version
  in `Cargo.toml` erhöht wird (die CI erstellt dann automatisch das GitHub-Release).
- Zum Testen der CI ohne Release: auf `main` ohne Versionssprung pushen oder einen Branch
  `ci/…` verwenden.
- Vor dem Versionssprung: Abschnitt `## [x.y.z] – Datum` in `CHANGELOG.md` anlegen. Dieser Text
  erscheint im GitHub-Release und im Update-Fenster der App.
