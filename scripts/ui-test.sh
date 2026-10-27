#!/usr/bin/env bash
# Oberflächen-Test (Linux/macOS): Launcher mit --ui-test starten, auf das Ergebnis warten,
# Bildschirmfoto machen. Aufruf: scripts/ui-test.sh <programm> <name>
set -u
exe="$1"; name="$2"; out="ui-$name"
rm -rf "$out"; mkdir -p "$out"
"$exe" --ui-test "$out" > "ui-test-$name.log" 2>&1 &
pid=$!
for _ in $(seq 1 120); do
  [ -s "$out/ui-test.json" ] && break
  sleep 1
done
sleep 2   # Bilder und Übergänge fertig zeichnen lassen
if [ "$(uname)" = Darwin ]; then
  screencapture -x "ui-$name.png" || true
else
  import -window root "ui-$name.png" 2>/dev/null || xwd -root -silent | convert xwd:- "ui-$name.png" || true
fi
kill "$pid" 2>/dev/null; sleep 1; kill -9 "$pid" 2>/dev/null
python3 scripts/ui-check.py "$out/ui-test.json" "ui-$name.png" "tests/ui/baseline-$name.png" 2>&1 | tee -a "ui-test-$name.log"
exit "${PIPESTATUS[0]}"
