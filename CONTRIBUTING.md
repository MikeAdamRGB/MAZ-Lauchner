# Mitmachen

Danke, dass du helfen willst! 💜

1. Repo forken und klonen
2. `cargo run` zum Testen (Windows)
3. Tailwind-Klassen geändert? Die CI erzeugt `assets/maz.css` beim Bauen automatisch neu. Lokal: `./scripts/build-css.ps1`.
4. Neue Funktion? Eintrag in `CHANGELOG.md` (Abschnitt **Neu** / **Verbessert** / **Behoben**) – Updates werden gesammelt, siehe `docs/UPDATE-POLICY.md`.
5. `cargo fmt` und `cargo test` laufen lassen
6. Pull Request öffnen

**Regeln für Texte in der App:** Einfache, freundliche Sprache. Jede Fehlermeldung erklärt *was passiert ist*, *warum* und *Schritt für Schritt, wie man es behebt* (siehe `src/problem.rs`).
