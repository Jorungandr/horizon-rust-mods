"""Download the pinned Horizon source tarball via concurrent ranged requests.

Range requests survive the flaky proxy far better than one long 825MB stream:
each chunk is small, independently retried, and SHA-verifiable as a whole.
"""
import hashlib
import os
import ssl
import sys
import time
import urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed

REV = "85b5bcca7c79bb470b90bdb5914b413aa66fe7ae"
URL = f"https://codeload.github.com/horizon-rust/horizon/tar.gz/{REV}"
OUT = r"G:\WorkBuddyWork\2026-11-16-16-13-43\horizon-ports\horizon-mods-staging\native\target\horizon-src.tar.gz"
PARTS = OUT + ".parts"
CTX = ssl.create_default_context()
UA = {"User-Agent": "Mozilla/5.0"}
CHUNK = 8 << 20          # 8 MiB per chunk
WORKERS = 12


def total_size():
    req = urllib.request.Request(URL, headers={**UA, "Range": "bytes=0-0"})
    with urllib.request.urlopen(req, timeout=30, context=CTX) as r:
        cr = r.headers.get("Content-Range")
        if cr and "/" in cr:
            return int(cr.rsplit("/", 1)[1])
        return int(r.headers.get("Content-Length", 0))


def get_range(s, e, idx):
    part = os.path.join(PARTS, f"{idx:05d}")
    need = e - s + 1
    if os.path.exists(part) and os.path.getsize(part) == need:
        return idx
    last = None
    for t in range(1, 9):
        try:
            req = urllib.request.Request(URL, headers={**UA, "Range": f"bytes={s}-{e}"})
            with urllib.request.urlopen(req, timeout=60, context=CTX) as r, open(part, "wb") as f:
                while True:
                    b = r.read(1 << 20)
                    if not b:
                        break
                    f.write(b)
            if os.path.getsize(part) == need:
                return idx
            last = RuntimeError("short part")
        except Exception as exc:
            last = exc
            time.sleep(min(t * 1.5, 8))
    raise RuntimeError(f"chunk {idx} failed: {last}")


def main():
    size = total_size()
    if not size:
        raise RuntimeError("could not determine size")
    n = (size + CHUNK - 1) // CHUNK
    print(f"source size={size/1e6:.1f}MB chunks={n} workers={WORKERS}", flush=True)
    os.makedirs(PARTS, exist_ok=True)
    ranges = []
    for i in range(n):
        s = i * CHUNK
        e = min(s + CHUNK, size) - 1
        ranges.append((s, e, i))

    done = 0
    with ThreadPoolExecutor(max_workers=WORKERS) as ex:
        futs = {ex.submit(get_range, s, e, i): i for s, e, i in ranges}
        for fut in as_completed(futs):
            fut.result()
            done += 1
            if done % 10 == 0 or done == n:
                print(f"chunks {done}/{n}", flush=True)

    with open(OUT, "wb") as out:
        for i in range(n):
            with open(os.path.join(PARTS, f"{i:05d}"), "rb") as f:
                out.write(f.read())
    got = os.path.getsize(OUT)
    print(f"SOURCE DONE size={got} match={got == size}", flush=True)
    import tarfile
    with tarfile.open(OUT, "r:gz") as tf:
        names = tf.getnames()
    print(f"TAR OK entries={len(names)} top={names[0].split('/')[0]}", flush=True)


if __name__ == "__main__":
    main()
