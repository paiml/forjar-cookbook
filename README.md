# Forjar Cookbook

Qualification suite that proves forjar works on real infrastructure.

Every recipe is a real forjar config applied to real machines. When a recipe
exposes a bug or missing feature, we stop, implement the fix in forjar,
then retry and mark it qualified.

**Primary runner**: Self-hosted Intel (32-core Xeon, 283 GB RAM, 2x AMD GPU)

## Quick Start

```bash
# Validate all recipes
cargo run --example validate_all

# Plan all recipes (dry-run)
cargo run --example plan_all

# Qualify a single recipe on the runner
make qualify-recipe RECIPE=01

# Update README dashboard from CSV
make update-qualifications
```

## Quality Gates

| Gate | Threshold |
|------|-----------|
| Test coverage | >= 95% (`cargo llvm-cov`) |
| Lint | Zero warnings (`cargo clippy -- -D warnings`) |
| Format | Zero diff (`cargo fmt --check`) |
| Code health | `pmat comply check` passes |
| Shell safety | `bashrs lint scripts/ Makefile` |
| Docs | `./scripts/check-docs-consistency.sh` |

## Qualification Dashboard

<!-- QUALIFICATION_TABLE_START -->
**Qualification Summary** (updated: pending initial qualification)

| Status | Count |
|--------|-------|
| Qualified | 0 |
| Blocked   | 1 |
| Pending   | 19 |

| # | Recipe | Category | Status | Tier | Idempotent | Time (1st) | Time (2nd) | Blocker |
|---|--------|----------|--------|------|------------|------------|------------|---------|
| 1 | developer-workstation | infra | ![pending](https://img.shields.io/badge/PENDING-lightgray) | 2+3 | Strong | — | — | — |
| 2 | web-server | infra | ![pending](https://img.shields.io/badge/PENDING-lightgray) | 2+3 | Strong | — | — | — |
| 3 | postgresql-database | infra | ![pending](https://img.shields.io/badge/PENDING-lightgray) | 2+3 | Strong | — | — | — |
| 4 | monitoring-stack | infra | ![pending](https://img.shields.io/badge/PENDING-lightgray) | 2+3 | Weak | — | — | — |
| 5 | redis-cache | infra | ![pending](https://img.shields.io/badge/PENDING-lightgray) | 2+3 | Weak | — | — | — |
| 7 | rocm-gpu | gpu | ![blocked](https://img.shields.io/badge/BLOCKED-red) | 3 | Strong | — | — | FJ-1126: ROCm userspace not installed |
| 9 | secure-baseline | infra | ![pending](https://img.shields.io/badge/PENDING-lightgray) | 2+3 | Strong | — | — | — |
| 40 | scheduled-tasks | linux | ![pending](https://img.shields.io/badge/PENDING-lightgray) | 2+3 | Strong | — | — | — |
<!-- QUALIFICATION_TABLE_END -->

## License

MIT
