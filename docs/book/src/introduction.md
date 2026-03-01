# Introduction

The Forjar Cookbook is a qualification suite that proves forjar works on real
infrastructure. Every recipe is a real forjar config that gets applied to real
machines on a self-hosted runner.

## Philosophy

This is not documentation — it is a living test harness. When a recipe exposes
a bug or missing feature in forjar, we:

1. **Stop** — record the gap in the qualification checklist
2. **Implement** — fix the bug or add the feature in the forjar repo
3. **Release** — publish the new forjar version
4. **Retry** — re-run the recipe and mark it qualified

## Quality Standards

- 95% minimum test coverage
- Zero clippy warnings
- `pmat comply` enforced
- `bashrs` linted shell scripts
- No source file over 500 lines
