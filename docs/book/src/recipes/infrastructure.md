# Infrastructure Recipes

Core server provisioning recipes that form the foundation of the cookbook.
These recipes cover the most common infrastructure patterns: developer
workstations, web servers, databases, caching, monitoring, and security.

## #1 Developer Workstation

Provisions a development machine with build tools, dotfiles, directory
structure, and shell configuration. This is the first cookbook recipe and
validates the package, file, and user resource types end-to-end.

**Resources**: dev-packages (apt), dev-user (user), home-dir (file),
gitconfig (file), vimrc (file), tmux-conf (file), shell-rc (file)

**Tier**: 2+3 | **Idempotency**: Strong | **Grade**: A

## #2 Web Application Server

Nginx reverse proxy with TLS directory structure, site configuration,
and firewall rules. Validates the service resource type with systemd
integration and network/firewall resources.

**Resources**: nginx-pkg (apt), site-config (file), tls-dir (file),
nginx-service (service), firewall-http (network)

**Tier**: 2+3 | **Idempotency**: Strong | **Grade**: A

## #3 PostgreSQL Database

PostgreSQL installation with data directory, authentication config,
and connection tuning. Tests the service resource lifecycle with a
stateful application.

**Resources**: pg-packages (apt), pg-config (file), pg-hba (file),
pg-data-dir (file), pg-service (service)

**Tier**: 2+3 | **Idempotency**: Strong | **Grade**: A

## #4 Monitoring Stack

Prometheus and Grafana with scrape targets, dashboards, and alerting
rules. Tests multi-service orchestration and configuration file
management.

**Resources**: monitoring-packages (apt), prometheus-config (file),
grafana-provisioning (file), prometheus-dir (file), grafana-dir (file)

**Tier**: 2+3 | **Idempotency**: Strong | **Grade**: A

## #5 Redis Cache

Redis installation with persistence config, memory limits, and
eviction policy. Tests service configuration with custom settings
and sysctl tuning for production workloads.

**Resources**: redis-pkg (apt), redis-config (file), redis-data-dir (file),
redis-service (service)

**Tier**: 2+3 | **Idempotency**: Strong | **Grade**: A

## #6 CI Runner

Self-hosted CI runner with Docker, workspace directories, and build
toolchain. Tests user creation, directory hierarchy, and service
management for continuous integration.

**Resources**: ci-packages (apt), runner-user (user), workspace-dir (file),
workspace-builds (file), workspace-cache (file), docker-service (service)

**Tier**: 2+3 | **Idempotency**: Strong | **Grade**: A

## #7 ROCm GPU

AMD ROCm userspace installation with kernel module verification and
GPU compute setup. Tests the GPU resource type with `gpu_backend: rocm`.

**Resources**: rocm-packages (apt), gpu-check (gpu), rocm-env (file)

**Tier**: 3 | **Idempotency**: Strong | **Blocked**: FJ-1126

## #8 NVIDIA GPU

NVIDIA CUDA toolkit and driver installation with GPU compute setup.
Tests the GPU resource type with `gpu_backend: nvidia`.

**Resources**: nvidia-packages (apt), gpu-check (gpu), cuda-env (file)

**Tier**: 3 | **Idempotency**: Strong | **Blocked**: FJ-1127

## #9 Secure Baseline

SSH hardening, fail2ban, firewall defaults, and unattended-upgrades.
Establishes a security baseline that other recipes can build upon.

**Resources**: security-pkgs (apt), sshd-config (file), fail2ban-config (file),
firewall-ssh (network), sshd-service (service), fail2ban-service (service)

**Tier**: 2+3 | **Idempotency**: Strong | **Grade**: A

## #10 NFS Server

NFS server with exports, directory structure, and service management.
Tests mount resource type and cross-machine file sharing patterns.

**Resources**: nfs-packages (apt), exports-config (file), share-dir (file),
nfs-service (service)

**Tier**: 3 | **Idempotency**: Strong | **Blocked**: FJ-1128

## #22 Secrets Lifecycle

Secret management lifecycle: key generation, encryption, deployment,
rotation, and audit. Uses age encryption for ENC[age,...] markers.
Tests the full secrets workflow from creation through rotation.

**Resources**: age-key-dir (file), secrets-encrypted (file),
rotation-script (file), audit-log (file)

**Tier**: 1+2 | **Idempotency**: Strong | **Blocked**: FJ-1129

## #23 TLS Certificates

TLS certificate management: self-signed generation for dev/test,
renewal scripting, expiry monitoring, and permission hardening.
Validates file ownership and mode enforcement for PKI artifacts.

**Resources**: cert-dir (file), self-signed-cert (file),
renewal-script (file), expiry-check (cron)

**Tier**: 1+2 | **Idempotency**: Strong | **Grade**: A

## #24 Fleet Provisioning

Fleet-scale provisioning: deploy base packages, node identity,
monitoring agent, and SSH keys to multiple machines from a single
config. Tests multi-machine orchestration at fleet scale.

**Resources**: base-packages (package), node-identity (file),
monitoring-agent (file), ssh-keys (file)

**Tier**: 1+2 | **Idempotency**: Strong | **Grade**: A

## Operator Authorization

Machines can restrict which operators are allowed to apply changes
using `allowed_operators`. When set, `forjar apply --operator NAME`
must match the list or the apply is rejected before execution.

```yaml
machines:
  production:
    hostname: prod.example.com
    addr: 10.0.1.10
    allowed_operators:
      - deploy-bot
      - ops-team
```

```bash
# Allowed — deploy-bot is in the list
forjar apply -f forjar.yaml --operator deploy-bot

# Denied — random-user is not authorized
forjar apply -f forjar.yaml --operator random-user
# Error: operator 'random-user' not authorized for machine 'production'
```

If `allowed_operators` is empty or omitted, all operators are permitted.
When `--operator` is not specified, identity is resolved from
`$USER@hostname` automatically.
