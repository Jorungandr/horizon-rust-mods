"""Resumable, multi-source downloader for the pinned Horizon source tarball.

The proxy frequently drops long transfers. Strategy:
  - keep a growing .part file; on each attempt request Range bytes=<size>-
  - if the server honours Range (206) -> append the missing tail.
  - if it ignores Range (200, full body) -> slice off the already-have prefix.
  - rotate sources and retry on timeout/short reads until size matches, then
    verify gzip integrity. Never restarts from zero.
"""
import gzip
import os
import ssl
import time
import urllib.request

REV = "85b5bcca7c79bb470b90bdb5914b413aa66fe7ae"
EXPECT = 58827887
OUT = r"G:\WorkBuddyWork\2026-11-16-16-13-43\horizon-ports\horizon-mods-staging\native\target\horizon-src.tar.gz"
PART = OUT + ".part"
SOURCES = [
    f"https://codeload.github.com/horizon-rust/horizon/tar.gz/{REV}",
    f"https://gh-proxy.com/https://codeload.github.com/horizon-rust/horizon/tar.gz/{REV}",
]
CTX = ssl.create_default_context()


def have():
    return os.path.getsize(PART) if os.path.exists(PART) else 0


def attempt(url, start, si):
    """Download from `start`, write appended bytes. Return bytes written."""
    headers = {"User-Agent": "Mozilla/5.0"}
    if start > 0:
        headers["Range"] = f"bytes={start}-"
    req = urllib.request.Request(url, headers=headers)
    with urllib.request.urlopen(req, timeout=45, context=CTX) as r:
        cr = r.headers.get("Content-Range")
        cl = int(r.headers.get("Content-Length", 0))
        # If server honoured range: status 206, body is exactly the tail.
        tail_mode = (r.status == 206) or (cr is not None)
        written = 0
        mode = "ab" if tail_mode else "wb"   # 200 = full body: overwrite temp then slice
        tmp = PART + f".src{si}"
        with open(tmp, mode) as f:
            if tail_mode and start > 0 and mode == "wb":
                pass
            while True:
                try:
                    b = r.read(131072)
                except Exception:
                    break   # timeout mid-stream: keep what we got
                if not b:
                    break
                f.write(b)
                written += len(b)
        if tail_mode:
            return written, True
        # Full body received (maybe partial). tmp holds from byte 0.
        got = open(tmp, "rb").read()
        need = EXPECT - start
        if len(got) >= need:
            with open(PART, "ab") as pf:
                pf.write(got[start:EXPECT])
            return EXPECT - start, True
        # Server sent fewer than needed; write what we can from `start`.
        take = got[start:] if len(got) > start else b""
        if take:
            with open(PART, "ab") as pf:
                pf.write(take)
        return len(take), False


def main():
    os.makedirs(os.path.dirname(PART), exist_ok=True)
    si = 0
    stall = 0
    for attempt_no in range(1, 200):
        start = have()
        if start >= EXPECT:
            break
        url = SOURCES[si % len(SOURCES)]
        try:
            written, _ = attempt(url, start, si)
            if start == 0 and written == 0:
                stall += 1
            else:
                stall = 0
            print(f"try {attempt_no} src{si%len(SOURCES)} +{written/1e6:.1f}MB -> {have()/1e6:.1f}/{EXPECT/1e6:.1f}MB", flush=True)
        except Exception as e:
            stall += 1
            print(f"try {attempt_no} src{si%len(SOURCES)} FAIL {type(e).__name__} {str(e)[:40]} (have {have()/1e6:.1f}MB)", flush=True)
        if stall >= 6:
            si += 1
            stall = 0
            print(f"--- switching source to {si%len(SOURCES)} ---", flush=True)
        time.sleep(1)

    final = have()
    if final != EXPECT:
        print(f"INCOMPLETE {final}/{EXPECT}", flush=True)
        return
    os.replace(PART, OUT)
    with gzip.open(OUT, "rb") as g:
        n = 0
        while g.read(1 << 20):
            n += 1
            if n > 5000:
                break
    print(f"SOURCE READY size={final} gzip_ok bytes_read_chunks={n}", flush=True)


if __name__ == "__main__":
    main()
