<p align="center">
  <img src="assets/hero.svg" alt="Forjar Cookbook" width="900"/>
</p>

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
**Qualification Summary** (updated: 2026-03-03 15:48 UTC)

| Status | Count |
|--------|-------|
| Qualified | 57 |
| Blocked   | 5 |
| Pending   | 0 |

**Grade Distribution**

| Grade | Count |
|-------|-------|
| A | 57 |
| B | 0 |
| C | 0 |
| D | 0 |
| F | 5 |

| # | Recipe | Category | Status | Grade | Tier | Idempotent | Time (1st) | Time (2nd) | Score | Blocker |
|---|--------|----------|--------|-------|------|------------|------------|------------|-------|---------|
| 1 | developer-workstation | infra | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 7.6s | 408ms | 94 | — |
| 2 | web-server | infra | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 11.5s | 971ms | 94 | — |
| 3 | postgresql-database | infra | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 17.6s | 364ms | 94 | — |
| 4 | monitoring-stack | infra | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Weak | 9.2s | 429ms | 93 | — |
| 5 | redis-cache | infra | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Weak | 9.1s | 382ms | 93 | — |
| 6 | ci-runner | infra | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 3 | Strong | 8.1s | 363ms | 94 | — |
| 7 | rocm-gpu | gpu | ![blocked](https://img.shields.io/badge/BLOCKED-red) | ![F](https://img.shields.io/badge/F-red) | 3 | Strong | — | — | 0 | FJ-1126: ROCm userspace not installed |
| 8 | nvidia-gpu | gpu | ![blocked](https://img.shields.io/badge/BLOCKED-red) | ![F](https://img.shields.io/badge/F-red) | 3 | Strong | — | — | 0 | FJ-1127: No NVIDIA hardware |
| 9 | secure-baseline | infra | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 36.8s | 355ms | 93 | — |
| 10 | nfs-file-server | infra | ![blocked](https://img.shields.io/badge/BLOCKED-red) | ![F](https://img.shields.io/badge/F-red) | 3 | Strong | — | — | 0 | FJ-1128: NFS kernel modules not loaded |
| 11 | dev-shell | nix | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 1+2 | Strong | 714ms | 22ms | 94 | — |
| 12 | toolchain-pin | nix | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 1+2 | Strong | 980ms | 21ms | 95 | — |
| 13 | build-sandbox | nix | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 1+2 | Strong | 639ms | 21ms | 94 | — |
| 14 | system-profile | nix | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 1+2 | Strong | 1.5s | 23ms | 94 | — |
| 15 | workspace | nix | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 1+2 | Strong | 1.3s | 25ms | 93 | — |
| 16 | rust-release | rust | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 1+2 | Strong | 712ms | 22ms | 94 | — |
| 17 | static-musl | rust | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 1+2 | Strong | 906ms | 22ms | 94 | — |
| 18 | multi-stage-build | rust | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 1+2 | Strong | 6.9s | 36ms | 95 | — |
| 19 | cross-compile | rust | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 1+2 | Strong | 1.1s | 22ms | 94 | — |
| 20 | sovereign-stack | advanced | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 1.2s | 21ms | 94 | — |
| 21 | apr-model | advanced | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 3 | Weak | 1.5s | 24ms | 93 | — |
| 22 | secrets-lifecycle | advanced | ![blocked](https://img.shields.io/badge/BLOCKED-red) | ![F](https://img.shields.io/badge/F-red) | 2+3 | Strong | — | — | 0 | FJ-1129: Secret provider exec fails |
| 23 | tls-certificates | advanced | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 1.1s | 23ms | 95 | — |
| 24 | fleet-provisioning | advanced | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 1.2s | 21ms | 94 | — |
| 25 | apt-repo | packages | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 883ms | 22ms | 95 | — |
| 26 | deb-package | packages | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 1.1s | 23ms | 94 | — |
| 27 | private-apt-repo | packages | ![blocked](https://img.shields.io/badge/BLOCKED-red) | ![F](https://img.shields.io/badge/F-red) | 2+3 | Strong | — | — | 0 | FJ-1130: GPG key import fails |
| 28 | rpm-build | packages | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 1.1s | 24ms | 94 | — |
| 29 | distribution-pipeline | packages | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 1.2s | 21ms | 94 | — |
| 30 | saved-plan | opentofu | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 1+2 | Strong | 833ms | 22ms | 95 | — |
| 31 | json-plan | opentofu | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 1+2 | Strong | 764ms | 21ms | 94 | — |
| 32 | check-blocks | opentofu | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 1+2 | Strong | 809ms | 210ms | 91 | — |
| 33 | lifecycle | opentofu | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 1+2 | Strong | 921ms | 22ms | 95 | — |
| 34 | moved-blocks | opentofu | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 1+2 | Strong | 508ms | 22ms | 94 | — |
| 35 | refresh-only | opentofu | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 1+2 | Strong | 738ms | 19ms | 95 | — |
| 36 | resource-targeting | opentofu | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 1+2 | Strong | 843ms | 22ms | 94 | — |
| 37 | testing-dsl | opentofu | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 1+2 | Strong | 713ms | 22ms | 94 | — |
| 38 | state-encryption | opentofu | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 1+2 | Strong | 968ms | 22ms | 95 | — |
| 39 | cross-config | opentofu | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 1+2 | Strong | 736ms | 23ms | 95 | — |
| 40 | scheduled-tasks | linux | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 1.2s | 21ms | 95 | — |
| 41 | user-provisioning | linux | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 375ms | 22ms | 94 | — |
| 42 | kernel-tuning | linux | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 910ms | 20ms | 95 | — |
| 43 | log-management | linux | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 1.0s | 22ms | 94 | — |
| 44 | time-sync | linux | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 810ms | 21ms | 94 | — |
| 45 | custom-systemd-units | linux | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 970ms | 20ms | 95 | — |
| 46 | resource-limits | linux | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 758ms | 22ms | 95 | — |
| 47 | automated-patching | linux | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 988ms | 21ms | 94 | — |
| 48 | hostname-locale-dns | linux | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 1.0s | 19ms | 95 | — |
| 49 | swap-memory | linux | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 3 | Weak | 711ms | 22ms | 93 | — |
| 50 | failure-partial-apply | failure | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 788ms | 23ms | 95 | — |
| 51 | failure-state-recovery | failure | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 936ms | 24ms | 95 | — |
| 52 | failure-idempotent-crash | failure | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 695ms | 22ms | 95 | — |
| 53 | stack-dev-server | composability | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 1.1s | 23ms | 94 | — |
| 54 | stack-web-production | composability | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 1.3s | 22ms | 94 | — |
| 55 | stack-gpu-lab | composability | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 3 | Strong | 1.1s | 21ms | 94 | — |
| 56 | stack-build-farm | composability | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 1.2s | 22ms | 94 | — |
| 57 | stack-package-pipeline | composability | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 1.3s | 21ms | 94 | — |
| 58 | stack-ml-inference | composability | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 3 | Weak | 1.3s | 21ms | 93 | — |
| 59 | stack-ci-infrastructure | composability | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 1.1s | 21ms | 94 | — |
| 60 | stack-sovereign-ai | composability | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 3 | Strong | 1.8s | 22ms | 94 | — |
| 61 | stack-fleet-baseline | composability | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 1.2s | 42ms | 94 | — |
| 62 | stack-cross-distro | composability | ![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen) | ![A](https://img.shields.io/badge/A-brightgreen) | 2+3 | Strong | 1.3s | 22ms | 94 | — |
<!-- QUALIFICATION_TABLE_END -->

## License

MIT
