#!/bin/bash
# Documentation consistency check — ensures README matches CSV.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
CSV="$ROOT_DIR/docs/certifications/recipes.csv"
README="$ROOT_DIR/README.md"

echo "Checking documentation consistency..."

# 1. CSV exists and has content
if [ ! -f "$CSV" ]; then
    echo "FAIL: $CSV not found"
    exit 1
fi

CSV_ROWS=$(tail -n +2 "$CSV" | grep -c '.' || true)
if [ "$CSV_ROWS" -eq 0 ]; then
    echo "FAIL: $CSV has no data rows"
    exit 1
fi
echo "  CSV: $CSV_ROWS recipes"

# 2. README has markers
if ! grep -q "QUALIFICATION_TABLE_START" "$README"; then
    echo "FAIL: README.md missing QUALIFICATION_TABLE_START marker"
    exit 1
fi
if ! grep -q "QUALIFICATION_TABLE_END" "$README"; then
    echo "FAIL: README.md missing QUALIFICATION_TABLE_END marker"
    exit 1
fi
echo "  README: markers present"

# 3. README has qualification summary
if ! grep -q "Qualification Summary" "$README"; then
    echo "FAIL: README.md missing Qualification Summary"
    exit 1
fi
echo "  README: summary present"

echo "PASS: documentation consistency checks passed"
