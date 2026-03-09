# Security Scanning

Forjar's static security scanner detects IaC security smells before apply.

## Quick Start

```rust
use forjar::core::security_scanner::{scan, severity_counts};
use forjar::core::parser;

let yaml = std::fs::read_to_string("recipe.yaml").unwrap();
let config = parser::parse_config(&yaml).unwrap();
let findings = scan(&config);
let (critical, high, medium, low) = severity_counts(&findings);

println!("Findings: {} critical, {} high, {} medium, {} low",
    critical, high, medium, low);
```

## Detection Rules

| Rule | Category | Severity | Detects |
|------|----------|----------|---------|
| SS-1 | Hard-coded secret | Critical | `password=`, `token=`, `api_key=` in content |
| SS-2 | HTTP without TLS | High | Unencrypted `http://` URLs (except localhost) |
| SS-3 | World-accessible | High | File mode last digit ≥ 4 |
| SS-4 | Missing integrity | Medium | External source without hash check |
| SS-5 | Privileged container | Critical | Docker `privileged=true` |
| SS-6 | No resource limits | Low | Docker without memory/CPU limits |
| SS-7 | Weak crypto | High | md5, sha1, des, rc4, sslv3 references |
| SS-8 | Insecure protocol | High | `telnet://`, `ftp://`, `rsh://` |
| SS-9 | Unrestricted network | Medium | Binding to `0.0.0.0` |
| SS-10 | Sensitive data | Critical | PII patterns (`ssn=`, `credit_card=`) |

## Path Deny Policy

```yaml
policy:
  deny_paths:
    - /etc/shadow
    - /etc/sudoers
    - /root/.ssh/*
```

## Operator Authorization

```yaml
machines:
  production:
    hostname: prod-01
    addr: 10.0.1.1
    user: deploy
    allowed_operators:
      - deploy-bot
      - admin@company.com
```

## Falsification

```bash
cargo run --example platform_security_falsification
```

Key invariants:
- **SS-1**: Hardcoded `password=` triggers Critical finding
- **SS-3**: mode `0777` triggers High finding
- **SS-2**: `http://` triggers, `http://localhost` does not
- **Clean config**: mode `0600` + no secrets → zero SS-1/SS-3 findings
- **Path policy**: exact match and glob `*` suffix both work
