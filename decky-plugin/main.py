"""Open Game Launcher – Decky-Backend.

Liest nur die Spieleliste, die der Launcher selbst schreibt (decky-games.json).
Startet nichts, schreibt nichts, braucht kein Netzwerk und läuft ohne Root-Rechte.
"""

import json
import os
import re

import decky

FILE_NAME = "decky-games.json"
NEW_ID = "io.github.mikeadamrgb.OpenGameLauncher"
OLD_ID = "io.github.mikeadamrgb.MAZLauncher"
MAX_BYTES = 2 * 1024 * 1024
ID_RE = re.compile(r"^[A-Za-z0-9:._/-]{1,200}$")
CTRL_RE = re.compile(r"[\x00-\x1f\x7f]")


def _candidates():
    home = decky.DECKY_USER_HOME
    return [
        os.path.join(home, ".config", "open-game-launcher", FILE_NAME),
        os.path.join(home, ".var", "app", NEW_ID, "config", "open-game-launcher", FILE_NAME),
        os.path.join(home, ".var", "app", OLD_ID, "config", "open-game-launcher", FILE_NAME),
    ]


def _clean(text, limit):
    if not isinstance(text, str):
        return ""
    return CTRL_RE.sub("", text)[:limit]


def _valid_launcher(launcher):
    """Nur der Launcher selbst: Flatpak mit bekannter ID oder eine vorhandene Programmdatei."""
    if not isinstance(launcher, dict):
        return None
    exe = launcher.get("exe")
    prefix = launcher.get("prefix") or ""
    start_dir = launcher.get("start_dir") or ""
    if not isinstance(exe, str) or not isinstance(prefix, str) or not isinstance(start_dir, str):
        return None
    if exe == "flatpak":
        if prefix not in ("run " + NEW_ID, "run " + OLD_ID):
            return None
    else:
        if prefix != "" or not os.path.isabs(exe) or not os.path.isfile(exe):
            return None
        if os.path.basename(exe) not in ("maz-launcher", "open-game-launcher") and not exe.endswith(".AppImage"):
            return None
    # Keine Zeichen, die Steam als Teil eines Befehls deuten könnte
    for part in (exe, prefix, start_dir):
        if any(c in part for c in '"\'`$;&|<>\n\r'):
            return None
    return {"exe": exe, "start_dir": start_dir, "prefix": prefix}


def load():
    found = []
    for path in _candidates():
        try:
            st = os.stat(path)
        except OSError:
            continue
        # Nur normale Dateien des Benutzers, nicht zu groß
        if not os.path.isfile(path) or os.path.islink(path) or st.st_size > MAX_BYTES or st.st_uid != os.getuid():
            continue
        found.append((st.st_mtime, path))
    if not found:
        return {"ok": False, "reason": "missing"}
    found.sort(reverse=True)
    try:
        with open(found[0][1], "r", encoding="utf-8") as f:
            data = json.load(f)
    except (OSError, ValueError):
        return {"ok": False, "reason": "broken"}
    if not isinstance(data, dict) or data.get("format") != "og-decky" or data.get("version") != 1:
        return {"ok": False, "reason": "broken"}
    launcher = _valid_launcher(data.get("launcher"))
    if launcher is None:
        return {"ok": False, "reason": "launcher"}
    games = []
    for g in data.get("games") or []:
        if not isinstance(g, dict):
            continue
        gid = g.get("id")
        if not isinstance(gid, str) or not ID_RE.match(gid):
            continue
        games.append({
            "id": gid,
            "name": _clean(g.get("name"), 120) or gid,
            "installed": bool(g.get("installed")),
            "favorite": bool(g.get("favorite")),
            "last_played": int(g.get("last_played") or 0) if isinstance(g.get("last_played"), int) else 0,
            "minutes": int(g.get("minutes") or 0) if isinstance(g.get("minutes"), int) else 0,
            "source": _clean(g.get("source"), 40),
        })
        if len(games) >= 400:
            break
    return {"ok": True, "launcher": launcher, "games": games}


class Plugin:
    async def get_library(self):
        try:
            return load()
        except Exception as e:  # noqa: BLE001 – das Plugin darf nie abstürzen
            decky.logger.error("Spieleliste konnte nicht gelesen werden: %s", e)
            return {"ok": False, "reason": "broken"}

    async def _main(self):
        decky.logger.info("Open Game Launcher plugin loaded")

    async def _unload(self):
        pass

    async def _uninstall(self):
        pass
