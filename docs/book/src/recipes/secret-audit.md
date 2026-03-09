# Secret Audit & Namespace Isolation

Forjar provides a JSONL audit trail for all secret access and namespace-isolated execution to prevent secret leakage.

## Secret Access Audit (FJ-3308)

Every secret lifecycle event is logged with BLAKE3 hash (never plaintext):

```rust
use forjar::core::secret_audit::{
    append_audit, make_resolve_event, make_inject_event,
    make_discard_event, read_audit, audit_summary,
    format_audit_summary, filter_by_key,
};

// Resolve → Inject → Discard lifecycle
let resolve = make_resolve_event("db_pass", "env", &hash, Some("web-01"));
append_audit(state_dir, &resolve).unwrap();

let inject = make_inject_event("db_pass", "env", &hash, "ns-forjar-1");
append_audit(state_dir, &inject).unwrap();

let discard = make_discard_event("db_pass", &hash);
append_audit(state_dir, &discard).unwrap();

// Analyze
let events = read_audit(state_dir).unwrap();
let summary = audit_summary(&events);
println!("{}", format_audit_summary(&summary));
```

### Event Types

| Type | When |
|------|------|
| `resolve` | Secret fetched from provider |
| `inject` | Secret pushed to child process |
| `discard` | Secret cleared from memory |
| `rotate` | Key rotated to new value |

## Namespace Isolation (FJ-3306)

Secrets are injected into isolated child processes. The parent environment is never contaminated.

```rust
use forjar::core::secret_namespace::{NamespaceConfig, execute_isolated};
use forjar::core::ephemeral::ResolvedEphemeral;

let config = NamespaceConfig {
    namespace_id: "ns-forjar-apply-1".into(),
    audit_enabled: true,
    state_dir: Some(state_dir.into()),
    inherit_env: vec!["PATH".into()],
};

let secrets = vec![ResolvedEphemeral {
    key: "DB_PASS".into(),
    value: "secret".into(),
    hash: blake3::hash(b"secret").to_hex().to_string(),
}];

let result = execute_isolated(&config, &secrets, "sh", &["-c", "echo $DB_PASS"]).unwrap();
// Parent env is clean — DB_PASS only existed in the child
```

## Policy Boundary Testing (FJ-3209)

Verify policy rules are non-vacuous by generating boundary configs:

```rust
use forjar::core::policy_boundary::{test_boundaries, format_boundary_results};

let result = test_boundaries(&compliance_pack);
println!("{}", format_boundary_results(&result));
// Result: 48/48 boundary tests passed
```

## Running the Examples

```bash
cargo run --example secret_audit
cargo run --example secret_namespace
cargo run --example policy_boundary
```
