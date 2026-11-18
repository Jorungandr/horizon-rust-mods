"""Probe which Mojang CDN hosts are reachable and can serve the 26.3 client.jar."""
import urllib.request, ssl, time
CTX = ssl.create_default_context()
UA = {"User-Agent": "Mozilla/5.0"}
H = "e877b6a07acd633fb3bb475002175cec036e7b87"
hosts = [
    f"https://piston-data.mojang.com/v1/objects/{H}",
    f"https://launcher.mojang.com/v1/objects/{H}/client.jar",
    f"https://launcher.mojang.com/v1/objects/{H}",
    f"https://piston-data.mojang.com/v1/objects/{H}/client.jar",
]
for url in hosts:
    try:
        req = urllib.request.Request(url, headers={**UA, "Range": "bytes=0-1023"})
        t = time.time()
        with urllib.request.urlopen(req, timeout=25, context=CTX) as r:
            data = r.read()
            print(f"OK  status={r.status} got={len(data)}B in {time.time()-t:.1f}s  {url}")
    except Exception as e:
        print(f"FAIL {type(e).__name__}: {e}  {url}")
