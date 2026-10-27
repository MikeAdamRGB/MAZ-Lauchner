# Verbreitung über Paketmanager

Jedes Release hängt `OG-Paketquellen-<Version>.zip` an. Darin stehen fertig ausgefüllte
Paketquellen mit den echten Prüfsummen dieses Releases (erzeugt von `scripts/render-packaging.py`
aus den Vorlagen in `packaging/`). Einreichen geht nur mit eigenem Konto beim jeweiligen Dienst –
das macht die CI bewusst **nicht** automatisch.

| Dienst | Vorlage | Einreichen |
|---|---|---|
| **winget** (Windows) | `packaging/winget/` | Pull-Request an [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs) unter `manifests/m/MikeAdamRGB/OpenGameLauncher/<Version>/`, oder `wingetcreate submit` |
| **Scoop** (Windows) | `packaging/scoop/open-game-launcher.json` | Eigener „Bucket“ (ein GitHub-Repo mit der JSON-Datei) oder Pull-Request an [ScoopInstaller/Extras](https://github.com/ScoopInstaller/Extras). `checkver`/`autoupdate` sind eingetragen. Startet im portablen Modus (ohne Installer). |
| **Homebrew** (macOS) | `packaging/homebrew/open-game-launcher.rb` | Eigener Tap (`homebrew-tap`-Repo) oder Pull-Request an [Homebrew/homebrew-cask](https://github.com/Homebrew/homebrew-cask). Hinweis: Homebrew verlangt für den offiziellen Cask meist eine signierte App. |
| **AUR** (Arch Linux) | `packaging/aur/PKGBUILD` | Konto auf [aur.archlinux.org](https://aur.archlinux.org), SSH-Schlüssel hinterlegen, Repo `open-game-launcher-bin` anlegen, `makepkg --printsrcinfo > .SRCINFO`, pushen |
| **Flathub** (Linux) | `packaging/flathub/` | Siehe `packaging/flathub/README.md` (Pull-Request an flathub/flathub) |

Installation danach (sobald angenommen):

```text
winget install MikeAdamRGB.OpenGameLauncher
scoop install open-game-launcher          # nach "scoop bucket add …"
brew install --cask open-game-launcher    # nach "brew tap …"
yay -S open-game-launcher-bin
flatpak install flathub io.github.mikeadamrgb.MAZLauncher
```

Bei Installation über einen Paketmanager am besten auch über diesen aktualisieren
(`winget upgrade`, `scoop update`, `brew upgrade`, `yay -Syu`, `flatpak update`).
