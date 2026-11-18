#!/usr/bin/env bash
# Build the injected Horizon client on Windows with the launcher's own Rust.
set -o pipefail
ROOT="/g/WorkBuddyWork/2026-11-16-16-13-43/horizon-ports"
SRC="$ROOT/build-win/horizon-src"
RD="/c/Users/ASUS/AppData/Roaming/horizon-launcher/rust/1.99.1-x86_64-pc-windows-gnu"

export CARGO_HOME="G:/WorkBuddyWork/2026-11-16-16-13-43/horizon-ports/build-win/cargo-home"
export PATH="$RD/bin:/d/MinGW/bin:$PATH"
export CC_x86_64_pc_windows_gnu="D:/MinGW/bin/gcc.exe"
export AR_x86_64_pc_windows_gnu="D:/MinGW/bin/ar.exe"
export CARGO_BUILD_JOBS=8

cd "$SRC" || exit 1
echo "=== rustc ==="
rustc.exe --version
echo "=== build start $(date) ==="
cargo.exe build --locked -p horizon-client --bin horizon-client --profile fast 2>&1
STATUS=$?
echo "=== build end $(date) EXIT=$STATUS ==="
exit $STATUS
