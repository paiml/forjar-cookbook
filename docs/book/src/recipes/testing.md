# Testing Recipes

Patterns for validating infrastructure recipes using forjar's
convergence, mutation, and behavior testing modes.

## Convergence Testing

Verify that applying a recipe twice produces the same result.
The convergence runner creates isolated tempdir sandboxes, runs
scripts via `bash -euo pipefail`, and verifies convergence,
idempotency, and preservation in parallel.

```bash
forjar test --group convergence -f forjar.yaml
```

Output:

```
Convergence Verification (mode: simulated)
==========================================
Backend: pepita (mode: simulated)

  [PASS] nginx-config/file: converge=true idem=true preserve=true (18ms)
  [PASS] app-config/file: converge=true idem=true preserve=true (15ms)
  [PASS] db-config/file: converge=true idem=true preserve=true (14ms)

Convergence: 3/3 passed (100%)
```

Each test creates real files in `$FORJAR_SANDBOX`, executes
apply/query scripts, and verifies state matches across runs.

## Mutation Testing

Verify that forjar detects all infrastructure drift scenarios.
Eight mutation operators simulate common drift. File-scoped
operators (delete, modify, chmod, corrupt) run in local sandboxes.
System operators (stop service, remove package) require a
container backend.

```bash
forjar test --group mutation -f forjar.yaml
```

Output:

```
Mutation Score: 80% (Grade B)
  8/10 detected, 0 survived, 2 errored
  file: 8/8 detected (100%)
  service: 0/2 errored (requires container backend)

Safety: system operators rejected in local mode:
  stop_service: requires container backend
  kill_process: requires container backend
```

With Docker available, system operators run in ephemeral
containers and the score improves to Grade A.

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

## Resource Coverage Report

Check which resources have behavior specs and check scripts:

```bash
forjar test --group coverage -f forjar.yaml
```

Output shows per-resource coverage levels (L0 = no tests, L1 = check script, L2 = behavior spec):

```
Resource Coverage Report
========================
  nginx-pkg: L1 (package)
  nginx-config: L2 (file)
  firewall-rule: L0 (network)
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
  Mutation:    8/10 detected (grade B, 2 need container)
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
