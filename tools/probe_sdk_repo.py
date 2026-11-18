#!/usr/bin/env python3
import json, urllib.request
UA = {"User-Agent": "x"}
def get(u):
    try:
        return json.loads(urllib.request.urlopen(urllib.request.Request(u, headers=UA), timeout=40).read())
    except Exception as e:
        return {"__err": str(e)[:80]}

# 1. does an sdk repo exist?
for repo in ("sdk", "horizon-sdk", "mod-sdk", "horizon-mods"):
    r = get("https://api.github.com/repos/horizon-rust/" + repo)
    print("repo", repo, "->", r.get("full_name") or r.get("__err"))

# 2. launcher 0.2.70 release assets (maybe a tools index)
rel = get("https://api.github.com/repos/horizon-rust/horizon/releases/tags/launcher%400.2.70")
print("\nlauncher@0.2.70 assets:", [a["name"] for a in rel.get("assets", [])] if "assets" in rel else rel)

# 3. try to fetch sdk.json for our build from the release download path
for name in ("sdk.json", "SHA256SUMS"):
    url = "https://github.com/horizon-rust/horizon/releases/download/sdk-9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0/" + name
    try:
        data = urllib.request.urlopen(urllib.request.Request(url, headers=UA), timeout=30).read()
        print("FETCH OK", name, len(data))
    except Exception as e:
        print("FETCH FAIL", name, str(e)[:60])
