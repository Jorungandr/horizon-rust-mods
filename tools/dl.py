"""Robust multi-part CDN downloader with SHA1 verify and resume, for E: drive.

usage:
  python dl.py core       # version json + client.jar + asset index
  python dl.py libs       # 114 libraries
  python dl.py assets     # ~490MB asset objects
All write only into E:\\Minecraft\\.minecraft and only ADD 26.3 files.
"""
import hashlib, json, os, ssl, sys, time, urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed

ROOT = r"E:\Minecraft\.minecraft"
VERSION = "26.3"
VER_JSON_URL = "https://piston-meta.mojang.com/v1/packages/9f67e10ed013deeaad20ac3c800652765d493408/26.3.json"
CTX = ssl.create_default_context()
UA = {"User-Agent": "Mozilla/5.0"}


def sha1(path):
    h = hashlib.sha1()
    with open(path, "rb") as f:
        for c in iter(lambda: f.read(1 << 20), b""):
            h.update(c)
    return h.hexdigest()


def get(url, tries=6, headers=None):
    last = None
    for i in range(1, tries + 1):
        try:
            req = urllib.request.Request(url, headers={**UA, **(headers or {})})
            with urllib.request.urlopen(req, timeout=45, context=CTX) as r:
                return r.read(), r.status
        except Exception as e:  # noqa
            last = e
            time.sleep(min(1.5 * i, 6))
    raise RuntimeError(f"get failed {url}: {last}")


def download_small(url, dest, expect=None, tries=6):
    os.makedirs(os.path.dirname(dest), exist_ok=True)
    if os.path.exists(dest) and expect and sha1(dest) == expect:
        return ("cached", os.path.getsize(dest))
    data, _ = get(url, tries=tries)
    if expect:
        got = hashlib.sha1(data).hexdigest()
        if got != expect:
            raise RuntimeError(f"sha1 {got} != {expect} for {dest}")
    with open(dest, "wb") as f:
        f.write(data)
    return ("ok", len(data))


def download_chunked(url, dest, expect, chunk=4 << 20, workers=8):
    """Range-parallel download into .parts then assemble; SHA1 verified."""
    os.makedirs(os.path.dirname(dest), exist_ok=True)
    if os.path.exists(dest) and sha1(dest) == expect:
        return ("cached", os.path.getsize(dest))
    # probe total size
    _, _data = None, None
    head, _ = get(url, headers={"Range": "bytes=0-0"})
    cr = None
    req = urllib.request.Request(url, headers={**UA, "Range": "bytes=0-0"})
    with urllib.request.urlopen(req, timeout=45, context=CTX) as r:
        cr = r.headers.get("Content-Range")  # bytes 0-0/TOTAL
    if not cr:
        raise RuntimeError("server ignored Range")
    total = int(cr.split("/")[-1])
    ranges = [(s, min(s + chunk - 1, total - 1)) for s in range(0, total, chunk)]
    partdir = dest + ".parts"
    os.makedirs(partdir, exist_ok=True)

    def fetch(idx_rng):
        idx, (s, e) = idx_rng
        part = os.path.join(partdir, f"{idx:04d}")
        need = e - s + 1
        if os.path.exists(part) and os.path.getsize(part) == need:
            return idx
            last = None
            for t in range(1, 7):
                try:
                    req = urllib.request.Request(url, headers={**UA, "Range": f"bytes={s}-{e}"})
                    with urllib.request.urlopen(req, timeout=45, context=CTX) as r, open(part, "wb") as f:
                        while True:
                            b = r.read(1 << 20)
                            if not b:
                                break
                            f.write(b)
                    if os.path.getsize(part) == need:
                        return idx
                    last = RuntimeError("short part")
                except Exception as exc:  # noqa
                    last = exc
                    time.sleep(min(t * 1.5, 6))
        raise RuntimeError(f"part {idx} failed: {last}")

    with ThreadPoolExecutor(max_workers=workers) as ex:
        futs = [ex.submit(fetch, x) for x in enumerate(ranges)]
        for fu in as_completed(futs):
            fu.result()
    h = hashlib.sha1()
    with open(dest, "wb") as out:
        for idx in range(len(ranges)):
            with open(os.path.join(partdir, f"{idx:04d}"), "rb") as f:
                b = f.read()
                h.update(b)
                out.write(b)
    if h.hexdigest() != expect:
        raise RuntimeError(f"assembled sha1 {h.hexdigest()} != {expect}")
    # cleanup parts
    for idx in range(len(ranges)):
        os.remove(os.path.join(partdir, f"{idx:04d}"))
    os.rmdir(partdir)
    return ("ok", total)


