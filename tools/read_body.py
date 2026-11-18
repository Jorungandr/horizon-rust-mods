#!/usr/bin/env python3
import json, urllib.request, re, html
UA = {"User-Agent": "x"}
def get(u):
    return json.loads(urllib.request.urlopen(urllib.request.Request(u, headers=UA), timeout=60).read())
for tag in ("nightly",):
    rel = get("https://api.github.com/repos/horizon-rust/horizon/releases/tags/" + tag)
    body = rel.get("body") or ""
    txt = re.sub(r"<[^>]+>", "\n", body)
    txt = html.unescape(txt)
    txt = "\n".join(line.strip() for line in txt.splitlines() if line.strip())
    print(txt)
