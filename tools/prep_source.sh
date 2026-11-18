#!/usr/bin/env bash
# Extract the downloaded source tarball into the layout native.py expects:
#   native/target/horizon/<rev>/  with a .horizon-revision marker.
# Run INSIDE the container (paths are /workspace/...).
set -euo pipefail
REV=85b5bcca7c79bb470b90bdb5914b413aa66fe7ae
TAR=/workspace/native/target/horizon-src.tar.gz
DEST=/workspace/native/target/horizon/$REV
PARTIAL=/workspace/native/target/horizon/.$REV.partial

echo "=== verify tarball ==="
python3 -c "import tarfile; t=tarfile.open('$TAR','r:gz'); n=t.getnames(); print('entries',len(n),'top',n[0].split('/')[0])"

echo "=== extract ==="
rm -rf "$PARTIAL" "$DEST"
mkdir -p "$PARTIAL"
tar -xzf "$TAR" -C "$PARTIAL"
TOP=$(find "$PARTIAL" -mindepth 1 -maxdepth 1 -type d | head -1)
# native.py's horizon-mixins refuses symlinks; docs have a few, none build-critical.
find "$TOP" -type l -delete
echo "$REV" > "$TOP/.horizon-revision"
mv "$TOP" "$DEST"
rm -rf "$PARTIAL"

echo "=== ready ==="
ls -la "$DEST/.horizon-revision"
wc -l "$DEST/crates/horizon-client/src/game/native_hooks.rs" 2>/dev/null || echo "hooks file path differs"
grep -E "NATIVE_HOOKS_API" "$DEST/crates/horizon-client/src/game/native_hooks.rs" 2>/dev/null | head -2
echo "SOURCE READY at $DEST"
