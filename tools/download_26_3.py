"""Download a complete, PCL2-compatible Minecraft Java 26.3 installation into
E:\\Minecraft\\.minecraft, using Mojang's official CDN with SHA1 verification.

Safe: only ADDS version 26.3; never touches existing versions/assets/libraries.
Streaming download with retries; verifies sha1 where Mojang publishes one.
"""
import hashlib
import json
import os
import ssl
import sys
import time
import urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed

ROOT = r"E:\Minecraft\.minecraft"
VERSION = "26.3"
VER_JSON_URL = "https://piston-meta.mojang.com/v1/packages/9f67e10ed013deeaad20ac3c800652765d493408/26.3.json"
CTX = ssl.create_default_context()
UA = {"User-Agent": "Mozilla/5.0 (Horizon mod test resource fetch)"}


def sha1(path):
    h = hashlib.sha1()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def download(url, dest, expect_sha1=None, tries=4):
    os.makedirs(os.path.dirname(dest), exist_ok=True)
    if os.path.exists(dest) and expect_sha1 and sha1(dest) == expect_sha1:
        return ("cached", dest, os.path.getsize(dest))
    last = None
    for attempt in range(1, tries + 1):
        try:
            req = urllib.request.Request(url, headers=UA)
            tmp = dest + ".part"
            with urllib.request.urlopen(req, timeout=60, context=CTX) as r, open(tmp, "wb") as out:
                while True:
                    chunk = r.read(1 << 20)
                    if not chunk:
                        break
                    out.write(chunk)
            if expect_sha1:
                got = sha1(tmp)
                if got != expect_sha1:
                    raise RuntimeError(f"sha1 mismatch {got} != {expect_sha1}")
            os.replace(tmp, dest)
            return ("ok", dest, os.path.getsize(dest))
        except Exception as e:  # noqa: BLE001
            last = e
            print(f"  retry {attempt}/{tries} {os.path.basename(dest)}: {e}", flush=True)
            time.sleep(min(2 * attempt, 8))
    raise RuntimeError(f"failed {url}: {last}")


def main():
    mode = sys.argv[1] if len(sys.argv) > 1 else "core"

    # 1) version json
    vdir = os.path.join(ROOT, "versions", VERSION)
    vjson = os.path.join(vdir, f"{VERSION}.json")
    if not os.path.exists(vjson):
        download(VER_JSON_URL, vjson, tries=5)
    with open(vjson, "r", encoding="utf-8") as f:
        meta = json.load(f)

    if mode == "core":
        # client jar
        c = meta["downloads"]["client"]
        print("downloading client.jar ...", flush=True)
        print(download(c["url"], os.path.join(vdir, f"{VERSION}.jar"), c["sha1"]), flush=True)
        # asset index
        a = meta["assetIndex"]
        aidx = os.path.join(ROOT, "assets", "indexes", f"{a['id']}.json")
        print("downloading asset index ...", flush=True)
        print(download(a["url"], aidx, a["sha1"]), flush=True)
        print("CORE DONE", flush=True)
        return

    if mode == "assets":
        a = meta["assetIndex"]
        aidx = os.path.join(ROOT, "assets", "indexes", f"{a['id']}.json")
        with open(aidx, "r", encoding="utf-8") as f:
            index = json.load(f)
        objs = index["objects"]
        jobs = []
        for name, info in objs.items():
            h = info["hash"]
            size = info["size"]
            dest = os.path.join(ROOT, "assets", "objects", h[:2], h)
            url = f"https://piston-data.mojang.com/v1/objects/{h}"
            jobs.append((url, dest, h, size))
        print(f"assets: {len(jobs)} objects, total {sum(j[3] for j in jobs)/1e6:.1f} MB", flush=True)

        def work(job):
            url, dest, h, size = job
            try:
                download(url, dest, h, tries=4)
                return True
            except Exception as e:  # noqa: BLE001
                print(f"  asset FAILED {h[:8]} ({size}B): {e}", flush=True)
                return False

        ok = 0
        with ThreadPoolExecutor(max_workers=12) as ex:
            futs = [ex.submit(work, j) for j in jobs]
            for i, fu in enumerate(as_completed(futs), 1):
                if fu.result():
                    ok += 1
                if i % 200 == 0:
                    print(f"  progress {i}/{len(jobs)} ok={ok}", flush=True)
        print(f"ASSETS DONE ok={ok}/{len(jobs)}", flush=True)
        return

    if mode == "libraries":
        libs = []
        for lib in meta["libraries"]:
            dl = lib.get("downloads", {})
            art = dl.get("artifact")
            if art:
                libs.append((art["url"], os.path.join(ROOT, "libraries", art["path"]), art["sha1"], art["size"]))
        print(f"libraries: {len(libs)} files", flush=True)

        def work(job):
            url, dest, h, size = job
            try:
                download(url, dest, h, tries=4)
                return True
            except Exception as e:  # noqa: BLE001
                print(f"  lib FAILED {os.path.basename(dest)}: {e}", flush=True)
                return False

        ok = 0
        with ThreadPoolExecutor(max_workers=12) as ex:
            futs = [ex.submit(work, j) for j in libs]
            for i, fu in enumerate(as_completed(futs), 1):
                if fu.result():
                    ok += 1
        print(f"LIBRARIES DONE ok={ok}/{len(libs)}", flush=True)
        return

    print("unknown mode", mode)
    sys.exit(2)


if __name__ == "__main__":
    main()
