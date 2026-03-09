# Policy Enforcement Recipes

These recipes demonstrate how to use forjar's Policy-as-Code engine to
enforce compliance, security, and operational standards.

## Security Baseline

Enforce file ownership and permissions across all managed configs:

```yaml
version: "1.0"
name: secure-baseline
machines:
  web:
    hostname: web-01
    addr: 10.0.0.1
resources:
  nginx-conf:
    type: file
    machine: web
    path: /etc/nginx/nginx.conf
    owner: root
    mode: "0644"
    tags: [web, config]
  app-conf:
    type: file
    machine: web
    path: /etc/app/config.yaml
    owner: root
    mode: "0640"
    tags: [app, config]
policies:
  - type: assert
    id: SEC-001
    message: "config files must be owned by root"
    resource_type: file
    tag: config
    condition_field: owner
    condition_value: root
    severity: error
    remediation: "Set owner: root"
    compliance:
      - framework: cis
        control: "6.1.2"
  - type: require
    id: SEC-002
    message: "all files must have explicit permissions"
    resource_type: file
    field: mode
    severity: error
    remediation: "Add mode: '0644' or stricter"
```

## Package Hygiene

Keep package resources small for parallel installs and clear dependency
tracking:

```yaml
policies:
  - type: limit
    id: PERF-001
    message: "package resources should have < 5 packages"
    resource_type: package
    field: packages
    max_count: 5
    severity: warning
    remediation: "Split into role-based package groups"
```

## Tagging Standards

Require tags on all resources for filtering and audit:

```yaml
policies:
  - type: limit
    id: OPS-001
    message: "all resources must have at least 1 tag"
    field: tags
    min_count: 1
    severity: info
    remediation: "Add tags: [role, env] for filtering"
```

## CI Integration

Use JSON output in CI pipelines to gate deployments:

```bash
#!/bin/bash
result=$(forjar policy forjar.yaml --json)
passed=$(echo "$result" | jq -r '.passed')

if [ "$passed" = "false" ]; then
  echo "Policy check failed:"
  echo "$result" | jq '.violations[] | "\(.severity): [\(.policy_id)] \(.message)"'
  exit 1
fi
```

## Compliance Audit Trail

Map policies to frameworks for automated compliance reporting:

```yaml
policies:
  - type: assert
    id: SEC-010
    message: "SSH keys must be specified for all machines"
    condition_field: state
    condition_value: present
    compliance:
      - framework: cis
        control: "5.2.1"
      - framework: stig
        control: "V-238215"
      - framework: soc2
        control: "CC6.1"
```

Export JSON output to your compliance platform for continuous audit evidence.
