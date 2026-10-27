# Flathub

Flathub baut Apps aus einem eigenen Repository `flathub/io.github.mikeadamrgb.OpenGameLauncher`.
Seit 2.1 heißt die App-ID `io.github.mikeadamrgb.OpenGameLauncher` (vorher `io.github.mikeadamrgb.MAZLauncher`). Die neue App übernimmt beim ersten Start die Einstellungen der alten.

Einreichen (einmalig, mit eigenem GitHub-Konto):

1. https://github.com/flathub/flathub forken, Branch `new-pr` als Basis nehmen.
2. `io.github.mikeadamrgb.OpenGameLauncher.yml` aus diesem Ordner (nach `scripts/render-packaging.py`
   im Release-Anhang `OG-Paketquellen-<Version>.zip`) hineinlegen und einen Pull-Request öffnen.
3. Auf die Prüfung durch das Flathub-Team warten (Fragen zu `--filesystem=host:ro` und
   `org.freedesktop.Flatpak` beantworten: ein Launcher muss Spiele finden und starten).

Das Manifest packt die fertige Programmdatei aus dem GitHub-Release (`.deb`) ein, statt Rust
neu zu bauen. Flathub akzeptiert das für Open-Source-Apps; ein Bau aus dem Quelltext wäre mit
`flatpak-cargo-generator` möglich, ist aber deutlich aufwendiger.
