# Open Game Launcher – Decky plugin (Steam Deck)

Shows the games from Open Game Launcher in the Quick Access menu (`…` button) and starts them
in Gaming Mode – with overlay, controller layout and focus like any Steam game.

## Install

1. Install [Decky Loader](https://decky.xyz) on the Steam Deck.
2. Download `Open-Game-Launcher-Decky.zip` from the latest
   [release](https://github.com/MikeAdamRGB/Open-Game-Launcher/releases/latest).
3. Decky → Settings → Developer → *Install plugin from ZIP file*.
4. Start Open Game Launcher once in Desktop Mode. It writes the game list
   (Settings → General → Connections → “Steam Deck plugin”, on by default).

## How it works / security

- The launcher writes `~/.config/open-game-launcher/decky-games.json` (Flatpak:
  `~/.var/app/io.github.mikeadamrgb.OpenGameLauncher/config/open-game-launcher/`). It only contains
  game IDs, names, playtime and favorites – no paths to games, no keys, no accounts.
- The plugin runs **without root** (`"flags": []`), reads only that file, does not write files,
  does not use the network and does not run programs itself.
- To start a game it creates **one** non-Steam shortcut “Open Game Launcher” and sets its launch
  options to `--play <id>`. Only IDs made of `A–Z a–z 0–9 : . _ / -` are accepted.

## Build

```sh
npm install --ignore-scripts
npm run build      # → dist/index.js
```
