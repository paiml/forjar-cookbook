# Query Enrichments

Forjar's state-query engine provides operational intelligence over your infrastructure. These cookbook patterns show how to use enrichment flags for debugging, monitoring, and compliance.

## Debugging a Failed Apply

When an apply fails, use `--failures` with `--since` to narrow down the timeline:

```bash
# What failed in the last hour?
forjar state-query --failures --since 1h

# Show the full event log for the failed run
forjar state-query --events --run run-007

# Check if the resource has a pattern of failures
forjar state-query "nginx-pkg" --history
```

## Drift Monitoring

Combine `--drift` with `--churn` to find resources that both drift frequently and change often — likely candidates for configuration management improvements:

```bash
# Find drifted resources
forjar state-query --drift

# Find high-churn resources (changing too often)
forjar state-query --churn

# Export drift data for monitoring dashboards
forjar state-query --drift --json | jq '.[] | {resource: .resource_id, machine}'
```

## Health Dashboard

Use `--health` for a quick stack-wide overview, suitable for CI gates or monitoring scripts:

```bash
# Quick health check
forjar state-query --health

# Machine-parseable for CI
forjar state-query --health --json

# Fail CI if health is below threshold
HEALTH=$(forjar state-query --health --json | jq '.health_pct')
if (( $(echo "$HEALTH < 95" | bc -l) )); then
  echo "Stack health below 95%: ${HEALTH}%"
  exit 1
fi
```

## Status Filtering

Filter resources by convergence status for targeted investigation:

```bash
# Only show failed resources
forjar state-query --status failed

# Only converged resources (for compliance reports)
forjar state-query --status converged --json

# Drifted resources on a specific machine
forjar state-query --status drifted --resource-type file
```

## Time-Windowed Reports

The `--since` flag accepts relative durations (`1h`, `7d`, `30m`) or ISO 8601 timestamps:

```bash
# Daily report: events in last 24 hours
forjar state-query --events --since 1d

# Weekly failure report
forjar state-query --failures --since 7d --json > weekly-failures.json

# Events since a specific deploy
forjar state-query --events --since 2026-03-01T00:00:00
```

## Git History Fusion

Use `-G` to search by intent — the commit messages that introduced resources:

```bash
# Find resources related to a past fix
forjar state-query "fix memory leak" -G

# Find resources introduced for security hardening
forjar state-query "CVE" -G
```