def load_meta():
    vdir = os.path.join(ROOT, "versions", VERSION)
    vjson = os.path.join(vdir, f"{VERSION}.json")
    if not os.path.exists(vjson):
        download_small(VER_JSON_URL, vjson)
    with open(vjson, "r", encoding="utf-8") as f:
        return json.load(f)


def main():
    mode = sys.argv[1]
    meta = load_meta()
    vdir = os.path.join(ROOT, "versions", VERSION)

    if mode == "core":
        c = meta["downloads"]["client"]
        url = f"https://launcher.mojang.com/v1/objects/{c['sha1']}/client.jar"
        print("client.jar ...", flush=True)
        print(download_chunked(url, os.path.join(vdir, f"{VERSION}.jar"), c["sha1"]), flush=True)
        a = meta["assetIndex"]
        print("asset index ...", flush=True)
        print(download_small(a["url"], os.path.join(ROOT, "assets", "indexes", f"{a['id']}.json"), a["sha1"]), flush=True)
        print("CORE DONE", flush=True)

    elif mode == "server":
        s = meta["downloads"]["server"]
        url = f"https://launcher.mojang.com/v1/objects/{s['sha1']}/server.jar"
        dest = os.path.join(ROOT, "server-26.3", "server.jar")
        print("server.jar ...", flush=True)
        print(download_chunked(url, dest, s["sha1"]), flush=True)
        print("SERVER DONE", flush=True)

    elif mode == "libs":
        libs = []
        for lib in meta["libraries"]:
            art = lib.get("downloads", {}).get("artifact")
            if art:
                libs.append((art["url"], os.path.join(ROOT, "libraries", art["path"]), art["sha1"], art["size"]))
        print(f"libs={len(libs)}", flush=True)
        ok = fail = 0

        def work(j):
            try:
                download_small(j[0], j[1], j[2])
                return True
            except Exception as e:  # noqa
                print(f"  LIB FAIL {os.path.basename(j[1])}: {e}", flush=True)
                return False

        with ThreadPoolExecutor(max_workers=10) as ex:
            for fu in as_completed([ex.submit(work, j) for j in libs]):
                if fu.result():
                    ok += 1
                else:
                    fail += 1
        print(f"LIBS DONE ok={ok} fail={fail}", flush=True)

    elif mode == "assets":
        a = meta["assetIndex"]
        idx = os.path.join(ROOT, "assets", "indexes", f"{a['id']}.json")
        with open(idx, encoding="utf-8") as f:
            objs = json.load(f)["objects"]
        jobs = [(f"https://launcher.mojang.com/v1/objects/{v['hash']}/{v['hash']}",
                 os.path.join(ROOT, "assets", "objects", v["hash"][:2], v["hash"]), v["hash"], v["size"])
                for v in objs.values()]
        print(f"assets={len(jobs)} total={sum(j[3] for j in jobs)/1e6:.1f}MB", flush=True)
        ok = fail = 0

        def work(j):
            try:
                download_small(j[0], j[1], j[2])
                return True
            except Exception as e:  # noqa
                print(f"  ASSET FAIL {j[2][:8]}: {e}", flush=True)
                return False

        with ThreadPoolExecutor(max_workers=14) as ex:
            futs = [ex.submit(work, j) for j in jobs]
            for i, fu in enumerate(as_completed(futs), 1):
                if fu.result():
                    ok += 1
                else:
                    fail += 1
                if i % 300 == 0:
                    print(f"  {i}/{len(jobs)} ok={ok} fail={fail}", flush=True)
        print(f"ASSETS DONE ok={ok} fail={fail}", flush=True)


if __name__ == "__main__":
    main()
