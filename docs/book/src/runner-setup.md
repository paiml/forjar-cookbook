# Runner Setup

## Intel Runner (Primary)

The self-hosted Intel runner is the primary qualification target.
GitHub Actions `ubuntu-latest` handles code quality (fmt, clippy,
test, coverage), but real qualification happens on bare metal.

| Component | Spec |
|-----------|------|
| OS | Ubuntu 22.04 (Jammy), kernel 6.8.0-101-generic |
| CPU | 32-core Intel Xeon W |
| RAM | 283 GB |
| Storage | 3.6 TB NVMe RAID-0 |
| GPU | 2x AMD Radeon Pro W5700X (Navi 10) |
| Docker | 29.2.1 |
| GPU devices | `/dev/kfd`, `/dev/dri/renderD128`, `/dev/dri/renderD129` |

### Why Self-Hosted

| Concern | GitHub Actions | Self-Hosted |
|---------|---------------|-------------|
| Systemd | Stubbed | Real |
| apt persistence | Ephemeral | Persists |
| Firewall (ufw) | No enforcement | Real |
| GPU | None | 2x AMD |
| NFS | Impossible | Real |
| Docker | DinD hacks | Native daemon |

## Connecting

```bash
ssh intel
cd ~/src/forjar-cookbook
```

## Qualification Workflow

```bash
# Qualify a single recipe
make qualify-recipe RECIPE=01

# Qualify all recipes
make qualify-all

# Score a recipe (static only, no apply)
make score-recipe RECIPE=01

# Score all recipes and update CSV
make score

# Update README dashboard from CSV
make update-qualifications
```

## Installing Forjar

The runner needs the latest forjar binary:

```bash
cd ~/src/forjar
cargo install --path .
forjar --version
```
