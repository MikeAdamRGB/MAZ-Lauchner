"""Webseite: Google-Schriften beim Veröffentlichen selbst ausliefern (Datenschutz).

Besucher laden dann nichts mehr von Google – keine IP-Adresse geht an fremde Server.
Die Schriften (OFL-Lizenz) werden einmal in der CI heruntergeladen und neben die Seite gelegt.

Aufruf: python3 -I scripts/selfhost-fonts.py site
"""

import hashlib
import pathlib
import re
import sys
import urllib.request

site = pathlib.Path(sys.argv[1])
index = site / "index.html"
html = index.read_text(encoding="utf-8")

m = re.search(r'<link href="(https://fonts\.googleapis\.com/css2\?[^"]+)" rel="stylesheet">', html)
if not m:
    print("Keine Google-Schriften gefunden – nichts zu tun.")
    sys.exit(0)

UA = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0 Safari/537.36"


def get(url):
    if not url.startswith(("https://fonts.googleapis.com/", "https://fonts.gstatic.com/")):
        raise SystemExit("Unerwartete Adresse: " + url)
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    with urllib.request.urlopen(req, timeout=30) as r:
        return r.read(20 * 1024 * 1024)


css = get(m.group(1).replace("&amp;", "&")).decode("utf-8")
fonts = site / "fonts"
fonts.mkdir(exist_ok=True)


def local(match):
    url = match.group(1)
    data = get(url)
    name = hashlib.sha256(data).hexdigest()[:16] + ".woff2"
    (fonts / name).write_bytes(data)
    return "url(fonts/" + name + ")"


css = re.sub(r"url\((https://fonts\.gstatic\.com/[^)]+\.woff2)\)", local, css)
if "https://" in css:
    raise SystemExit("Es sind noch fremde Adressen in den Schriften.")
(site / "fonts.css").write_text(css, encoding="utf-8")

html = html.replace('<link rel="preconnect" href="https://fonts.googleapis.com">\n', "")
html = html.replace('<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>\n', "")
html = html.replace(m.group(0), '<link href="fonts.css" rel="stylesheet">')
html = html.replace(" https://fonts.googleapis.com", "").replace("font-src https://fonts.gstatic.com", "font-src 'self'")
if "fonts.googleapis" in html or "fonts.gstatic" in html:
    raise SystemExit("Google-Adressen sind noch in index.html.")
index.write_text(html, encoding="utf-8")
print("Schriften lokal:", len(list(fonts.iterdir())), "Dateien")
