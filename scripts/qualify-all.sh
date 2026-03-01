#!/bin/bash
# Qualify all recipes on the self-hosted runner.
# Run from the cookbook project root.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
RECIPES_DIR="$ROOT_DIR/recipes"

echo "Qualifying all recipes in $RECIPES_DIR..."

PASS=0
FAIL=0
SKIP=0

for recipe in "$RECIPES_DIR"/*.yaml; do
    NAME=$(basename "$recipe")
    STATE_DIR="/tmp/cookbook-qualify-$(echo "$NAME" | sed 's/.yaml//')"

    echo "--- $NAME ---"
    rm -rf "$STATE_DIR"

    if cargo run --bin cookbook-runner -- qualify -f "$recipe" --state-dir "$STATE_DIR"; then
        PASS=$((PASS + 1))
    else
        FAIL=$((FAIL + 1))
    fi
    echo ""
done

echo "Results: $PASS qualified, $FAIL failed, $SKIP skipped"

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
