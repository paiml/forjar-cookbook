# Operational Intelligence Recipes

Pre-apply quality gates that analyze configuration complexity, blast radius,
and drift risk — blocking deployment when thresholds are exceeded.

## #75 Complexity Gate

Pre-apply complexity gate: validates that config complexity stays within
acceptable bounds (Grade C or better) before allowing apply to proceed.
Uses `forjar complexity` to score resource count, DAG depth, cross-machine
dependencies, template usage, and conditional expressions.

**Tier**: 1+2 | **Idempotency**: Strong | **Grade**: A

```bash
# Gate check — blocks if grade is D or F
forjar complexity -f forjar.yaml --json | jq -e '.grade | test("[ABC]")'

# Full complexity report
forjar complexity -f forjar.yaml
```

Scoring dimensions: resources (1x, cap 30), DAG depth (5x, cap 20),
cross-machine deps (3x, cap 15), templates (2x, cap 10), conditionals
(2x, cap 10), includes (3x, cap 10), machines (2x, cap 5).

## #76 Impact Gate

Pre-apply impact analysis gate: validates that no single resource change has
critical blast radius. Uses `forjar impact` to compute affected resources,
machine spread, and estimated cascade time.

**Tier**: 1+2 | **Idempotency**: Strong | **Grade**: A

```bash
# Gate check — blocks if risk is critical
forjar impact -f forjar.yaml -r db-pkg --json | jq -e '.risk != "critical"'

# Full impact report
forjar impact -f forjar.yaml -r db-pkg
```

Risk levels: none (0 affected), low (1-3), medium (4-10), high (11-25),
critical (25+).

## #77 Drift Prediction Alert

Drift prediction monitoring: analyzes historical event logs to identify
resources most likely to drift. Ranks resources by drift rate, trend
(increasing/decreasing/stable), and mean time between drifts.

**Tier**: 1+2 | **Idempotency**: Strong | **Grade**: A

```bash
# Top 5 drift-prone resources
forjar drift-predict --state-dir state/ --limit 5

# Filter by machine
forjar drift-predict --state-dir state/ --machine web

# JSON for alerting integration
forjar drift-predict --state-dir state/ --json
```

Risk algorithm: `risk = min(1.0, (drift_rate * 0.5 + min(0.3, drift_count * 0.05)) * trend_multiplier)` where trend_multiplier is 1.3 (increasing), 0.7 (decreasing), 1.0 (stable).

## #78 Security Scan Gate

Static security analysis gate for CI/CD pipelines. Blocks deployment
if findings exceed configured severity threshold.

**Tier**: 1 | **Idempotency**: Strong | **Grade**: A

```bash
# Full scan
forjar security-scan -f forjar.yaml

# Gate: block on critical findings
forjar security-scan -f forjar.yaml --json | \
  jq -e '.findings | map(select(.severity == "critical")) | length == 0'
```

Checks: root-owned writable files (SS-1), services without limits (SS-2),
world-readable sensitive files (SS-3), external content without integrity (SS-4),
unencrypted secrets (SS-5).

## #79 Convergence Proof

Generate convergence proofs and SLSA provenance attestation for compliance.

**Tier**: 1 | **Idempotency**: Strong | **Grade**: A

```bash
# Prove convergence properties
forjar prove -f forjar.yaml

# SLSA provenance attestation
forjar provenance -f forjar.yaml --json > slsa.json

# Merkle DAG lineage
forjar lineage -f forjar.yaml --json > lineage.json

# Gate: all proofs must pass
forjar prove -f forjar.yaml --json | jq -e '.passed == .total'
```

Five proof obligations: codegen-completeness, dag-acyclicity, state-coverage,
hash-determinism, idempotency-structure.

## #80 Cost Estimation

Estimate apply time and resource requirements before committing.

**Tier**: 1 | **Idempotency**: Strong | **Grade**: A

```bash
# Estimate apply cost
forjar cost-estimate -f forjar.yaml

# JSON for CI budgeting
forjar cost-estimate -f forjar.yaml --json

# Combine with impact for risk assessment
forjar cost-estimate -f forjar.yaml --json > cost.json
forjar impact -f forjar.yaml -r critical-pkg --json > impact.json
```

Estimates based on static analysis of resource types (Package ~30s, File ~2s,
Service ~10s) and dependency chain length.
