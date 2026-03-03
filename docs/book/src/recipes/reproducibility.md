# Reproducibility Series (Recipes 63–67)

The reproducibility series demonstrates Forjar's Nix-compatible content-addressed store model. Each recipe builds on the previous, progressing from version pinning to full profile generation with rollback.

## Overview

| Recipe | Name | Concepts |
|--------|------|----------|
| 63 | `store-version-pinned` | Version-pinned apt, `store: true`, lock file |
| 64 | `store-cargo-sandbox` | Sandboxed cargo build, input closure tracking |
| 65 | `store-ssh-cache` | Multi-machine SSH cache, substitution protocol |
| 66 | `store-repro-ci-gate` | Reproducibility score CI gate, purity validation |
| 67 | `store-profile-rollback` | Profile generations, atomic symlink, GC |

## Key Concepts

### Content-Addressed Store

Every build output lives at a deterministic path:

```
/var/lib/forjar/store/<blake3-hash>/
├── meta.yaml          # Input manifest, provenance
└── content/           # Build output
```

The hash is derived from: `composite_hash([recipe_hash, input_hashes, arch, provider])`. Same inputs always produce the same path.

### 4-Level Purity Model

| Level | Name | Requirement |
|-------|------|-------------|
| 0 | Pure | Version + store + sandbox |
| 1 | Pinned | Version + store (no sandbox) |
| 2 | Constrained | Provider-scoped, floating version |
| 3 | Impure | Unconstrained (curl\|bash) |

A recipe's purity is the **maximum** (least pure) of all its transitive dependencies.

### Recipe 63: Version-Pinned Store

The foundation. Adds `version:` and `store: true` to apt packages:

```yaml
nginx:
  type: package
  provider: apt
  packages: [nginx]
  version: "1.24.0-2ubuntu7.3"
  store: true
```

This achieves **Pinned** purity (level 1) — version-locked but not sandboxed.

### Recipe 64: Sandboxed Cargo Build

Adds `sandbox: full` for full build isolation:

```yaml
sandbox-config:
  type: file
  path: /etc/forjar/sandbox.yaml
  content: |
    level: full
    memory_mb: 2048
    cpus: 4.0
    timeout: 600
```

With input closure tracking via `forjar.inputs.lock.yaml`.

### Recipe 65: SSH Binary Cache

Configures multi-machine cache substitution:

```yaml
cache-config:
  type: file
  path: /etc/forjar/cache.yaml
  content: |
    sources:
      - type: ssh
        host: cache.internal
        user: forjar
        path: /var/lib/forjar/cache
      - type: local
        path: /var/lib/forjar/store
    auto_push: true
```

Substitution protocol: local store → SSH cache → build from scratch.

### Recipe 66: CI Gate

Reproducibility score validation for CI pipelines:

```yaml
ci-gate-config:
  type: file
  path: /etc/forjar/ci-gate.yaml
  content: |
    validation:
      min_purity_level: pinned
      min_score: 75
      strict_mode: true
```

### Recipe 67: Profile Rollback

Profile generation management with atomic rollback:

```yaml
gen-config:
  type: file
  path: /etc/forjar/generation.yaml
  content: |
    current_generation: 1
    rollback:
      enabled: true
      keep_generations: 5
```

## CLI Commands

These recipes use Forjar's store CLI commands:

```bash
forjar pin                          # Pin all inputs
forjar pin --check                  # CI gate — fail if stale
forjar cache list                   # List store entries
forjar cache verify                 # Re-hash all entries
forjar store gc --dry-run           # Preview garbage collection
forjar store diff <hash>            # Diff against upstream
forjar archive pack <hash>          # Pack into .far
forjar convert --reproducible       # Auto-convert recipe
forjar store-import apt nginx       # Import from any provider
forjar store-import --list-providers # List 8 supported providers
```

## Runtime Executors

Behind the CLI, three executor modules handle the build lifecycle:

- **Sandbox executor** (FJ-1316): 10-step lifecycle — create namespace, overlay mount, bind inputs read-only, cgroup limits, seccomp BPF (Full level: deny connect/mount/ptrace), build, hash, store, cleanup.
- **Substitution protocol** (FJ-1322): local store → SSH cache → build fallback → auto-push. Avoids rebuilding when an entry already exists.
- **Derivation executor** (FJ-1342): Resolves inputs, computes closure hash, checks store for hits (skip build), orchestrates sandbox build for misses. Supports DAG execution for chained derivations.

## Lock File Freshness

The lock file (`forjar.inputs.lock.yaml`) pins every input to an
exact version and BLAKE3 hash. Pin tripwire checks detect staleness:

```bash
# Check if all pins are fresh (CI gate)
forjar pin --check -f forjar.yaml
# Exit code 0 = fresh, 1 = stale or missing pins

# Update a single pin
forjar pin --update nginx -f forjar.yaml

# Update all pins to latest versions
forjar pin --update -f forjar.yaml
```

In strict mode (`strict_mode: true`), stale pins produce **errors**
instead of warnings — useful for CI pipelines where freshness is
mandatory.

## Secret Scanning

Forjar scans all config values for plaintext secrets using 15 regex
patterns (AWS keys, PEM headers, JWT tokens, etc.). Sensitive values
must use age encryption:

```yaml
# BAD — plaintext secret detected
params:
  db_password: "supersecret123"

# GOOD — age-encrypted
params:
  db_password: "ENC[age,encrypted-value-here]"
```

Secret scanning runs automatically during `forjar apply` and
`forjar validate`. See `cargo run --example store_secret_scan`.

## Running the Series

```bash
cd forjar-cookbook
cargo run -p cookbook-runner -- validate recipes/63-store-version-pinned.yaml
cargo run -p cookbook-runner -- validate recipes/64-store-cargo-sandbox.yaml
cargo run -p cookbook-runner -- validate recipes/65-store-ssh-cache.yaml
cargo run -p cookbook-runner -- validate recipes/66-store-repro-ci-gate.yaml
cargo run -p cookbook-runner -- validate recipes/67-store-profile-rollback.yaml
```
