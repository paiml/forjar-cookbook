#!/bin/bash
# Self-test for install-pinned-forjar.sh: the pin reader and --check must be
# able to FAIL. Uses fake forjar binaries and fixture locks; no network.
set -euo pipefail

SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
SUT="$SCRIPT_DIR/install-pinned-forjar.sh"
WORK="$(mktemp -d)"
trap 'rm -rf "${WORK:?}"' EXIT
CASES=0
FAILED=0

expect() { # expect <want-exit> <label> <cmd...>
    local want="$1" label="$2" rc=0
    shift 2
    "$@" >"$WORK/out" 2>&1 || rc=$?
    CASES=$((CASES + 1))
    if [ "$rc" -eq "$want" ]; then
        echo "  ok: $label (exit $rc)"
    else
        echo "  NOT OK: $label: exit $rc, wanted $want"
        cat "$WORK/out"
        FAILED=$((FAILED + 1))
    fi
}

fake_forjar() { # fake_forjar <dir> <version>
    mkdir -p "$1"
    printf '#!/bin/sh\necho "forjar %s (abc1234)"\n' "$2" >"$1/forjar"
    chmod +x "$1/forjar"
}

lock() { # lock <file> <body>
    printf '%b' "$2" >"$1"
}

GOOD="$WORK/good.lock"
lock "$GOOD" '[[package]]\nname = "forjar-contracts"\nversion = "9.9.9"\n\n[[package]]\nname = "forjar"\nversion = "1.33.0"\nsource = "registry"\n'
ABSENT="$WORK/absent.lock"
lock "$ABSENT" '[[package]]\nname = "forjar-contracts"\nversion = "9.9.9"\n'
NOVERSION="$WORK/noversion.lock"
lock "$NOVERSION" '[[package]]\nname = "forjar"\nsource = "registry"\nversion = "1.33.0"\n'
JUNK="$WORK/junk.lock"
lock "$JUNK" '[[package]]\nname = "forjar"\nversion = "1.33.0; rm"\n'

fake_forjar "$WORK/match" 1.33.0
fake_forjar "$WORK/drift" 1.34.2
mkdir -p "$WORK/empty"
BASE="/usr/bin:/bin"

expect 0 "--print reads the forjar entry, not forjar-contracts" \
    env FORJAR_LOCK="$GOOD" "$SUT" --print
PRINTED="$(FORJAR_LOCK="$GOOD" "$SUT" --print)"
CASES=$((CASES + 1))
if [ "$PRINTED" = "1.33.0" ]; then
    echo "  ok: --print prints 1.33.0"
else
    echo "  NOT OK: --print printed '$PRINTED'"
    FAILED=$((FAILED + 1))
fi
expect 1 "lock without a forjar entry is fatal" env FORJAR_LOCK="$ABSENT" "$SUT" --print
expect 1 "entry whose next line is not version is fatal" env FORJAR_LOCK="$NOVERSION" "$SUT" --print
expect 1 "non-numeric version is fatal" env FORJAR_LOCK="$JUNK" "$SUT" --print
expect 1 "unreadable lock is fatal" env FORJAR_LOCK="$WORK/missing.lock" "$SUT" --print
expect 0 "--check passes when PATH forjar is the pin" \
    env FORJAR_LOCK="$GOOD" PATH="$WORK/match:$BASE" "$SUT" --check
expect 1 "--check fails when PATH forjar has drifted (1.34.2 vs 1.33.0)" \
    env FORJAR_LOCK="$GOOD" PATH="$WORK/drift:$BASE" "$SUT" --check
expect 1 "--check fails when forjar is not on PATH" \
    env FORJAR_LOCK="$GOOD" PATH="$WORK/empty:$BASE" "$SUT" --check
expect 1 "unknown mode is fatal" env FORJAR_LOCK="$GOOD" "$SUT" --bogus

echo "install-pinned-forjar-test: $CASES case(s), $FAILED failed"
[ "$CASES" -eq 10 ] || { echo "FAIL: ran $CASES cases, expected 10"; exit 1; }
[ "$FAILED" -eq 0 ]
