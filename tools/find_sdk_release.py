#!/usr/bin/env python3
import json, urllib.request
UA = {"User-Agent": "x"}
def get(u):
    return json.loads(urllib.request.urlopen(urllib.request.Request(u, headers=UA), timeout=60).read())

# scan ALL releases (multiple pages) for sdk tags/assets
for page in (1, 2):
    rels = get("https://api.github.com/repos/horizon-rust/horizon/releases?per_page=100&page=%d" % page)
    if not rels:
        break
    for r in rels:
        tag = r["tag_name"]
        assets = [a["name"] for a in r["assets"]]
        hit = "sdk" in tag.lower() or any("sdk" in n.lower() for n in assets)
        if hit:
            print("SDK HIT:", tag, "->", [n for n in assets if "sdk" in n.lower()])
    print("page", page, "scanned", len(rels), "releases")

# check latest stable release body for native SDK mention
rel = get("https://api.github.com/repos/horizon-rust/horizon/releases/tags/v0.2.97")
body = (rel.get("body") or "").lower()
print("\nv0.2.97 mentions 'sdk':", "sdk" in body)
print("v0.2.97 mentions 'native mod':", "native mod" in body)
