# Compliance Packs

Forjar includes built-in compliance packs for industry security frameworks. This recipe demonstrates evaluating infrastructure against CIS Ubuntu 22.04 benchmarks.

## CIS Ubuntu 22.04 Pack

The built-in pack has 24 rules covering file permissions, service hardening, network configuration, access controls, authentication, and system maintenance.

### Using the Pack

```rust
use forjar::core::cis_ubuntu_pack::{cis_ubuntu_2204_pack, severity_summary};
use forjar::core::compliance_pack::evaluate_pack;
use std::collections::HashMap;

let pack = cis_ubuntu_2204_pack();
let (errors, warnings, info) = severity_summary(&pack);
// 14 error, 9 warning, 1 info

let mut resources = HashMap::new();
let mut file = HashMap::new();
file.insert("type".into(), "file".into());
file.insert("mode".into(), "0644".into());
file.insert("owner".into(), "root".into());
resources.insert("etc-passwd".into(), file);

let result = evaluate_pack(&pack, &resources);
println!("Pass rate: {:.1}%", result.pass_rate());
```

### Key Rules

| Rule | Severity | Description |
|------|----------|-------------|
| CIS-1.1.1 | Error | /tmp must be separate partition |
| CIS-2.1.2 | Error | No telnet server installed |
| CIS-5.2.1 | Error | SSH root login disabled |
| CIS-5.3.1 | Error | No world-writable files (mode 777) |
| CIS-6.1.1 | Error | Files must have owner defined |

### Export as YAML

```rust
use forjar::core::cis_ubuntu_pack::cis_ubuntu_yaml;

let yaml = cis_ubuntu_yaml().unwrap();
std::fs::write("policies/cis-ubuntu-22.04.yaml", &yaml).unwrap();
```

## Policy Coverage

Track which resources have policies and which are uncovered:

```rust
use forjar::core::policy_coverage::{compute_coverage, format_coverage};

let coverage = compute_coverage(&config);
println!("{}", format_coverage(&coverage));
// Policy Coverage: 60.0% (3/5)
//   Uncovered (2):
//     - redis-server
//     - backup-job
```

## Plugin Hot-Reload

WASM plugins are cached with BLAKE3 hash verification. When a `.wasm` file changes on disk, forjar detects the hash mismatch and reloads automatically.

```rust
use forjar::core::plugin_hot_reload::{PluginCache, ReloadCheck};

let mut cache = PluginCache::new();
cache.insert("my-plugin", manifest, wasm_path);

// On next invocation, check if plugin changed
match cache.needs_reload("my-plugin") {
    ReloadCheck::UpToDate => { /* use cached */ }
    ReloadCheck::Changed { .. } => { /* reload from disk */ }
    ReloadCheck::FileGone => { /* plugin deleted */ }
    ReloadCheck::NotCached => { /* first load */ }
}
```

## Webhook Events

External systems trigger forjar automation via HTTP webhooks:

```rust
use forjar::core::webhook_source::{WebhookConfig, validate_request};

let config = WebhookConfig {
    port: 8484,
    secret: Some("shared-secret".into()),
    allowed_paths: vec!["/webhook".into()],
    ..Default::default()
};
```

Request validation checks: POST method, body size, path allowlist, HMAC signature.

## Running the Examples

```bash
cargo run --example cis_compliance
cargo run --example policy_coverage
cargo run --example plugin_hot_reload
cargo run --example webhook_source
```
