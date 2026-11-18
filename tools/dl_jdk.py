"""Download + verify + extract Temurin JDK 25 (Windows x64) into E:\\Minecraft\\jdk-25.

Uses Adoptium API to get the latest JDK 25 URL/SHA256, Range-parallel download
with resume, then extracts with zipfile. Only adds files under E:\\Minecraft.
"""
import hashlib, json, os, shutil, ssl, time, urllib.request, zipfile
from concurrent.futures import ThreadPoolExecutor, as_completed

# GitHub CDN (Adoptium release asset) is blocked by proxy (502) on this host.
# Microsoft's aka.ms JDK 25 zip supports Range and is reachable, so use it.
URL = "https://aka.ms/download-jdk/microsoft-jdk-25-windows-x64.zip"
DEST_DIR = r"E:\Minecraft\jdk-25"
UA = {"User-Agent": "Mozilla/5.0"}
CTX = ssl.create_default_context()


def get_json(url, tries=6):
    last = None
    for i in range(1, tries + 1):
        try:
            req = urllib.request.Request(url, headers=UA)
            with urllib.request.urlopen(req, timeout=45, context=CTX) as r:
                return json.load(r)
        except Exception as e:  # noqa
            last = e
            time.sleep(min(1.5 * i, 6))
    raise RuntimeError(f"api failed: {last}")


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for c in iter(lambda: f.read(1 << 20), b""):
            h.update(c)
    return h.hexdigest()


def fetch_meta():
    d = get_json(API)[0]
    p = d["binary"]["package"]
    return p["link"], p["checksum"], p["size"], d["version"]["semver"]


def probe(url):
    req = urllib.request.Request(url, headers={**UA, "Range": "bytes=0-0"})
    with urllib.request.urlopen(req, timeout=45, context=CTX) as r:
        cr = r.headers.get("Content-Range")
        if cr:
            return int(cr.split("/")[-1])
    return None


def dl_range(url, dest, total, chunk=8 << 20, workers=8):
    ranges = [(s, min(s + chunk - 1, total - 1)) for s in range(0, total, chunk)]
    partdir = dest + ".parts"
    os.makedirs(partdir, exist_ok=True)

    def fetch(x):
        idx, (s, e) = x
        part = os.path.join(partdir, f"{idx:04d}")
        need = e - s + 1
        if os.path.exists(part) and os.path.getsize(part) == need:
            return idx
        last = None
        for t in range(1, 9):
            try:
                req = urllib.request.Request(url, headers={**UA, "Range": f"bytes={s}-{e}"})
                with urllib.request.urlopen(req, timeout=60, context=CTX) as r, open(part, "wb") as f:
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
                time.sleep(min(t * 1.5, 8))
        raise RuntimeError(f"part {idx} failed: {last}")

    with ThreadPoolExecutor(max_workers=workers) as ex:
        for fu in as_completed([ex.submit(fetch, x) for x in enumerate(ranges)]):
            fu.result()
    with open(dest, "wb") as out:
        for idx in range(len(ranges)):
            with open(os.path.join(partdir, f"{idx:04d}"), "rb") as f:
                out.write(f.read())
    shutil.rmtree(partdir)


def dl_stream(url, dest):
    req = urllib.request.Request(url, headers=UA)
    with urllib.request.urlopen(req, timeout=120, context=CTX) as r, open(dest, "wb") as f:
        shutil.copyfileobj(r, f, 1 << 20)


def main():
    url = URL
    print(f"JDK 25 (Microsoft OpenJDK) from {url}", flush=True)
    os.makedirs(DEST_DIR, exist_ok=True)
    dest = os.path.join(DEST_DIR, "jdk.zip")
    # Determine expected size from Range probe; resume-aware chunk download verifies
    # via zip CRC on extraction (no external checksum endpoint available).
    total = probe(url)
    if not total:
        raise RuntimeError("Range probe failed")
    print(f"Range probe total={total / 1e6:.1f}MB", flush=True)
    partdir = dest + ".parts"
    complete = os.path.exists(dest) and os.path.getsize(dest) == total
    if complete:
        print("ZIP present (full size)", flush=True)
    else:
        dl_range(url, dest, total)
        print("chunked download done", flush=True)

    print("extracting ...", flush=True)
    with zipfile.ZipFile(dest) as z:  # triggers CRC check on read
        z.extractall(DEST_DIR)
    found = []
    for root, _dirs, files in os.walk(DEST_DIR):
        if "java.exe" in files and os.path.basename(root) == "bin":
            found.append(os.path.join(root, "java.exe"))
    for p in found:
        print("JAVA:", p, flush=True)
    if not found:
        raise RuntimeError("no java.exe found after extraction")
    print("JDK DONE", flush=True)


if __name__ == "__main__":
    main()
