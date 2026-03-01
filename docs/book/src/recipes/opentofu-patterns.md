# OpenTofu-Inspired Patterns

Patterns borrowed from OpenTofu/Terraform, adapted to forjar's
sovereign model. These recipes validate that forjar supports the
same operational patterns that infrastructure teams expect from
mature IaC tools.

## #30 Saved Plan Files

TOCTOU-safe plan/apply workflow. Generates a saved plan file that
captures the exact changes to be applied, then applies from the
saved plan rather than re-planning at apply time.

**Resources**: plan-file (file), apply-from-plan (file)

**Tier**: 1+2 | **Idempotency**: Strong

## #31 JSON Plan Format

Machine-readable plan output in JSON format for CI/CD integration.
The plan output can be parsed by downstream tools for approval
workflows and cost estimation.

**Resources**: plan-json (file), plan-parser (file)

**Tier**: 1+2 | **Idempotency**: Strong

## #32 Check Blocks

Post-apply health assertions that verify the system is actually
functioning after convergence. Check blocks run after apply and
fail the apply if assertions don't pass.

**Resources**: app-deploy (file), health-check (check)

**Tier**: 2+3 | **Idempotency**: Strong

## #33 Lifecycle Protection

Prevent accidental destruction of critical resources using lifecycle
rules (prevent_destroy, create_before_destroy, ignore_changes).

**Resources**: protected-db (file), lifecycle-rules (file)

**Tier**: 1+2 | **Idempotency**: Strong

## #34 Moved Blocks

Rename or reorganize resources without destroying and recreating them.
The moved block tells forjar that a resource has been renamed in the
config but should keep its existing state.

**Resources**: original-resource (file), moved-declaration (file)

**Tier**: 1+2 | **Idempotency**: Strong

## #35 Refresh-Only Mode

Sync forjar's state with actual infrastructure without making changes.
Useful for detecting drift and updating the state file to match
reality.

**Resources**: baseline-resources (file), state-sync (file)

**Tier**: 1+2 | **Idempotency**: Strong

## #36 Resource Targeting

Apply changes to specific resources without touching the rest of the
configuration. Useful for emergency patches and targeted updates.

**Resources**: targeted-resource (file), skip-resources (file)

**Tier**: 1+2 | **Idempotency**: Strong

## #37 Testing DSL

Native test assertions for recipe validation. Define expected outcomes
inline in the recipe config and verify them during plan or apply.

**Resources**: test-assertions (file), expected-state (file)

**Tier**: 1+2 | **Idempotency**: Strong

## #38 State Encryption

Encrypt the forjar state file at rest using AES-256-GCM. Protects
sensitive infrastructure state data from unauthorized access.

**Resources**: encrypted-state (file), key-config (file)

**Tier**: 1+2 | **Idempotency**: Strong

## #39 Cross-Config Data Source

Reference outputs from one forjar config as inputs to another.
Enables composition of independently managed infrastructure
components through data sources.

**Resources**: upstream-output (file), downstream-input (file)

**Tier**: 1+2 | **Idempotency**: Strong
