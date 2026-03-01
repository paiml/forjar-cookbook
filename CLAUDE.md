# Forjar Cookbook Development Guidelines

## Purpose

This repo is a **qualification suite** for forjar. Every recipe is a real
forjar config that gets applied to real machines. When a recipe exposes a
forjar bug or missing feature:

1. Stop — record the gap in `docs/certifications/recipes.csv`
2. Implement — fix the bug in the `forjar` repo
3. Retry — re-qualify the recipe
4. Dashboard — `make update-qualifications` regenerates README

## Code Search

NEVER use grep/glob for code search. ALWAYS prefer `pmat query`.

## Quality Gates

- **95% minimum test coverage** (`cargo llvm-cov --workspace --lib`)
- Zero clippy warnings (`cargo clippy -- -D warnings`)
- `pmat comply check` — all files pass
- `bashrs lint scripts/ Makefile` — all shell linted
- No source file over 500 lines
- Pre-commit hooks enforce complexity (cyclomatic 30, cognitive 25)
- Never use `cargo tarpaulin` — use `cargo llvm-cov`

## Testing

```bash
cargo test                              # Run all tests
make check                              # Full gate chain
make coverage-check                     # Verify >= 95%
make examples                           # Validate + plan all recipes
```

## Self-Hosted Runner

The Intel runner is the **primary** qualification target:
- `ssh intel` — 32-core Xeon, 283 GB RAM, 2x AMD GPU
- GitHub Actions is secondary (code quality only)
- `make qualify-all` runs the full qualification suite on the runner

## Key Targets

```bash
make qualify-recipe RECIPE=01           # Qualify one recipe
make qualify-all                        # Qualify all recipes on runner
make update-qualifications              # Regenerate README table from CSV
make docs-check                         # Verify README ↔ CSV consistency
```
