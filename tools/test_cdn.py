"""Connectivity + small download test against Mojang's official CDN."""
import sys, urllib.request, ssl, time

ctx = ssl.create_default_context()
url = "https://piston-meta.mojang.com/v1/packages/9f67e10ed013deeaad20ac3c800652765d493408/26.3.json"
for attempt in range(1, 4):
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
        with urllib.request.urlopen(req, timeout=30, context=ctx) as r:
            data = r.read()
        print(f"OK attempt={attempt} bytes={len(data)}")
        print(data[:80].decode("utf-8", "replace"))
        sys.exit(0)
    except Exception as e:
        print(f"FAIL attempt={attempt}: {type(e).__name__}: {e}")
        time.sleep(2)
sys.exit(1)
