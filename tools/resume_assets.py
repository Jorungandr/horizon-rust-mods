"""Resume only the missing/broken asset objects. Fast-fail with a failure list.

Only adds files under E:\\Minecraft\\.minecraft\\assets\\objects. Safe to re-run.
"""
import hashlib, json, os, ssl, time, urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed

LOG = os.path.join(os.path.dirname(__file__), "resume_assets.log")
def log(s):
    line = str(s)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(line + "\n")
    print(line, flush=True)

ROOT = r"E:\Minecraft\.minecraft"
IDX = os.path.join(ROOT, "assets", "indexes", "34.json")
CTX = ssl.create_default_context()
UA = {"User-Agent": "Mozilla/5.0"}


def sha1_file(path):
    h = hashlib.sha1()
    with open(path, "rb") as f:
        for c in iter(lambda: f.read(1 << 20), b""):
            h.update(c)
    return h.hexdigest()


def fetch_one(job, tries=4):
    h, size = job
    # Official CDN (launcher/piston-data mojang.com) is behind a broken proxy
    # (502) on this host; BMCLAPI mirrors the same objects and is reachable.
    url = f"https://bmclapi2.bangbang93.com/assets/{h[:2]}/{h}"
    dest = os.path.join(ROOT, "assets", "objects", h[:2], h)
    if os.path.exists(dest) and os.path.getsize(dest) == size and sha1_file(dest) == h:
        return ("cached", h, 0)
    last = None
    for t in range(1, tries + 1):
        try:
            req = urllib.request.Request(url, headers=UA)
            with urllib.request.urlopen(req, timeout=25, context=CTX) as r:
                data = r.read()
            if hashlib.sha1(data).hexdigest() != h:
                raise RuntimeError("sha1 mismatch")
            os.makedirs(os.path.dirname(dest), exist_ok=True)
            with open(dest, "wb") as f:
                f.write(data)
            return ("ok", h, len(data))
        except Exception as exc:  # noqa
            last = exc
            time.sleep(min(t * 1.0, 4))
    return ("FAIL", h, str(last)[:80])


def main():
    objs = json.load(open(IDX, encoding="utf-8"))["objects"]
    jobs = [(v["hash"], v["size"]) for v in objs.values()]
    missing = []
    for h, size in jobs:
        p = os.path.join(ROOT, "assets", "objects", h[:2], h)
        if not (os.path.exists(p) and os.path.getsize(p) == size):
            missing.append((h, size))
    print(f"total={len(jobs)} missing={len(missing)}", flush=True)
    ok = cached = 0
    failed = []
    with ThreadPoolExecutor(max_workers=10) as ex:
        futs = [ex.submit(fetch_one, j) for j in missing]
        for i, fu in enumerate(as_completed(futs), 1):
            status, h, info = fu.result()
            if status == "ok":
                ok += 1
            elif status == "cached":
                cached += 1
            else:
                failed.append((h, info))
            if i % 200 == 0:
                print(f"  {i}/{len(missing)} ok={ok} fail={len(failed)}", flush=True)
    print(f"DONE ok={ok} cached={cached} fail={len(failed)}", flush=True)
    if failed:
        print("FAILED LIST (hash, error):", flush=True)
        for h, info in failed:
            print(f"  {h} {info}", flush=True)


if __name__ == "__main__":
    main()
