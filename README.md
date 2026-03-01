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
**Qualification Summary** (updated: 2026-03-01 16:36 UTC)

| Status | Count |
|--------|-------|
| Qualified | 56 |
| Blocked   | 5 |
| Pending   | 0 |

**Grade Distribution**

| Grade | Count |
|-------|-------|
| A | 0 |
| B | 0 |
| C | 0 |
| D | 56 |
| F | 5 |

| # | Recipe | Category | Status | Grade | Tier | Idempotent | Time (1st) | Time (2nd) | Score | Blocker |
|---|--------|----------|--------|-------|------|------------|------------|------------|-------|---------|
| 1 | developer-workstation | infra | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 10.4s | 478ms | 68 | — |
| 2 | web-server | infra | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 8.4s | 346ms | 72 | — |
| 3 | postgresql-database | infra | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 15.2s | 355ms | 76 | — |
| 4 | monitoring-stack | infra | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Weak | 8.1s | 350ms | 72 | — |
| 5 | redis-cache | infra | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Weak | 7.5s | 325ms | 74 | — |
| 6 | ci-runner | infra | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 3 | Strong | 9.5s | 370ms | 68 | — |
| 7 | rocm-gpu | gpu | ![blocked](https://img.shields.io/badge/BLOCKED-red) | ![F](https://img.shields.io/badge/F-red) | 3 | Strong | — | — | 0 | FJ-1126: ROCm userspace not installed |
| 8 | nvidia-gpu | gpu | ![blocked](https://img.shields.io/badge/BLOCKED-red) | ![F](https://img.shields.io/badge/F-red) | 3 | Strong | — | — | 0 | FJ-1127: No NVIDIA hardware |
| 9 | secure-baseline | infra | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 38.3s | 337ms | 71 | — |
| 10 | nfs-file-server | infra | ![blocked](https://img.shields.io/badge/BLOCKED-red) | ![F](https://img.shields.io/badge/F-red) | 3 | Strong | — | — | 0 | FJ-1128: NFS kernel modules not loaded |
| 11 | dev-shell | nix | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 1+2 | Strong | 926ms | 25ms | 82 | — |
| 12 | toolchain-pin | nix | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 1+2 | Strong | 1.2s | 21ms | 81 | — |
| 13 | build-sandbox | nix | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 1+2 | Strong | 866ms | 20ms | 73 | — |
| 14 | system-profile | nix | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 1+2 | Strong | 9.8s | 20ms | 72 | — |
| 15 | workspace | nix | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 1+2 | Strong | 8.3s | 23ms | 79 | — |
| 16 | rust-release | rust | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 1+2 | Strong | 877ms | 20ms | 72 | — |
| 17 | static-musl | rust | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 1+2 | Strong | 9.5s | 32ms | 72 | — |
| 19 | cross-compile | rust | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 1+2 | Strong | 13.1s | 20ms | 72 | — |
| 20 | sovereign-stack | advanced | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 1.4s | 22ms | 72 | — |
| 21 | apr-model | advanced | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 3 | Weak | 1.7s | 23ms | 72 | — |
| 22 | secrets-lifecycle | advanced | ![blocked](https://img.shields.io/badge/BLOCKED-red) | ![F](https://img.shields.io/badge/F-red) | 2+3 | Strong | — | — | 0 | FJ-1129: Secret provider exec fails |
| 23 | tls-certificates | advanced | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 1.4s | 22ms | 81 | — |
| 24 | fleet-provisioning | advanced | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 11.2s | 24ms | 72 | — |
| 25 | apt-repo | packages | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 1.1s | 21ms | 72 | — |
| 26 | deb-package | packages | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 1.2s | 23ms | 80 | — |
| 27 | private-apt-repo | packages | ![blocked](https://img.shields.io/badge/BLOCKED-red) | ![F](https://img.shields.io/badge/F-red) | 2+3 | Strong | — | — | 0 | FJ-1130: GPG key import fails |
| 28 | rpm-build | packages | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 8.8s | 30ms | 80 | — |
| 29 | distribution-pipeline | packages | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 1.7s | 27ms | 80 | — |
| 30 | saved-plan | opentofu | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 1+2 | Strong | 987ms | 22ms | 81 | — |
| 31 | json-plan | opentofu | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 1+2 | Strong | 8.6s | 22ms | 80 | — |
| 32 | check-blocks | opentofu | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 1+2 | Strong | 1.4s | 188ms | 79 | — |
| 33 | lifecycle | opentofu | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 1+2 | Strong | 1.4s | 21ms | 81 | — |
| 34 | moved-blocks | opentofu | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 1+2 | Strong | 1.3s | 21ms | 81 | — |
| 35 | refresh-only | opentofu | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 1+2 | Strong | 1.4s | 21ms | 82 | — |
| 36 | resource-targeting | opentofu | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 1+2 | Strong | 2.1s | 20ms | 81 | — |
| 37 | testing-dsl | opentofu | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 1+2 | Strong | 886ms | 20ms | 72 | — |
| 38 | state-encryption | opentofu | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 1+2 | Strong | 1.2s | 24ms | 82 | — |
| 39 | cross-config | opentofu | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 1+2 | Strong | 942ms | 24ms | 82 | — |
| 40 | scheduled-tasks | linux | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 1.2s | 22ms | 81 | — |
| 41 | user-group-provisioning | linux | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 1.4s | 22ms | 81 | — |
| 42 | kernel-tuning | linux | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 1.1s | 28ms | 79 | — |
| 43 | log-management | linux | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 1.4s | 21ms | 80 | — |
| 44 | time-sync | linux | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 8.7s | 22ms | 80 | — |
| 45 | custom-systemd-units | linux | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 1.2s | 23ms | 81 | — |
| 46 | resource-limits | linux | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 912ms | 21ms | 79 | — |
| 47 | automated-patching | linux | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 13.5s | 29ms | 80 | — |
| 48 | hostname-locale-dns | linux | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 1.4s | 21ms | 79 | — |
| 49 | swap-memory | linux | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 3 | Weak | 877ms | 21ms | 81 | — |
| 50 | failure-partial-apply | failure | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 992ms | 22ms | 81 | — |
| 51 | failure-state-recovery | failure | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 2.1s | 32ms | 80 | — |
| 52 | failure-idempotent-crash | failure | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 1.2s | 30ms | 81 | — |
| 53 | stack-dev-server | composability | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 8.4s | 22ms | 72 | — |
| 54 | stack-web-production | composability | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 1.5s | 22ms | 81 | — |
| 55 | stack-gpu-lab | composability | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 3 | Strong | 8.4s | 23ms | 80 | — |
| 56 | stack-build-farm | composability | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 1.5s | 20ms | 72 | — |
| 57 | stack-package-pipeline | composability | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 1.4s | 21ms | 80 | — |
| 58 | stack-ml-inference | composability | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 3 | Weak | 1.4s | 24ms | 81 | — |
| 59 | stack-ci-infrastructure | composability | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 8.7s | 38ms | 72 | — |
| 60 | stack-sovereign-ai | composability | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 3 | Strong | 8.9s | 22ms | 72 | — |
| 61 | stack-fleet-baseline | composability | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 1.5s | 22ms | 81 | — |
| 62 | stack-cross-distro | composability | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![D](https://img.shields.io/badge/D-orange) | 2+3 | Strong | 1.5s | 23ms | 80 | — |
<!-- QUALIFICATION_TABLE_END -->

## License

MIT
