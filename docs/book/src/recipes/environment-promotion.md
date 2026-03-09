# Environment Promotion Recipes

Recipes for multi-environment workflows using forjar's environment
promotion pipelines (FJ-3500).

## Dev/Staging/Prod Pipeline

```yaml
version: "1.0"
name: web-platform
machines:
  web:
    hostname: web-01
    addr: 10.0.1.10
params:
  log_level: debug
  replicas: 1
resources:
  app:
    type: package
    machine: web
    provider: apt
    packages: [nginx]

environments:
  dev:
    description: "Development"
    params:
      log_level: debug
      replicas: 1
    machines:
      web:
        addr: dev-web.internal

  staging:
    description: "Staging"
    params:
      log_level: info
      replicas: 2
    machines:
      web:
        addr: staging-web.internal
    promotion:
      from: dev
      gates:
        - validate: { deep: true }
        - script: "curl -sf http://dev-web/health"
      auto_approve: true

  production:
    description: "Production"
    params:
      log_level: warn
      replicas: 5
    machines:
      web:
        addr: prod-web.internal
    promotion:
      from: staging
      gates:
        - validate: { deep: true }
        - policy: { strict: true }
        - coverage: { min: 90 }
        - script: "run-integration-tests.sh"
      auto_approve: false
      rollout:
        strategy: canary
        canary_count: 1
        health_check: "curl -sf http://localhost/health"
        health_timeout: "30s"
        percentage_steps: [10, 25, 50, 100]
```

## Applying to an Environment

```bash
# Apply to dev (default)
forjar apply --env-name dev

# Apply to staging with environment-scoped state
forjar apply --env-name staging

# Apply to production
forjar apply --env-name production
```

## Listing Environments

```bash
forjar environments list
forjar environments list --json
```

## Diffing Environments

```bash
forjar environments diff dev staging
forjar environments diff staging production --json
```

## Progressive Rollout

Production promotions support progressive rollout:

1. **Canary**: Deploy to 1 machine, run health check
2. **Percentage steps**: 10% → 25% → 50% → 100%
3. **Auto-rollback**: If health check fails, revert

## Example

```bash
cargo run --example environments
cargo run --example promotion_gates
```
