"""Tests für das Decky-Backend (ohne Steam Deck): python3 -m unittest test_main"""

import json
import os
import sys
import tempfile
import types
import unittest

HOME = tempfile.mkdtemp()
fake = types.ModuleType("decky")
fake.DECKY_USER_HOME = HOME
fake.logger = types.SimpleNamespace(info=lambda *a: None, error=lambda *a: None)
sys.modules["decky"] = fake

import main  # noqa: E402


def write(data):
    d = os.path.join(HOME, ".config", "open-game-launcher")
    os.makedirs(d, exist_ok=True)
    with open(os.path.join(d, main.FILE_NAME), "w", encoding="utf-8") as f:
        f.write(data if isinstance(data, str) else json.dumps(data))


class Backend(unittest.TestCase):
    def test_missing_and_broken(self):
        p = os.path.join(HOME, ".config", "open-game-launcher", main.FILE_NAME)
        if os.path.exists(p):
            os.remove(p)
        self.assertEqual(main.load(), {"ok": False, "reason": "missing"})
        write("kaputt")
        self.assertEqual(main.load()["reason"], "broken")

    def test_valid_and_filtered(self):
        exe = sys.executable  # gibt es sicher, heißt aber nicht wie der Launcher
        write({"format": "og-decky", "version": 1, "launcher": {"exe": "flatpak", "start_dir": "", "prefix": "run " + main.NEW_ID},
               "games": [{"id": "steam:440", "name": "TF2\x07", "installed": True, "minutes": 5},
                         {"id": "x; rm -rf ~", "name": "böse"}, "quatsch"]})
        r = main.load()
        self.assertTrue(r["ok"])
        self.assertEqual([g["id"] for g in r["games"]], ["steam:440"])
        self.assertEqual(r["games"][0]["name"], "TF2")
        # Fremdes Programm oder fremde Flatpak-ID wird abgelehnt
        write({"format": "og-decky", "version": 1, "launcher": {"exe": exe, "start_dir": "", "prefix": ""}, "games": []})
        self.assertEqual(main.load()["reason"], "launcher")
        write({"format": "og-decky", "version": 1, "launcher": {"exe": "flatpak", "start_dir": "", "prefix": "run org.evil.App"}, "games": []})
        self.assertEqual(main.load()["reason"], "launcher")
        write({"format": "og-decky", "version": 1, "launcher": {"exe": "flatpak", "start_dir": "", "prefix": "run " + main.NEW_ID + ";x"}, "games": []})
        self.assertEqual(main.load()["reason"], "launcher")


if __name__ == "__main__":
    unittest.main()
