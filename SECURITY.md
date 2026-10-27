# Security / Sicherheit

## Reporting a vulnerability

Please **do not** open a public issue. Use GitHub's private reporting:
**Security → Report a vulnerability** in this repository. You get an answer within a few days.

Bitte Sicherheitslücken **nicht** öffentlich melden, sondern über **Security → Report a vulnerability**.

## Verify your download

Every release has `SHA256SUMS.txt` and a signed build provenance (Sigstore) that proves the file
was built by this repository's GitHub Actions from a specific commit:

```sh
sha256sum -c SHA256SUMS.txt --ignore-missing          # Linux
Get-FileHash .\Open-Game-Launcher.exe -Algorithm SHA256 # Windows (compare with SHA256SUMS.txt)
gh attestation verify Open-Game-Launcher.exe --repo MikeAdamRGB/Open-Game-Launcher
```

The in-app updater only downloads files from this repository's releases (HTTPS only), checks the
SHA-256 checksum before installing and never installs an older version.

## What the app may do – per download

| Download | Runs as | Notes |
|---|---|---|
| Windows `.exe` | your user, no admin (`asInvoker`) | Installs to `%LOCALAPPDATA%\Programs`. DLLs only from System32, Control Flow Guard and CET shadow stack enabled, static C runtime. Keys in the settings are encrypted with Windows DPAPI. |
| macOS `.dmg` / `.app.zip` | your user | Ad-hoc signed, not notarized (no paid Apple account). |
| AppImage, `.deb`, `.rpm`, Pacman | your user | Packages contain only files owned by root, no install scripts. Settings are `0600`, folders `0700`. |
| Flatpak | sandbox, with exceptions | Needs `--filesystem=host:ro` (find games, read only) and `org.freedesktop.Flatpak` (start games and stores outside the sandbox). That is required for a launcher, but it means the sandbox is **not** a security boundary. |
| Snap | classic confinement | Same reason: a launcher must start other programs. |
| Extensions (`.ogext`) | separate process, your user | Never loaded into the launcher. Only JSON over stdin/stdout, minimal environment, network only if declared and allowed. You see the permissions before installing. |
| Decky plugin | Steam Deck, **without root** | Only reads the game list the launcher writes; no network, no files written. |

## Built-in protections (short)

- No shell for starting games or helpers (no command chaining); Windows uninstall paths are checked.
- Web content in the window is plain text only; a Content Security Policy blocks loading or
  sending data to foreign servers, iframes, forms and plugins.
- Links: only known, safe schemes; Steam links with extra launch arguments are blocked.
- News, achievements and artwork: HTTPS only, with size limits; SteamGridDB images only from its own CDN.
- Backups never contain keys or accounts, and restoring never takes over programs to run before or
  after a game, or consent to extensions.
- CI: all GitHub Actions pinned to a commit, `cargo audit` on every build, pinned and
  checksum-verified packaging tools, npm without install scripts, Dependabot for updates.
