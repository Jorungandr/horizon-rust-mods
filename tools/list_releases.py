#!/usr/bin/env python3
import json, urllib.request
UA = {"User-Agent": "x"}
def get(u):
    return json.loads(urllib.request.urlopen(urllib.request.Request(u, headers=UA), timeout=60).read())

rels = get("https://api.github.com/repos/horizon-rust/horizon/releases?per_page=50")
for r in rels:
    names = [a["name"] for a in r["assets"]]
    sdk = [n for n in names if "sdk" in n.lower()]
    print(r["tag_name"], "| assets:", len(names), "| sdk:", sdk[:4])
