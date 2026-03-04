# Operational Intelligence Recipes

Pre-apply quality gates that analyze configuration complexity, blast radius,
and drift risk — blocking deployment when thresholds are exceeded.

## #75 Complexity Gate

Pre-apply complexity gate: validates that config complexity stays within
acceptable bounds (Grade C or better) before allowing apply to proceed.
Uses `forjar complexity` to score resource count, DAG depth, cross-machine
dependencies, template usage, and conditional expressions.

**Tier**: 1+2 | **Idempotency**: Strong

```bash
# Gate check — blocks if grade is D or F
forjar complexity -f forjar.yaml --json | grep -q '"grade":"[ABC]"'
```

## #76 Impact Gate

Pre-apply impact analysis gate: validates that no single resource change has
critical blast radius. Uses `forjar impact` to compute affected resources,
machine spread, and estimated cascade time.

**Tier**: 1+2 | **Idempotency**: Strong

```bash
# Gate check — blocks if any resource has critical risk
forjar impact -f forjar.yaml -r db-pkg --json | grep -vq '"critical"'
```

## #77 Drift Prediction Alert

Drift prediction monitoring: analyzes historical event logs to identify
resources most likely to drift. Ranks resources by drift rate, trend
(increasing/decreasing/stable), and mean time between drifts.

**Tier**: 1+2 | **Idempotency**: Strong

```bash
# Show top 5 drift-prone resources
forjar drift-predict --state-dir state/ --limit 5
```
