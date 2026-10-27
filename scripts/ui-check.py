"""Prüft das Ergebnis des Oberflächen-Tests und vergleicht (wenn vorhanden) mit einem Referenzbild.

Aufruf: python3 scripts/ui-check.py <ui-test.json> <bildschirmfoto.png> <referenz.png>
"""
import json
import os
import sys

result, shot, baseline = sys.argv[1:4]
if not os.path.isfile(result):
    print("FEHLER: Kein Ergebnis – die Oberfläche ist nicht (rechtzeitig) gestartet.")
    sys.exit(1)
data = json.load(open(result, encoding="utf-8"))
print("Ergebnis:", data)
ok = True
if int(data.get("cards", 0)) < 8:
    print("FEHLER: Es werden weniger als 8 Spiele-Kacheln angezeigt.")
    ok = False
if int(data.get("alerts", 0)) > 0:
    print("FEHLER: Im Fenster steht eine Fehlermeldung.")
    ok = False

# Bildvergleich nur, wenn es ein Referenzbild gibt (Abweichung wird gemeldet, nicht als Fehler gewertet,
# weil Schriftglättung und Fenstergröße je nach Runner leicht schwanken)
if os.path.isfile(baseline) and os.path.isfile(shot):
    try:
        from PIL import Image, ImageChops

        a = Image.open(shot).convert("L").resize((480, 300))
        b = Image.open(baseline).convert("L").resize((480, 300))
        diff = ImageChops.difference(a, b)
        mean = sum(i * c for i, c in enumerate(diff.histogram())) / (480 * 300) / 255
        print(f"Abweichung zum Referenzbild: {mean:.1%}")
        if mean > 0.15:
            print("::warning::Das Bildschirmfoto weicht deutlich vom Referenzbild ab.")
    except Exception as e:  # noqa: BLE001
        print("Bildvergleich übersprungen:", e)
elif not os.path.isfile(shot):
    print("::warning::Kein Bildschirmfoto erstellt.")
sys.exit(0 if ok else 1)
