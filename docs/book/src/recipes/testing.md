# Testing Recipes

Patterns for validating infrastructure recipes using forjar's
convergence, mutation, and behavior testing modes.

## Convergence Testing

Verify that applying a recipe twice produces the same result.
The convergence runner builds targets from your config and tests
each resource in parallel.

```bash
forjar test --group convergence -f forjar.yaml
```

Output:

```
Convergence Test Runner (simulated)
===================================
Stack: my-stack (5 resources)

Targets: 5 resources

  [PASS] nginx-pkg/package: converge=true idem=true preserve=true
  [PASS] nginx-config/file: converge=true idem=true preserve=true
  [PASS] nginx-svc/service: converge=true idem=true preserve=true

Convergence: 5/5 passed (100%)
```

## Mutation Testing

Verify that forjar detects all infrastructure drift scenarios.
Eight mutation operators simulate common drift (file deletion,
content change, permission change, service stop, package removal).

```bash
forjar test --group mutation -f forjar.yaml
```

Output:

```
Mutation Test Runner (mode: Simulated)
====================
Stack: my-stack (5 resources)

Mutation Score: 100% (Grade A)
  7/7 detected, 0 survived, 0 errored

  file: 4/4 detected (100%)
  package: 1/1 detected (100%)
  service: 2/2 detected (100%)
```

### Mutation Score Grades

| Grade | Score | Meaning |
|-------|-------|---------|
| A | >= 90% | All mutations detected |
| B | >= 80% | Most mutations detected |
| C | >= 60% | Significant gaps |
| F | < 60% | Drift detection broken |

## Behavior Specs

Behavior-driven testing describes what the system should look like
after convergence. Create `.spec.yaml` files alongside your config.

```yaml
# nginx.spec.yaml
name: nginx web server
config: forjar.yaml
behaviors:
  - name: nginx is installed
    state: present
  - name: port 80 is open
    state: listening
  - name: config is valid
    state: verified
```

```bash
forjar test --group behavior -f forjar.yaml
```

## Unified Test Runner

Run all three modes with a single command:

```bash
cargo run --example test_suite
```

This produces a combined report:

```
=== Combined Test Report ===
  Convergence: 3/3 passed (100%)
  Mutation:    7/7 detected (grade A)
  Behavior:    3/3 passed
  Overall: PASS
```

## Recipe Qualification

Every cookbook recipe must pass all three testing modes before
qualification. The scorer checks:

- COR (20%): Convergence and idempotency
- IDM (20%): Idempotent apply (zero changes on second run)
- PRF (15%): Performance within budget
- SAF (15%): No unsafe patterns (curl|bash, chmod 777)
- OBS (10%): Observability hooks (on_success, on_failure)

Recipes that pass all quality gates receive an A grade.

## CI Integration

Add testing to your CI workflow:

```yaml
# .github/workflows/test.yml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Convergence tests
        run: forjar test --group convergence -f forjar.yaml
      - name: Mutation tests
        run: forjar test --group mutation -f forjar.yaml
      - name: Behavior tests
        run: forjar test --group behavior -f forjar.yaml
```
