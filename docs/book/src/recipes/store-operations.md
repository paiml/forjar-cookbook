# Store Operations

This section covers forjar's content-addressed store operations:
importing packages, managing caches, garbage collection, pin resolution,
and reproducible conversion.

## Provider Import

Import artifacts from any supported provider into the store:

```bash
# Import from apt
forjar store-import apt curl --version 7.88.1

# Import from cargo
forjar store-import cargo ripgrep --version 14.1.0

# Import a Docker image
forjar store-import docker alpine:3.18

# Import from Nix
forjar store-import nix nixpkgs#ripgrep

# List all providers
forjar store-import --list-providers
```

Each import follows an 8-step pipeline: validate, generate CLI, I8 gate,
create staging dir, execute, hash output, atomic move, write meta.yaml.

## Pin Resolution

Lock all input versions to ensure reproducible builds:

```bash
# Pin all inputs (creates forjar.inputs.lock.yaml)
forjar pin -f forjar.yaml

# Check if lock file is fresh (CI gate, exit 1 if stale)
forjar pin --check -f forjar.yaml

# Update a specific pin
forjar pin --update nginx -f forjar.yaml

# Update all pins
forjar pin --update -f forjar.yaml
```

Resolution queries each provider's CLI:

| Provider | Resolution Command |
|----------|-------------------|
| apt | `apt-cache policy <name>` |
| cargo | `cargo search <name> --limit 1` |
| nix | `nix eval <flake>.version --raw` |
| uv/pip | `pip index versions <name>` |
| docker | `docker image inspect <name>` |

## Cache Management

Push and pull store entries via SSH:

```bash
# List local store entries
forjar cache list

# Push all entries to SSH cache
forjar cache push deploy@cache.internal:/var/forjar/cache

# Push a specific hash
forjar cache push deploy@cache.internal:/cache --hash abc123

# Verify store integrity
forjar cache verify
```

The substitution protocol automatically checks caches before building:
local store -> SSH caches (in order) -> build from scratch.

## Garbage Collection

Remove unreachable store entries:

```bash
# Dry-run (see what would be deleted)
forjar store gc --dry-run

# Execute GC (with journal for recovery)
forjar store gc

# Keep more profile generations
forjar store gc --keep-generations 10

# JSON output for scripting
forjar store gc --dry-run --json
```

GC roots include: current profile, last N generations, lock file pins,
and `.gc-roots/` symlinks. A GC journal is written before deletion.

## Store Diff and Sync

Check if upstream has changed and re-import:

```bash
# Diff a store entry against its upstream
forjar store diff abc123def456

# Sync (dry-run)
forjar store sync abc123def456

# Sync (apply changes)
forjar store sync abc123def456 --apply
```

## Reproducible Conversion

Upgrade a recipe's reproducibility level:

```bash
# Analyze conversion opportunities
forjar convert --reproducible -f forjar.yaml

# Apply conversion (backup + version pins + store flags + lock file)
forjar convert --reproducible --apply -f forjar.yaml
```

The conversion ladder: Impure -> Constrained -> Pinned -> Pure.
`--apply` handles steps 1-3 automatically (version pins, store flags,
lock file generation).

## Sandbox Builds

Build packages in an isolated namespace with full reproducibility:

```yaml
# In forjar.yaml
resources:
  app:
    type: package
    machine: build-host
    packages: [ripgrep]
    provider: cargo
    version: "14.1.0"
    store: true
    sandbox:
      level: full         # full | network_only | minimal | none
      memory_mb: 2048
      cpus: 4
      timeout_secs: 600
```

Sandbox levels control isolation:

| Level | Network | Filesystem | Seccomp | Cgroups |
|-------|---------|-----------|---------|---------|
| Full | blocked | read-only inputs, overlayfs | BPF filter | memory + CPU |
| NetworkOnly | allowed | read-only inputs | none | memory + CPU |
| Minimal | allowed | PID/mount namespace | none | none |
| None | allowed | no isolation | none | none |

The sandbox lifecycle (10 steps): create namespace, mount overlayfs,
bind inputs read-only, apply cgroups, apply seccomp, execute script,
extract outputs, hash output, atomic store, cleanup namespace.

## Profile Management

Manage profile generations for instant rollback:

```bash
# List profile generations
forjar store list --generations

# Rollback to a previous generation
forjar store rollback --generation 2

# Current profile is an atomic symlink — rollback is crash-safe
```

Profile generations let you switch between different versions of your
entire dependency set atomically. See recipe #67 for a complete example.

## Store Listing

Browse store entries with provenance info:

```bash
# List all entries
forjar store list

# Show provider info
forjar store list --show-provider

# JSON output
forjar store list --json
```

## Execution Architecture

All store operations bridge plan generation to actual execution via
forjar's transport layer. Every shell command is validated through the
I8 bashrs provability gate before execution.

| Operation | Module | Pipeline |
|-----------|--------|----------|
| Import | `provider_exec.rs` | validate -> generate CLI -> I8 gate -> stage -> execute -> hash -> store |
| GC | `gc_exec.rs` | mark roots -> sweep -> journal -> delete -> report |
| Pin | `pin_resolve.rs` | query provider CLIs -> parse versions -> write lock file |
| Cache | `cache_exec.rs` | rsync to/from SSH -> verify hash -> atomic store |
| Convert | `convert_exec.rs` | backup -> modify YAML -> version pins -> store flags -> lock file |
| Diff/Sync | `sync_exec.rs` | query upstream -> compare hashes -> re-import -> replay derivations |
| Sandbox | `sandbox_run.rs` | create namespace -> isolate -> build -> extract -> hash -> store |

## Related Recipes

| Recipe | Topic | Key Feature |
|--------|-------|-------------|
| #63 | [Version-Pinned Store](../recipes/store-operations.md) | `store: true` + lock file |
| #64 | [Cargo Sandbox](../recipes/store-operations.md) | `sandbox: full` + isolation |
| #65 | [SSH Cache](../recipes/store-operations.md) | substitution protocol |
| #66 | [CI Repro Gate](../recipes/store-operations.md) | reproducibility scoring |
| #67 | [Profile Rollback](../recipes/store-operations.md) | generation management |
