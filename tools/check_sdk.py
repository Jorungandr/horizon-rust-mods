#!/usr/bin/env python3
import json, urllib.request
UA = {"User-Agent": "x"}
def get(u):
    return urllib.request.urlopen(urllib.request.Request(u, headers=UA), timeout=40).read()

build = "9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0"
for tag in ["sdk-" + build, "sdk-" + build[:8]]:
    try:
        d = json.loads(get("https://api.github.com/repos/horizon-rust/horizon/releases/tags/" + tag))
        print("FOUND", tag, "assets:", [a["name"] for a in d["assets"]])
    except Exception as e:
        print("MISS", tag, str(e)[:80])

# list all sdk-* tags via git refs (cheap)
try:
    refs = json.loads(get("https://api.github.com/repos/horizon-rust/horizon/git/matching-refs/tags/sdk-"))
    print("sdk tags count:", len(refs))
    for r in refs[-6:]:
        print("  ", r["ref"])
except Exception as e:
    print("refs failed:", e)
