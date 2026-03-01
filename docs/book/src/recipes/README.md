# Recipes

The cookbook contains 62 recipes organized by category. Each recipe is a
real forjar config that gets applied to real machines during qualification.

- **Infrastructure** (#1-10) — Developer workstation, web server, database, monitoring, security
- **Nix-Style** (#11-15) — Dev shells, toolchain pins, build sandboxes using pepita isolation
- **Rust Builds** (#16-21) — Release builds, musl static binaries, cross-compilation
- **Package Distribution** (#25-29) — .deb/.rpm build, private repos, fleet deploy
- **OpenTofu Patterns** (#30-39) — Saved plans, check blocks, lifecycle rules, testing DSL
- **Linux Administration** (#40-49) — Cron, users, sysctl, logs, time sync, systemd, patching
- **Failure Modes** (#50-52) — Partial apply recovery, state recovery, crash resilience
- **Composition Stacks** (#53-62) — Multi-recipe compositions building full environments
