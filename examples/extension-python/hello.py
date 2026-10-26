#!/usr/bin/env python3
"""Mini-Erweiterung für den MAZ Launcher in Python (nur Standardbibliothek).

Liest pro Zeile eine JSON-Anfrage von stdin und antwortet pro Zeile auf stdout.
Endet, sobald der Launcher stdin schließt.
"""
import json
import sys


def handle(req):
    call = req.get("call")
    if call == "hello":
        return {"name": "Python Example", "version": "1.0.0", "kinds": ["library"]}
    if call == "games":
        return {"games": [{
            "id": "py-1",
            "name": "Python Test Game",
            "launch": {"uri": "https://www.python.org/"},
        }]}
    raise LookupError(call)


for line in sys.stdin:
    try:
        req = json.loads(line)
    except ValueError:
        continue
    try:
        reply = {"id": req.get("id"), "result": handle(req)}
    except LookupError as e:
        reply = {"id": req.get("id"), "error": {"code": "unsupported", "message": str(e)}}
    sys.stdout.write(json.dumps(reply) + "\n")
    sys.stdout.flush()
