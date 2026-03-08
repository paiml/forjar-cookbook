# Quality Scoring (ForjarScore v2)

Every cookbook recipe is scored on two axes: **static** (design quality) and **runtime** (operational quality).

## How Scoring Works

```bash
# Score a recipe
cookbook-runner score --file recipes/92-cis-hardening.yaml

# Grade format: Static/Runtime
# A/A     — excellent design + excellent runtime
# A/pending — excellent design, not yet runtime-tested
# B/F     — good design, runtime failures
```

## Static Dimensions (Always Available)

| Dimension | Weight | What It Measures |
|-----------|--------|------------------|
| **SAF** Safety | 25% | Explicit modes/owners, version pins, no curl\|bash, no plaintext secrets |
| **OBS** Observability | 20% | Tripwire, lock_file, outputs, notify hooks |
| **DOC** Documentation | 15% | Header metadata, unique comments, descriptions |
| **RES** Resilience | 20% | Failure policy, DAG or tagged independence, deny_paths |
| **CMP** Composability | 20% | Params, templates, includes, tags, resource_groups |

## Runtime Dimensions (After Qualification)

| Dimension | Weight | What It Measures |
|-----------|--------|------------------|
| **COR** Correctness | 35% | Validate + plan + apply + convergence |
| **IDM** Idempotency | 35% | Second apply zero-change + hash stability |
| **PRF** Performance | 30% | Apply time vs budget, idempotent apply ≤ 2s |

## Grade Thresholds

| Grade | Composite | Min Dimension |
|-------|-----------|---------------|
| A | ≥ 90 | ≥ 80 |
| B | ≥ 75 | ≥ 60 |
| C | ≥ 60 | ≥ 40 |
| D | ≥ 40 | — |
| F | < 40 | — |

Overall = `min(static_grade, runtime_grade)`.

## Writing A-Grade Recipes

To achieve static grade A, ensure:

1. **All file resources** have explicit `mode` and `owner`
2. **All packages** have `version: "latest"` (or pinned)
3. **No curl|bash** patterns — download then execute separately
4. **Secrets** use `{{ secrets.* }}` templates, never plaintext
5. **Policy** includes `tripwire: true`, `lock_file: true`, notify hooks
6. **Header comments** include `Recipe:`, `Tier:`, `Idempotency:`, `Budget:`
7. **Resources** have `tags` and `resource_group` for selective application
8. **Include** shared policy defaults: `includes/policy-defaults.yaml`

## Current Scores

56 qualified recipes hold **B-grade** (composite 80-89, min dimension RES=61 or CMP=65). The 5 blocked recipes score C-grade static-only (36-41). The 11 pending recipes (88-98) score C/D-grade static-only (32-41) with runtime pending.
