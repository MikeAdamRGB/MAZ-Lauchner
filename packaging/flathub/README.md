# Flathub

Flathub baut Apps aus einem eigenen Repository `flathub/io.github.mikeadamrgb.MAZLauncher`.
Die App-ID bleibt `io.github.mikeadamrgb.MAZLauncher`, damit bestehende Installationen Updates bekommen.

Einreichen (einmalig, mit eigenem GitHub-Konto):

1. https://github.com/flathub/flathub forken, Branch `new-pr` als Basis nehmen.
2. `io.github.mikeadamrgb.MAZLauncher.yml` aus diesem Ordner (nach `scripts/render-packaging.py`
   im Release-Anhang `OG-Paketquellen-<Version>.zip`) hineinlegen und einen Pull-Request öffnen.
3. Auf die Prüfung durch das Flathub-Team warten (Fragen zu `--filesystem=host:ro` und
   `org.freedesktop.Flatpak` beantworten: ein Launcher muss Spiele finden und starten).

Das Manifest packt die fertige Programmdatei aus dem GitHub-Release (`.deb`) ein, statt Rust
neu zu bauen. Flathub akzeptiert das für Open-Source-Apps; ein Bau aus dem Quelltext wäre mit
`flatpak-cargo-generator` möglich, ist aber deutlich aufwendiger.
