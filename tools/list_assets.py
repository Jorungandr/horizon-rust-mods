#!/usr/bin/env python3
import json, urllib.request, sys
UA = {"User-Agent": "x"}
def get(u):
    return json.loads(urllib.request.urlopen(urllib.request.Request(u, headers=UA), timeout=60).read())

for tag in ("nightly", "nightly-2026-11-08"):
    rel = get("https://api.github.com/repos/horizon-rust/horizon/releases/tags/" + tag)
    print("=====", rel["tag_name"], "published:", rel["published_at"], "=====")
    for a in rel["assets"]:
        print("  ", a["name"], a["size"], "->", a["browser_download_url"])
    print("  BODY (first 800):", (rel.get("body") or "")[:800].replace("\n", " "))
