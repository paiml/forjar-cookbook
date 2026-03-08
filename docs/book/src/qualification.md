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
  ForjarScore computed     <- 8-dimension quality grade
        |
        v
  Mark QUALIFIED in CSV    <- cookbook-runner updates CSV + score
        |
        v
  cookbook-readme-sync      <- README table regenerated with grades
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

## CLI Commands

```
cookbook-runner validate --file <recipe.yaml>     # Parse + plan
cookbook-runner qualify  --file <recipe.yaml>      # Full cycle with scoring
cookbook-runner score    --file <recipe.yaml>      # Static-only analysis
```

The `score` command analyzes SAF/OBS/DOC/RES/CMP without running apply.
The `qualify` command runs the full cycle and computes all 8 dimensions.

## Forjar Score (v2)

Every recipe receives a **Forjar Score** — a two-tier quality grade. Pending and blocked recipes get a static-only grade; qualified recipes get both static and runtime grades.

### Static Dimensions (Always Available)

| Code | Weight | What It Measures |
|------|--------|------------------|
| SAF | 25% | Explicit modes/owners, version pins, no curl\|bash, no plaintext secrets |
| OBS | 20% | Tripwire, lock_file, outputs, notify hooks |
| DOC | 15% | Header metadata, unique comments, descriptions |
| RES | 20% | Failure policy, DAG or tagged independence, deny_paths |
| CMP | 20% | Params, templates, includes, tags, resource_groups |

### Runtime Dimensions (After Qualification)

| Code | Weight | What It Measures |
|------|--------|------------------|
| COR | 35% | Validate + plan + apply + convergence |
| IDM | 35% | Second apply zero-change + hash stability |
| PRF | 30% | Apply time vs budget, idempotent apply ≤ 2s |

### Grades

| Grade | Composite | Min Dimension | Meaning |
|-------|-----------|---------------|---------|
| A | ≥ 90 | ≥ 80 | Production-hardened |
| B | ≥ 75 | ≥ 60 | Solid, minor gaps |
| C | ≥ 60 | ≥ 40 | Functional but rough |
| D | ≥ 40 | — | Bare minimum |
| F | < 40 | — | Blocked or never-qualified |

Overall grade = `min(static_grade, runtime_grade)`. Format: `B/A` (static B, runtime A), `A/pending` (runtime not yet tested).

### Current Status

56 qualified recipes hold **B-grade** (composite 80-89). The 11 pending recipes (88-98) have static grades with runtime pending.
