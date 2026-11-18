"""Simplest possible: stream the tarball with a large per-read timeout.
Writes directly to .part; on failure, resume via Range if the server supports it,
otherwise restart. Verifies exact size + gzip."""
import gzip
import os
import socket
import ssl
import time
import urllib.request

REV = "85b5bcca7c79bb470b90bdb5914b413aa66fe7ae"
EXPECT = 58827887
OUT = r"G:\WorkBuddyWork\2026-11-16-16-13-43\horizon-ports\horizon-mods-staging\native\target\horizon-src.tar.gz"
PART = OUT + ".part"
URLS = [
    f"https://codeload.github.com/horizon-rust/horizon/tar.gz/{REV}",
    f"https://gh-proxy.com/https://codeload.github.com/horizon-rust/horizon/tar.gz/{REV}",
]
CTX = ssl.create_default_context()


def try_once(url, start):
    """Return total bytes in .part after this attempt."""
    headers = {"User-Agent": "Mozilla/5.0"}
    if start:
        headers["Range"] = f"bytes={start}-"
    req = urllib.request.Request(url, headers=headers)
    with urllib.request.urlopen(req, timeout=120, context=CTX) as r:
        range_ok = r.status == 206
        # Full-body responses must overwrite .part from zero.
        target = open(PART, "ab" if range_ok else "wb")
        cur = start if range_ok else 0
        try:
            while cur < EXPECT:
                try:
                    b = r.read(65536)
                except (socket.timeout, TimeoutError, OSError):
                    break
                if not b:
                    break
                # trim overrun beyond EXPECT
                room = EXPECT - cur
                if len(b) > room:
                    b = b[:room]
                target.write(b)
                cur += len(b)
        finally:
            target.close()
    return cur


def main():
    for rnd in range(30):
        start = os.path.getsize(PART) if os.path.exists(PART) else 0
        if start >= EXPECT:
            break
        url = URLS[rnd % len(URLS)]
        t = time.time()
        try:
            cur = try_once(url, start)
            print(f"round {rnd} src{rnd%len(URLS)} {start/1e6:.1f}->{cur/1e6:.1f}MB ({time.time()-t:.0f}s)", flush=True)
        except Exception as e:
            print(f"round {rnd} src{rnd%len(URLS)} FAIL {type(e).__name__} {str(e)[:50]} (have {start/1e6:.1f}MB)", flush=True)
        time.sleep(2)

    final = os.path.getsize(PART) if os.path.exists(PART) else 0
    if final != EXPECT:
        print(f"INCOMPLETE {final}/{EXPECT}", flush=True)
        return
    os.replace(PART, OUT)
    with gzip.open(OUT, "rb") as g:
        total = sum(len(g.read(1 << 20)) for _ in range(6000))
    print(f"SOURCE READY size={final} gzip_uncompressed~{total/1e6:.0f}MB", flush=True)


if __name__ == "__main__":
    main()
