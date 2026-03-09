# Recipe Quality Scoring

ForjarScore v2 measures recipe quality on two tiers: **static** (design quality) and **runtime** (operational quality).

## Quick Start

```rust
use forjar::core::scoring::{compute, ScoringInput};
use forjar::core::parser;

let yaml = std::fs::read_to_string("recipe.yaml").unwrap();
let config = parser::parse_config(&yaml).unwrap();

let input = ScoringInput {
    status: "qualified".into(),
    idempotency: "strong".into(),
    budget_ms: 60_000,
    runtime: None,
    raw_yaml: Some(yaml),
};

let result = compute(&config, &input);
println!("Grade: {} (static {})", result.grade, result.static_composite);
```

## Static Dimensions (5)

| Dimension | Weight | Measures |
|-----------|--------|----------|
| SAF | 25% | Safety (mode:0777 caps at 40, curl\|bash is critical) |
| OBS | 20% | Observability (tripwire, lock_file, notify hooks) |
| DOC | 15% | Documentation quality (unique comments, not volume) |
| RES | 20% | Resilience (failure policy, retries, deny_paths) |
| CMP | 20% | Composability (params, templates, tags, includes) |

## Runtime Dimensions (3)

| Dimension | Weight | Measures |
|-----------|--------|----------|
| COR | 35% | Correctness (validate → plan → apply → converge) |
| IDM | 35% | Idempotency (zero changes on re-apply, hash stability) |
| PRF | 30% | Performance (% of budget consumed, idempotent speed) |

## Grade Thresholds

```
A: composite ≥ 90, min dimension ≥ 80
B: composite ≥ 75, min dimension ≥ 60
C: composite ≥ 60, min dimension ≥ 40
D: composite ≥ 40
F: below 40
```

## Falsification (FJ-2803)

Each dimension has explicit Popperian rejection criteria:

```bash
cargo run --example scoring_falsification
```

Key invariants verified:
- **SAF**: mode:0777 → SAF ≤ 40; no files → SAF = 100
- **OBS**: Full policy → OBS ≥ 90; disabled → OBS ≤ 15
- **COR**: Full convergence → COR ≥ 90; nothing → COR = 0
- **IDM**: Strong + zero changes → IDM ≥ 90; 3 changes → IDM < 50
- **PRF**: 33% of budget → PRF ≥ 70; 200% → PRF ≤ 25
- **Monotonicity**: Adding `deny_paths` never decreases score

## Scoring All Recipes

```bash
cargo run --example score_cookbook
```

Iterates over every recipe in `examples/cookbook/`, validates, and scores. Useful for auditing the full cookbook.
