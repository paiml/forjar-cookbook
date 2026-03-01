# Failure Mode Recipes

Recipes that deliberately test forjar's error handling, recovery, and
resilience under adverse conditions. These recipes verify that forjar
degrades gracefully and provides actionable diagnostics.

## #50 Partial Apply Recovery

Simulates a partial apply failure where some resources converge but
others fail. Verifies that forjar records partial state correctly and
can resume from where it left off on retry.

**Resources**: succeeding-resource (file), failing-resource (file),
dependent-resource (file)

**Tier**: 2+3 | **Idempotency**: Strong | **Grade**: A

## #51 State Recovery

Tests forjar's ability to recover from corrupted or missing state files.
Verifies that forjar can reconstruct state from the actual system state
when the state file is damaged or deleted.

**Resources**: baseline-resources (file), state-file (file),
recovery-check (file)

**Tier**: 2+3 | **Idempotency**: Strong | **Grade**: A

## #52 Crash Resilience

Simulates forjar being interrupted mid-apply (SIGTERM, SIGKILL) and
verifies that the system is left in a consistent state. Tests lock
file cleanup, partial write handling, and state file atomicity.

**Resources**: long-running-resource (file), lock-file (file),
crash-marker (file)

**Tier**: 3 | **Idempotency**: Strong | **Requires**: Bare-metal or VM
