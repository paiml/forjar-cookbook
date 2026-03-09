# Rules Validation

Forjar's rulebook validation engine (FJ-3108) validates rulebook YAML
for correctness before deployment: event patterns, action completeness,
cooldown bounds, duplicate names, and event type coverage. These recipes
show how to integrate rulebook validation into CI pipelines and
compliance workflows.

## CI Pipeline Rulebook Validation

Gate deployments on rulebook correctness. The `forjar rules validate`
command checks event patterns, action completeness, cooldown bounds, and
duplicate names. Use `--json` for machine-parseable output in CI.

```yaml
# .github/workflows/rulebook-ci.yml
name: Rulebook Validation
on:
  pull_request:
    paths:
      - "rulebooks/**"
      - "forjar.yaml"

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install forjar
        run: cargo install forjar

      - name: Validate all rulebooks
        run: |
          exit_code=0
          for f in rulebooks/*.yaml; do
            echo "--- Validating $f ---"
            if ! forjar rules validate --file "$f"; then
              exit_code=1
            fi
          done
          exit $exit_code

      - name: Check event coverage
        run: |
          for f in rulebooks/*.yaml; do
            echo "--- Coverage: $f ---"
            forjar rules coverage --file "$f"
          done
```

### JSON Output for CI Gates

Use `--json` to parse validation results programmatically:

```bash
#!/bin/bash
set -euo pipefail

result=$(forjar rules validate --file rulebooks/production.yaml --json)
passed=$(echo "$result" | jq -r '.passed')
errors=$(echo "$result" | jq -r '.errors')

if [ "$passed" != "true" ]; then
  echo "Rulebook validation failed with $errors error(s):"
  echo "$result" | jq -r '.issues[] | "  [\(.severity)] \(.rulebook): \(.message)"'
  exit 1
fi

echo "Rulebook validation passed ($( echo "$result" | jq -r '.rulebook_count') rulebook(s))"
```

### Pre-Commit Hook

Validate rulebooks before every commit:

```bash
#!/bin/bash
# .git/hooks/pre-commit (or use pre-commit framework)
changed=$(git diff --cached --name-only --diff-filter=ACM | grep -E '\.yaml$' || true)
for f in $changed; do
  if head -5 "$f" | grep -q 'rulebooks:'; then
    echo "Validating rulebook: $f"
    forjar rules validate --file "$f" || exit 1
  fi
done
```

## Event Coverage Compliance Check

The `forjar rules coverage` command reports which event types are handled
by at least one rulebook. Use this to enforce that production deployments
cover critical event types.

```yaml
# rulebooks/production.yaml — full coverage example
rulebooks:
  - name: config-repair
    events:
      - type: file_changed
        match:
          path: /etc/nginx/nginx.conf
    actions:
      - apply:
          file: forjar.yaml
          tags: [config]
    cooldown_secs: 60

  - name: crash-recovery
    events:
      - type: process_exit
        match:
          process: myapp
          exit_code: "137"
    actions:
      - script: "systemctl restart myapp"
      - notify:
          channel: "https://hooks.slack.com/services/xxx"
          message: "myapp crashed on {{machine}} — OOM kill"
    cooldown_secs: 300
    max_retries: 3

  - name: scheduled-audit
    events:
      - type: cron_fired
    actions:
      - script: "forjar drift --tripwire -f forjar.yaml"
    cooldown_secs: 3600

  - name: deploy-hook
    events:
      - type: webhook_received
        match:
          ref: refs/heads/main
    actions:
      - script: "forjar apply -f forjar.yaml"

  - name: manual-ops
    events:
      - type: manual
    actions:
      - script: "forjar apply -f forjar.yaml --tags maintenance"
```

### Coverage Gate Script

Require minimum event type coverage before promotion:

```bash
#!/bin/bash
set -euo pipefail

# Get coverage as JSON
coverage=$(forjar rules coverage --file rulebooks/production.yaml --json)

# Required event types for production
required_types=("file_changed" "process_exit" "cron_fired")
missing=()

for etype in "${required_types[@]}"; do
  count=$(echo "$coverage" | jq -r --arg t "$etype" '.[$t] // 0')
  if [ "$count" = "0" ]; then
    missing+=("$etype")
  fi
done

if [ ${#missing[@]} -gt 0 ]; then
  echo "FAIL: missing required event types: ${missing[*]}"
  echo "Production rulebooks must handle: ${required_types[*]}"
  exit 1
fi

echo "Event coverage check passed"
```

