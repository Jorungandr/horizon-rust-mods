#!/usr/bin/env python3
import json, urllib.request
UA = {"User-Agent": "x"}
def get(u):
    try:
        return json.loads(urllib.request.urlopen(urllib.request.Request(u, headers=UA), timeout=40).read())
    except Exception as e:
        return {"__err": str(e)[:70]}

for tag in ("v0.2.97", "v0.2.90", "v0.2.80", "nightly-2026-11-07"):
    r = get("https://api.github.com/repos/horizon-rust/horizon/releases/tags/" + tag)
    if "__err" in r:
        print(tag, "->", r["__err"]); continue
    print("==", tag, "==")
    for a in r["assets"]:
        print("   ", a["name"])
