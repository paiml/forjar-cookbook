# Getting Started

## Prerequisites

- Rust toolchain (1.85+)
- forjar binary installed (`cargo install --path ../forjar`)
- Docker (for Tier 2 container tests)

## Quick Start

```bash
# Clone the cookbook
git clone https://github.com/paiml/forjar-cookbook
cd forjar-cookbook

# Run quality gates
make check

# Validate all recipes
cargo run --example validate_all

# Plan all recipes (dry-run, no apply)
cargo run --example plan_all
```

## Qualifying a Recipe

```bash
# On the self-hosted runner
make qualify-recipe RECIPE=01

# Update the README dashboard
make update-qualifications
```