### Coverage Report

```bash
$ forjar rules coverage --file rulebooks/production.yaml

Event Type Coverage
----------------------------------------
  [+] file_changed: 1 rulebook(s)
  [+] process_exit: 1 rulebook(s)
  [+] cron_fired: 1 rulebook(s)
  [+] webhook_received: 1 rulebook(s)
  [+] manual: 1 rulebook(s)
```

## Multi-Rulebook Config Validation

When a forjar config embeds multiple rulebooks, validate the entire
set for cross-rulebook consistency: no duplicate names, no conflicting
event patterns, and reasonable cooldowns.

```yaml
# forjar.yaml with inline rulebooks
version: "1.0"
name: multi-rulebook-stack
machines:
  web:
    hostname: web-01
    addr: 10.0.1.10

resources:
  nginx-config:
    type: file
    machine: web
    path: /etc/nginx/nginx.conf
    source: configs/nginx.conf
    tags: [config, nginx]

  app-binary:
    type: file
    machine: web
    path: /usr/local/bin/myapp
    source: bin/myapp
    mode: "0755"
    tags: [app, binary]

rulebooks:
  - name: config-repair
    events:
      - type: file_changed
        match:
          path: /etc/nginx/nginx.conf
    actions:
      - apply:
          file: forjar.yaml
          tags: [config]
    cooldown_secs: 60

  - name: app-watchdog
    events:
      - type: process_exit
        match:
          process: myapp
    actions:
      - script: "systemctl restart myapp"
      - notify:
          channel: "https://hooks.slack.com/services/xxx"
          message: "myapp restarted on {{machine}}"
    cooldown_secs: 120
    max_retries: 5

  - name: daily-drift-check
    events:
      - type: cron_fired
    actions:
      - script: "forjar drift --tripwire -f forjar.yaml"
      - notify:
          channel: "https://hooks.slack.com/services/xxx"
          message: "Daily drift report complete"
    cooldown_secs: 86400
```

### Validating Inline Rulebooks

```bash
# Validate the entire config (includes rulebook validation)
forjar validate -f forjar.yaml

# Validate just the rulebook section
forjar rules validate --file forjar.yaml
```

### Detecting Common Issues

The validation engine catches these problems:

| Issue | Severity | Description |
|-------|----------|-------------|
| Empty events list | Error | Rulebook has no event triggers |
| Empty actions list | Error | Rulebook has events but no actions |
| Duplicate name | Error | Two rulebooks share the same name |
| Zero cooldown | Warning | No cooldown risks rapid-fire execution |
| Excessive retries | Warning | `max_retries` > 10 may indicate a design issue |
| Empty apply file | Error | `apply:` action with blank file path |

```bash
# Example: validation catches a misconfigured rulebook
$ forjar rules validate --file bad-rules.yaml
Validating rulebooks in bad-rules.yaml
------------------------------------------------------------
3 rulebook(s), 3 error(s), 2 warning(s)
  [ERROR] empty-events: rulebook has no event patterns
  [ERROR] empty-events: duplicate rulebook name: empty-events
  [ERROR] rapid-fire: apply action has empty file path
  [WARN ] rapid-fire: cooldown_secs is 0 (no rate limiting)
  [WARN ] rapid-fire: max_retries (50) is unusually high

Validation FAILED.
```

### Combining with Policy Checks

Use rulebook validation alongside policy enforcement for comprehensive
config quality gates:

```bash
#!/bin/bash
set -euo pipefail

echo "=== Config Validation ==="
forjar validate -f forjar.yaml

echo "=== Policy Check ==="
forjar policy forjar.yaml

echo "=== Rulebook Validation ==="
forjar rules validate --file forjar.yaml

echo "=== Event Coverage ==="
forjar rules coverage --file forjar.yaml

echo "All checks passed"
```
