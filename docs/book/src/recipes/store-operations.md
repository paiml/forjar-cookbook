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
