# Linux Administration Recipes

System administration recipes covering cron jobs, user management,
kernel tuning, logging, time sync, systemd units, resource limits,
automated patching, hostname/locale configuration, and swap management.

## #40 Scheduled Tasks (Cron)

Backup scripts with cron scheduling and log cleanup automation.
Validates the cron resource type with environment management and
proper script permissions.

**Resources**: backup-dir (file), backup-script (file),
backup-cron (cron), log-cleanup-script (file), log-cleanup-cron (cron)

**Tier**: 2+3 | **Idempotency**: Strong | **Grade**: A

## #41 User Provisioning

Bulk user creation with group membership, SSH keys, and quota
management. Tests the user resource type at scale with proper
permission hierarchies.

**Resources**: users (user), groups (user), ssh-keys (file),
quota-config (file)

**Tier**: 2+3 | **Idempotency**: Strong | **Grade**: A

## #42 Kernel Tuning

Sysctl parameter management for production workloads. Covers network
buffer sizes, file descriptor limits, memory overcommit policy, and
other kernel tunables.

**Resources**: sysctl-config (file), limits-config (file),
modprobe-config (file)

**Tier**: 3 | **Idempotency**: Strong | **Requires**: Bare-metal or VM

## #43 Log Management

Centralized logging configuration with logrotate, rsyslog, and log
directory structure. Manages log retention policies and forwarding
rules.

**Resources**: logrotate-config (file), rsyslog-config (file),
log-dirs (file)

**Tier**: 2+3 | **Idempotency**: Strong | **Grade**: A

## #44 Time Synchronization

NTP/chrony configuration for accurate timekeeping. Ensures all
machines in a fleet maintain synchronized clocks with configurable
NTP servers.

**Resources**: chrony-pkg (package), chrony-config (file),
chrony-service (service)

**Tier**: 2+3 | **Idempotency**: Strong | **Grade**: A

## #45 Custom Systemd Units

Create and manage custom systemd service units, timers, and socket
activations. Tests the service resource type with custom unit files
and dependency ordering.

**Resources**: unit-file (file), service-config (file),
timer-config (file), service (service)

**Tier**: 3 | **Idempotency**: Strong | **Requires**: Systemd host

## #46 Resource Limits

ulimit and cgroup-based resource limits for production processes.
Manages PAM limits, systemd slice configurations, and per-user
resource constraints.

**Resources**: limits-conf (file), systemd-slice (file),
pam-config (file)

**Tier**: 3 | **Idempotency**: Strong | **Grade**: A

## #47 Automated Patching

Unattended-upgrades with approval policies, blacklists, and reboot
scheduling. Manages the full automated patching lifecycle with
notification hooks.

**Resources**: unattended-upgrades (package), auto-upgrade-config (file),
blacklist (file), reboot-policy (file)

**Tier**: 2+3 | **Idempotency**: Strong | **Grade**: A

## #48 Hostname and Locale

System identity configuration: hostname, domain, locale, timezone,
and DNS resolver settings. Foundational system configuration that
other recipes depend on.

**Resources**: hostname (file), hosts (file), locale-config (file),
timezone (file), resolv-config (file)

**Tier**: 2+3 | **Idempotency**: Strong | **Grade**: A

## #49 Swap Memory

Swap file or partition management with configurable size, swappiness,
and priority. Tests file-based swap creation and sysctl tuning for
memory management.

**Resources**: swap-file (file), fstab-entry (file),
sysctl-swappiness (file)

**Tier**: 3 | **Idempotency**: Strong | **Requires**: Bare-metal or VM
