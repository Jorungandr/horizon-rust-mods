#!/usr/bin/env python3
"""Fetch the release metadata for a Horizon Nightly and report the exact
horizon + horizon-mixins commits and SDK assets needed by a native mod's
[native] pins."""
import json, sys, urllib.request

TAG = "nightly-source/2026-11-13-9f42064"
UA = {"User-Agent": "x"}

def get(url):
    req = urllib.request.Request(url, headers=UA)
    return urllib.request.urlopen(req, timeout=60).read()

# 1. resolve the tag to a commit
ref = json.loads(get("https://api.github.com/repos/horizon-rust/horizon/git/ref/tags/" + TAG))
print("tag object type:", ref["object"]["type"])
sha = ref["object"]["sha"]
if ref["object"]["type"] == "tag":
    tag = json.loads(get(ref["object"]["url"]))
    sha = tag["object"]["sha"]
print("HORIZON_COMMIT =", sha)

# 2. list release assets whose name matches the nightly tag
rel_tag = "nightly-2026-11-13-9f42064"
try:
    rel = json.loads(get("https://api.github.com/repos/horizon-rust/horizon/releases/tags/" + rel_tag))
    print("release:", rel["tag_name"], "assets:", len(rel["assets"]))
    for a in rel["assets"]:
        print("  ASSET", a["name"], a["size"])
except Exception as e:
    print("release lookup failed:", e)

# 3. try the sdk.json / SHA256SUMS in the release assets
try:
    rel = rel
    for a in rel["assets"]:
        if a["name"] in ("sdk.json", "SHA256SUMS") or "sdk" in a["name"].lower():
            print("\n---", a["name"], "---")
            txt = get(a["browser_download_url"]).decode("utf-8", "replace")
            print(txt[:3000])
except Exception as e:
    print("sdk fetch failed:", e)
