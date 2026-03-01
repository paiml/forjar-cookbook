# Qualification Process

## The Cycle

Every recipe goes through a systematic qualification cycle:

```
Recipe YAML written
        |
        v
  forjar validate          <- Tier 1: does it parse?
        |
        v
  forjar plan              <- Tier 1: does the DAG resolve?
        |
        v
  forjar apply (container) <- Tier 2: does it converge in a container?
        |
        v
  forjar apply (runner)    <- Tier 3: does it converge on bare metal?
        |
        v
  Second apply = 0 changes <- Idempotency proven
        |
        v
  Timing within budget     <- Performance proven
        |
        v
  Mark QUALIFIED in CSV    <- cookbook-runner updates CSV
        |
        v
  cookbook-readme-sync      <- README table regenerated
```

## When a Recipe Fails

If a recipe fails because forjar is missing a feature or has a bug:

1. Mark **BLOCKED** in CSV with `blocker_ticket` and `blocker_description`
2. File the issue in the forjar repo
3. Implement the fix in forjar
4. Bump forjar version
5. Re-run the recipe
6. Mark **QUALIFIED** when it passes

## Idempotency Contract

Every recipe must satisfy:

- Apply #1: converge from clean state (N changes)
- Apply #2: re-apply immediately (0 changes, exit 0)
- State hash #1 == state hash #2

## Forjar Score

Every qualified recipe receives a **Forjar Score** — a multi-dimensional quality grade from A through F.

### Dimensions

| Code | Weight | What It Measures |
|------|--------|------------------|
| COR | 20% | Converges from clean state, all resources converged |
| IDM | 20% | Zero changes on re-apply, stable state hashes |
| PRF | 15% | Within time budget, fast idempotent re-apply |
| SAF | 15% | No dangerous patterns (0777, curl\|bash, missing mode/owner) |
| OBS | 10% | Tripwire, lock file, outputs, notify hooks, mode/owner coverage |
| DOC | 8% | Comment ratio, header metadata, description field |
| RES | 7% | Failure policy, dependency DAG ratio, lifecycle hooks |
| CMP | 5% | Params, templates, includes, tags, resource groups |

### Grades

| Grade | Composite | Min Dimension | Meaning |
|-------|-----------|---------------|---------|
| A | >= 90 | >= 80 | Production-hardened |
| B | >= 75 | >= 60 | Solid, minor gaps |
| C | >= 60 | >= 40 | Functional but rough |
| D | >= 40 | any | Bare minimum |
| F | < 40 | any | Blocked or never-qualified |

### Current Status

All 56 qualified recipes achieve **A-grade** (score 90-95, all dimensions >= 80).
