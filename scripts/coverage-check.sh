#!/bin/bash
# Coverage threshold check — enforces >= 95% line coverage.
# Uses cargo llvm-cov (NEVER tarpaulin).
set -euo pipefail

THRESHOLD=95

echo "Checking coverage threshold (>= ${THRESHOLD}%)..."

# Get summary output
SUMMARY=$(cargo llvm-cov --workspace --lib --summary-only 2>&1)

# Extract line coverage percentage
COVERAGE=$(echo "$SUMMARY" | grep "TOTAL" | awk '{print $NF}' | tr -d '%')

if [ -z "$COVERAGE" ]; then
    echo "ERROR: could not extract coverage from llvm-cov output"
    echo "$SUMMARY"
    exit 1
fi

# Compare (integer comparison — truncate decimal)
COVERAGE_INT=${COVERAGE%.*}

if [ "$COVERAGE_INT" -ge "$THRESHOLD" ]; then
    echo "PASS: coverage ${COVERAGE}% >= ${THRESHOLD}%"
    exit 0
else
    echo "FAIL: coverage ${COVERAGE}% < ${THRESHOLD}%"
    exit 1
fi
