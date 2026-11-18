#!/usr/bin/env python3
"""Compare the engine Text API between two Horizon commits (raw.githubusercontent)."""
import urllib.request

UA = {"User-Agent": "x"}
PATHS = ["crates/horizon-client-core/src/text.rs"]
REVS = {
    "ours_9ee13cd0": "9ee13cd038559db05de6a4ced4496028ce409564",
    "installed_9f42064c": "9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0",
}

def fetch(rev, path):
    url = f"https://raw.githubusercontent.com/horizon-rust/horizon/{rev}/{path}"
    try:
        return urllib.request.urlopen(urllib.request.Request(url, headers=UA), timeout=40).read().decode("utf-8", "replace")
    except Exception as e:
        return None

for path in PATHS:
    print("=====", path)
    texts = {}
    for name, rev in REVS.items():
        t = fetch(rev, path)
        texts[name] = t
        print(" ", name, "->", "OK %d bytes" % len(t) if t else "MISSING/404")
    a, b = texts.get("ours_9ee13cd0"), texts.get("installed_9f42064c")
    if a and b:
        if a == b:
            print("  => IDENTICAL")
        else:
            import difflib
            diff = list(difflib.unified_diff(a.splitlines(), b.splitlines(), "9ee13cd0", "9f42064c", lineterm="", n=2))
            print("  => DIFFERENT, %d diff lines; first 60:" % len(diff))
            for line in diff[:60]:
                print("   ", line)

# also print the Text struct + Span/Style definitions at installed rev for reference
t = texts.get("installed_9f42064c")
if t:
    print("\n===== installed rev: struct/impl signatures =====")
    for line in t.splitlines():
        s = line.strip()
        if s.startswith("pub struct") or s.startswith("pub fn") or s.startswith("impl") or s.startswith("pub struct Span") or "pub text" in s or "pub style" in s or "pub color" in s:
            print("  ", s)
