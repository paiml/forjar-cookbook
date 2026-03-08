# High Availability Recipes

Recipes for redundant infrastructure: load balancing, database
replication, container orchestration, and deployment strategies.

## 89: PostgreSQL Primary/Replica

Streaming replication with WAL archiving, pg_hba.conf for
replication ACL, and health monitoring via `pg_stat_replication`.

```yaml
resources:
  pg-conf-primary:
    type: file
    path: /etc/postgresql/{{params.pg_version}}/main/postgresql.conf
    content: |
      wal_level = replica
      max_wal_senders = 5
      wal_keep_size = 256MB
```

Two machines (primary + replica) with dependency ordering:
packages -> config -> replication slot -> service -> health check.

## 90: HAProxy + Keepalived

Active/passive load balancer with VRRP failover:

- HAProxy frontend (HTTP/HTTPS) with backend server pool
- Keepalived VRRP for virtual IP failover between LB nodes
- Stats page with authentication on port 8404
- VRRP firewall via iptables script (IP protocol 112)

```bash
forjar apply -f recipes/90-haproxy-ha.yaml
```

## 91: Podman Rootless Containers

Rootless container deployment without Docker daemon:
subuid/subgid configuration, systemd user units, auto-update
timer for registry-based container updates.

## 92: CIS Benchmark Level 1

Automated CIS controls with tagged resource groups:
- `[cis, audit]` — auditd rules for file access monitoring
- `[cis, kernel]` — sysctl hardening (network, kernel)
- `[cis, auth]` — login.defs, PAM password quality
- `[cis, filesystem]` — permission hardening, modprobe blacklist

Apply selective controls: `forjar apply -f recipes/92-cis-hardening.yaml --tag audit`

## 94: Canary Deployment

Blue/green deployment with health gate:

1. Deploy to blue and green slots
2. Health gate script checks endpoint before traffic switch
3. Traffic switch updates nginx backend configuration
4. Rollback script reverts on failure

Sequential dependency chain ensures gate enforcement.
