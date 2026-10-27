"""Füllt die Paketquellen (winget, Scoop, Homebrew, AUR, Flathub) mit Version und Prüfsummen.

Aufruf: python3 scripts/render-packaging.py <version> <SHA256SUMS.txt> <ausgabeordner>
Platzhalter: {VERSION}, {DATE}, {SHA_<dateiname>} (Dateiname darf {VERSION} enthalten).
"""
import datetime
import pathlib
import re
import sys

version, sums_file, out = sys.argv[1:4]
sums = {}
for line in open(sums_file, encoding="utf-8"):
    parts = line.split()
    if len(parts) == 2:
        sums[parts[1].lstrip("*")] = parts[0]

missing = set()


def fill(text: str) -> str:
    text = text.replace("{VERSION}", version).replace("{DATE}", datetime.date.today().isoformat())

    def sha(m):
        name = m.group(1)
        if name not in sums:
            missing.add(name)
            return "FEHLT"
        return sums[name]

    return re.sub(r"\{SHA_([^}]+)\}", sha, text)


root = pathlib.Path(__file__).resolve().parent.parent / "packaging"
for d in ["winget", "scoop", "homebrew", "aur", "flathub"]:
    for f in sorted((root / d).iterdir()):
        target = pathlib.Path(out) / d / f.name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(fill(f.read_text(encoding="utf-8")), encoding="utf-8")
if missing:
    print("Hinweis: keine Prüfsumme für", ", ".join(sorted(missing)))
print("Paketquellen geschrieben nach", out)
