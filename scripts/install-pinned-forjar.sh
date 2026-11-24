#!/bin/bash
# Install the forjar BINARY at the version Cargo.lock pins for the forjar LIBRARY.
#
# Why: CI ran `cargo install forjar --locked`, which takes whatever crates.io
# calls latest. The library the workspace links was pinned by Cargo.lock; the
# binary the recipe gate shells out to was not. forjar 1.34.2 (2026-11-23)
# made `forjar plan` refuse an empty state dir, so `plan_all` went 0/98 with
# no commit in this repo. A gate whose input moves without a commit cannot be
# bisected. It also compiled forjar from source on every run (539 s of a
# 650 s job, measured on run 43797854478).
#
# What: download the release asset for the pinned version, verify its sha256,
# install it, then ask the installed binary for its version.
#
# Usage:
#   install-pinned-forjar.sh            install into $FORJAR_BIN_DIR (default ~/.cargo/bin)
#   install-pinned-forjar.sh --print    print the pinned version and exit
#   install-pinned-forjar.sh --check    exit 1 unless `forjar` on PATH is the pinned version
# Env: FORJAR_LOCK (default: Cargo.lock beside scripts/), FORJAR_BIN_DIR, GH_TOKEN for gh.
set -euo pipefail

SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
LOCK="${FORJAR_LOCK:-$ROOT_DIR/Cargo.lock}"
MODE="${1:-install}"

fatal() {
    echo "FATAL: $*" >&2
    exit 1
}

[ -r "$LOCK" ] || fatal "cannot read $LOCK"

# The [[package]] entry is `name = "forjar"` followed by `version = "X.Y.Z"`.
VERSION="$(awk '
    $0 == "name = \"forjar\"" { hit = 1; next }
    hit == 1 {
        if ($1 == "version" && $2 == "=") { gsub(/"/, "", $3); print $3 }
        exit
    }' "$LOCK")"

case "$VERSION" in
    "" | *[!0-9.]* | .* | *. | *..*) fatal "no forjar version found in $LOCK (got '$VERSION')" ;;
esac

installed_version() {
    # `forjar --version` prints `forjar X.Y.Z (sha)`.
    "$1" --version 2>/dev/null | awk 'NR == 1 { print $2 }'
}

case "$MODE" in
    --print)
        echo "$VERSION"
        exit 0
        ;;
    --check)
        BIN="$(command -v forjar || true)"
        [ -n "$BIN" ] || fatal "forjar is not on PATH (Cargo.lock pins $VERSION)"
        GOT="$(installed_version "$BIN")"
        [ "$GOT" = "$VERSION" ] || fatal "forjar on PATH ($BIN) is '$GOT', Cargo.lock pins $VERSION"
        echo "OK: forjar $GOT at $BIN matches the Cargo.lock pin"
        exit 0
        ;;
    install) ;;
    *) fatal "unknown mode '$MODE' (use --print, --check or no argument)" ;;
esac

ARCH="$(uname -m)"
case "$ARCH" in
    x86_64 | aarch64) ;;
    *) fatal "no forjar release asset for architecture '$ARCH'" ;;
esac

DEST="${FORJAR_BIN_DIR:-$HOME/.cargo/bin}"
ASSET="forjar-$VERSION-$ARCH-unknown-linux-musl.tar.gz"
WORK="$(mktemp -d)"
trap 'rm -rf "${WORK:?}"' EXIT

gh release download "v$VERSION" --repo paiml/forjar \
    --pattern "$ASSET" --pattern "$ASSET.sha256" --dir "$WORK" \
    || fatal "could not download $ASSET from paiml/forjar release v$VERSION"

(cd "$WORK" && sha256sum --check --strict "$ASSET.sha256") \
    || fatal "sha256 mismatch for $ASSET"

tar -xzf "$WORK/$ASSET" -C "$WORK" forjar
mkdir -p "$DEST"
install -m 0755 "$WORK/forjar" "$DEST/forjar"

GOT="$(installed_version "$DEST/forjar")"
[ "$GOT" = "$VERSION" ] || fatal "installed binary reports '$GOT', expected $VERSION"
echo "OK: forjar $GOT installed at $DEST/forjar (Cargo.lock pin $VERSION)"
