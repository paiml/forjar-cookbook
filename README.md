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
**Qualification Summary** (updated: 2026-03-01 15:25 UTC)

| Status | Count |
|--------|-------|
| Qualified | 0 |
| Blocked   | 1 |
| Pending   | 60 |

**Grade Distribution**

| Grade | Count |
|-------|-------|
| A | 0 |
| B | 0 |
| C | 0 |
| D | 0 |
| F | 61 |

| # | Recipe | Category | Status | Grade | Tier | Idempotent | Time (1st) | Time (2nd) | Score | Blocker |
|---|--------|----------|--------|-------|------|------------|------------|------------|-------|---------|
| 1 | developer-workstation | infra | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 2 | web-server | infra | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 3 | postgresql-database | infra | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 4 | monitoring-stack | infra | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Weak | — | — | — | — |
| 5 | redis-cache | infra | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Weak | — | — | — | — |
| 6 | ci-runner | infra | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 3 | Strong | — | — | — | — |
| 7 | rocm-gpu | gpu | ![blocked](https://img.shields.io/badge/BLOCKED-red) | — | 3 | Strong | — | — | — | FJ-1126: ROCm userspace not installed |
| 8 | nvidia-gpu | gpu | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 3 | Strong | — | — | — | — |
| 9 | secure-baseline | infra | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 10 | nfs-file-server | infra | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 3 | Strong | — | — | — | — |
| 11 | dev-shell | nix | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 1+2 | Strong | — | — | — | — |
| 12 | toolchain-pin | nix | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 1+2 | Strong | — | — | — | — |
| 13 | build-sandbox | nix | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 1+2 | Strong | — | — | — | — |
| 14 | system-profile | nix | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 1+2 | Strong | — | — | — | — |
| 15 | workspace | nix | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 1+2 | Strong | — | — | — | — |
| 16 | rust-release | rust | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 1+2 | Strong | — | — | — | — |
| 17 | static-musl | rust | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 1+2 | Strong | — | — | — | — |
| 19 | cross-compile | rust | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 1+2 | Strong | — | — | — | — |
| 20 | sovereign-stack | advanced | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 21 | apr-model | advanced | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 3 | Weak | — | — | — | — |
| 22 | secrets-lifecycle | advanced | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 23 | tls-certificates | advanced | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 24 | fleet-provisioning | advanced | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 25 | apt-repo | packages | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 26 | deb-package | packages | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 27 | private-apt-repo | packages | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 28 | rpm-build | packages | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 29 | distribution-pipeline | packages | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 30 | saved-plan | opentofu | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 1+2 | Strong | — | — | — | — |
| 31 | json-plan | opentofu | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 1+2 | Strong | — | — | — | — |
| 32 | check-blocks | opentofu | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 1+2 | Strong | — | — | — | — |
| 33 | lifecycle | opentofu | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 1+2 | Strong | — | — | — | — |
| 34 | moved-blocks | opentofu | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 1+2 | Strong | — | — | — | — |
| 35 | refresh-only | opentofu | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 1+2 | Strong | — | — | — | — |
| 36 | resource-targeting | opentofu | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 1+2 | Strong | — | — | — | — |
| 37 | testing-dsl | opentofu | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 1+2 | Strong | — | — | — | — |
| 38 | state-encryption | opentofu | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 1+2 | Strong | — | — | — | — |
| 39 | cross-config | opentofu | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 1+2 | Strong | — | — | — | — |
| 40 | scheduled-tasks | linux | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 41 | user-group-provisioning | linux | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 42 | kernel-tuning | linux | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 43 | log-management | linux | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 44 | time-sync | linux | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 45 | custom-systemd-units | linux | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 46 | resource-limits | linux | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 47 | automated-patching | linux | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 48 | hostname-locale-dns | linux | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 49 | swap-memory | linux | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 3 | Weak | — | — | — | — |
| 50 | failure-partial-apply | failure | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 51 | failure-state-recovery | failure | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 52 | failure-idempotent-crash | failure | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 53 | stack-dev-server | composability | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 54 | stack-web-production | composability | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 55 | stack-gpu-lab | composability | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 3 | Strong | — | — | — | — |
| 56 | stack-build-farm | composability | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 57 | stack-package-pipeline | composability | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 58 | stack-ml-inference | composability | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 3 | Weak | — | — | — | — |
| 59 | stack-ci-infrastructure | composability | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 60 | stack-sovereign-ai | composability | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 3 | Strong | — | — | — | — |
| 61 | stack-fleet-baseline | composability | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
| 62 | stack-cross-distro | composability | ![pending](https://img.shields.io/badge/PENDING-lightgray) | — | 2+3 | Strong | — | — | — | — |
<!-- QUALIFICATION_TABLE_END -->

## License

MIT
