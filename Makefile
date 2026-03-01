# Forjar Cookbook — Qualification Suite
# Primary target: self-hosted Intel runner (bare-metal)

.SUFFIXES:

.PHONY: all build test lint fmt fmt-check check coverage coverage-check \
        examples docs-check update-qualifications qualify-recipe qualify-all \
        book bashrs-lint clean help

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
check: fmt-check lint test docs-check

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
	@echo "  check                Full quality gate chain (fmt, lint, test, docs)"
	@echo "  coverage             Generate coverage report"
	@echo "  coverage-check       Verify >= 95% coverage"
	@echo "  examples             Run all cargo examples"
	@echo "  docs-check           Check documentation consistency"
	@echo "  update-qualifications Update README table from CSV"
	@echo ""
	@echo "Qualification targets:"
	@echo "  qualify-recipe RECIPE=01   Qualify a single recipe"
	@echo "  qualify-all                Qualify all recipes (self-hosted runner)"
	@echo ""
	@echo "  book                 Build mdBook documentation"
	@echo "  bashrs-lint          Lint scripts and Makefile"
	@echo "  clean                Clean build artifacts"
	@echo "  help                 Show this help"
