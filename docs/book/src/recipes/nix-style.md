# Nix-Style Recipes

Declarative, reproducible environments using forjar's pepita kernel
isolation instead of the Nix store. These recipes use cgroups v2,
overlayfs, and network namespaces to create hermetic development
and build environments.

## #11 Development Shell

Isolated development shell with pinned toolchain versions. Equivalent
to `nix develop` but using forjar's pepita transport for kernel-level
isolation. The shell environment is reproducible and ephemeral.

**Resources**: shell-env (pepita), toolchain-packages (package),
env-config (file)

**Tier**: 2+3 | **Idempotency**: Strong

## #12 Toolchain Pin

Pin specific compiler and runtime versions using pepita overlayfs
layers. Ensures builds use exact versions regardless of the host
system's installed packages.

**Resources**: toolchain-layer (pepita), version-pins (file),
path-config (file)

**Tier**: 2+3 | **Idempotency**: Strong

## #13 Build Sandbox

Hermetic build environment with network isolation and restricted
filesystem access. Uses pepita's netns to prevent network access
during builds, ensuring reproducibility.

**Resources**: sandbox-env (pepita), build-script (file),
network-policy (network)

**Tier**: 2+3 | **Idempotency**: Strong

## #14 System Profile

System-wide profile that layers packages and configuration on top
of the base OS. Similar to NixOS system profiles but using forjar's
overlay mechanism.

**Resources**: profile-layer (pepita), system-packages (package),
profile-config (file)

**Tier**: 2+3 | **Idempotency**: Strong

## #15 Workspace

Multi-project workspace with shared dependencies and isolated
per-project environments. Manages a collection of development
environments under a single forjar config.

**Resources**: workspace-root (file), shared-deps (pepita),
project-envs (pepita)

**Tier**: 2+3 | **Idempotency**: Strong
