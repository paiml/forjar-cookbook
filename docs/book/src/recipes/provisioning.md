# Provisioning & Image Generation

Zero-touch machine provisioning: from bare metal to fully converged in a
single boot. These recipes and commands cover cross-machine builds, machine
bootstrap, cargo binary caching, bootable ISO generation, and Android
image packaging.

## Build Resource Type (FJ-33)

The `build` resource type introduces a two-machine workflow: compile on
a powerful build machine, deploy the artifact to a target device via
SSH+SCP.

```yaml
apr-binary:
  type: build
  machine: jetson           # deploy target
  build_machine: intel      # where compilation runs
  command: "cargo build --release --target aarch64-unknown-linux-gnu -p apr-cli"
  working_dir: ~/src/aprender
  source: /tmp/cross/release/apr    # artifact path on build machine
  target: ~/.cargo/bin/apr          # deploy path on target machine
  completion_check: "apr --version"
```

### Execution phases

1. **Build**: SSH to `build_machine`, execute `command` in `working_dir`
2. **Transfer**: SCP artifact from `build_machine:source` to `target`
3. **Verify**: Run `completion_check` locally on deploy machine

When `build_machine: localhost`, phases 1-2 use local `cp` instead of SSH/SCP.

### Fields

| Field | Required | Description |
|-------|----------|-------------|
| `build_machine` | yes | Machine name or `localhost` |
| `command` | yes | Build command to execute |
| `source` | yes | Artifact path on build machine |
| `target` | yes | Deploy path on target machine |
| `working_dir` | no | Build directory (default: home) |
| `completion_check` | no | Verify command (fallback: `test -x target`) |

### Planner integration

- Default state: `present`
- Proof obligation: Convergent
- Reversibility: Reversible (delete deployed artifact)
- Graph cost: 5 (same as Package)

### See also

- Recipe #19 (Cross-Compilation) — cross-compile toolchain setup
- Recipe #18 (Multi-Stage Build) — build pipeline without SSH

## Bootstrap Command (FJ-49)

New bare-metal machines require SSH key setup and sudo configuration
before `forjar apply` can manage them. The bootstrap command automates
this:

```bash
forjar bootstrap -f forjar.yaml --machine yoga
forjar bootstrap -f forjar.yaml --machine yoga --password
```

### Bootstrap phases

1. **SSH key injection**: Copy public key via `ssh-copy-id`
   (uses `sshpass` if `--password` provided)
2. **Sudo configuration**: Write passwordless sudo rule to
   `/etc/sudoers.d/<user>-nopasswd`
3. **Verification**: Confirm key-based auth AND `sudo -n true` succeeds

### Preconditions

- Machine must be defined in `forjar.yaml` with `hostname`, `addr`, `user`
- `ssh_key` field specifies the identity file (`.pub` suffix auto-appended)
- Target must have `sshd` running

Machines provisioned via `forjar image` (FJ-52) skip bootstrap entirely
since SSH keys and sudo are pre-configured in the autoinstall.

### See also

- Recipe #24 (Fleet Provisioning) — fleet-scale SSH key + identity setup

## Cargo Binary Cache (FJ-51)

`cargo install` recompiles from source every time. The cargo cache
stores compiled binaries at `~/.forjar/cache/cargo/<pkg>-<version>-<arch>/bin/`:

```
cache check → HIT:  cp from cache → done
            → MISS: cargo install --root staging
                    → populate cache
                    → cp to $CARGO_HOME/bin
```

### Cache key

`<package>-<version|"latest">-$(uname -m)` — architecture-aware to
prevent cross-arch cache poisoning.

### Environment controls

| Variable | Effect |
|----------|--------|
| `FORJAR_CACHE_DIR` | Override cache root |
| `FORJAR_NO_CARGO_CACHE` | Disable caching (force fresh build) |

## Autoinstall ISO Generation (FJ-52)

`forjar image` generates bootable Ubuntu autoinstall ISOs from
`forjar.yaml`:

```bash
# Generate user-data only (for PXE or manual ISO build)
forjar image --user-data -f forjar.yaml -m yoga

# Generate bootable ISO
forjar image --base ubuntu-22.04-live-server-amd64.iso \
  -f forjar.yaml -m yoga -o yoga-autoinstall.iso
```

### User-data generation

Reads the `machines:` section and produces Ubuntu autoinstall YAML:

| Machine field | Autoinstall field |
|---------------|-------------------|
| `hostname` | `identity.hostname` |
| `user` | `identity.username` |
| `ssh_key` | `ssh.authorized-keys` |
| `addr` | Static IP comment |

### First-boot convergence

The generated ISO includes a systemd oneshot service:

```ini
[Service]
Type=oneshot
ExecStart=/usr/local/bin/forjar apply --yes -f /etc/forjar/forjar.yaml
ExecStartPost=/usr/bin/touch /etc/forjar/.firstboot-done
```

Idempotent: runs once, creates marker file, never runs again.

### ISO repacking

When `--base` is provided, the command extracts the base ISO with
`xorriso`, injects user-data + forjar binary, and repacks with UEFI +
legacy boot support.

## Android Image Generation (FJ-54)

`forjar image --android` generates a Magisk module ZIP for rooted
Android devices:

```bash
forjar image --android -f forjar.yaml -m pixel -o forjar-magisk.zip
```

### Module structure

```
forjar-magisk.zip/
  module.prop              # Metadata (id, name, version, minMagisk)
  customize.sh             # Installer: set permissions, create dirs
  post-fs-data.sh          # Early boot: create /data/forjar
  service.sh               # Boot service: run forjar apply
  system/
    bin/forjar             # Binary stub (cross-compile separately)
    etc/init/forjar.rc     # Android init.rc service stanza
    etc/forjar/forjar.yaml # Embedded configuration
```

### Boot-time convergence

`service.sh` runs at `sys.boot_completed=1`, checks for
`/data/forjar/.firstboot-done`, runs `forjar apply`, and creates
the marker on success.

### Limitations

- Experimental (P2 priority)
- Requires rooted device with Magisk
- Binary stub must be replaced with cross-compiled aarch64 binary
- No `adb` transport yet (SSH via Termux is the current path)

## Integration Map

```
Bootstrap (FJ-49)          Image (FJ-52/54)
    │                           │
    │  SSH key + sudo           │  Autoinstall ISO / Magisk
    │                           │
    ▼                           ▼
Machine ready ──────────► forjar apply
                               │
                               ├── Build (FJ-33): cross-compile + deploy
                               ├── Package (FJ-51): cargo cache acceleration
                               └── All other resource types
```

The provisioning pipeline: `image` pre-configures machines, `bootstrap`
handles existing machines, `apply` converges to desired state, `build`
enables cross-compilation, and `cargo cache` accelerates repeated
deployments.
