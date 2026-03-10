# Forjar Cookbook — Qualification Suite
# Primary target: self-hosted Intel runner (bare-metal)

.SUFFIXES:
.DELETE_ON_ERROR:
.ONESHELL:

.PHONY: all build test lint fmt fmt-check check coverage coverage-check \
        validate-recipes examples docs-check update-qualifications \
        qualify-recipe qualify-all score score-recipe book bashrs-lint \
        clean release help

# Default target
all: check

# Build all crates
build:
	cargo build --workspace

# Run all tests
test:
	cargo test --workspace

# Run clippy lints
lint:
	cargo clippy --workspace --all-targets -- -D warnings

# Format code
fmt:
	cargo fmt --all

# Check formatting
fmt-check:
	cargo fmt --all -- --check

# Full quality gate chain
check: fmt-check lint test docs-check validate-recipes

# Coverage with llvm-cov (library code only)
coverage:
	cargo llvm-cov --workspace --lib --html
	@echo "Coverage report: target/llvm-cov/html/index.html"

# Coverage summary only
coverage-summary:
	cargo llvm-cov --workspace --lib --summary-only

# Coverage with threshold check (>= 95%)
coverage-check:
	@./scripts/coverage-check.sh

# Validate all recipe YAML files (blocks broken YAML from being committed)
validate-recipes:
	cargo run --example validate_all
	cargo run --example plan_all

# Run examples (validate + plan + score all recipes)
examples:
	cargo run --example validate_all
	cargo run --example plan_all
	cargo run --example score_all

# Documentation consistency check
docs-check:
	@./scripts/check-docs-consistency.sh

# Update README qualification table from CSV
update-qualifications:
	@echo "Updating README qualification table from CSV..."
	cargo run --bin cookbook-readme-sync --quiet

# Qualify a single recipe (usage: make qualify-recipe RECIPE=01)
qualify-recipe:
	@test -n "$(RECIPE)" || (echo "Usage: make qualify-recipe RECIPE=01"; exit 1)
	cargo run --bin cookbook-runner -- qualify -f recipes/$(RECIPE)*.yaml

# Score all recipes (static analysis, updates CSV)
score:
	cargo run --example score_all

# Score a single recipe (usage: make score-recipe RECIPE=01)
# Static-only: shows SAF/OBS/DOC/RES/CMP (COR/IDM/PRF need runtime data)
score-recipe:
	@test -n "$(RECIPE)" || (echo "Usage: make score-recipe RECIPE=01"; exit 1)
	-cargo run --bin cookbook-runner -- score -f recipes/$(RECIPE)*.yaml

# Qualify all recipes on the runner
qualify-all:
	@./scripts/qualify-all.sh

# Build mdBook
book:
	mdbook build docs/book

# Lint shell scripts and Makefile with bashrs
bashrs-lint:
	bashrs lint scripts/ Makefile

# Clean build artifacts
clean:
	cargo clean

# Release build
release:
	cargo build --workspace --release

# Help
help:
	@echo "Forjar Cookbook — Make targets:"
	@echo ""
	@echo "  build                Build all crates"
	@echo "  test                 Run all tests"
	@echo "  lint                 Run clippy lints"
	@echo "  fmt                  Format code"
	@echo "  check                Full quality gate chain (fmt, lint, test, docs, recipes)"
	@echo "  coverage             Generate coverage report"
	@echo "  coverage-check       Verify >= 95% coverage"
	@echo "  validate-recipes     Validate all recipe YAML (98/98 must pass)"
	@echo "  examples             Run all cargo examples"
	@echo "  docs-check           Check documentation consistency"
	@echo "  update-qualifications Update README table from CSV"
	@echo ""
	@echo "Scoring targets:"
	@echo "  score                      Score all recipes (static analysis)"
	@echo "  score-recipe RECIPE=01     Score a single recipe"
	@echo ""
	@echo "Qualification targets:"
	@echo "  qualify-recipe RECIPE=01   Qualify a single recipe"
	@echo "  qualify-all                Qualify all recipes (self-hosted runner)"
	@echo ""
	@echo "  book                 Build mdBook documentation"
	@echo "  bashrs-lint          Lint scripts and Makefile"
	@echo "  clean                Clean build artifacts"
	@echo "  help                 Show this help"
