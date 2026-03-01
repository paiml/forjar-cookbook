# Troubleshooting

## Recipe Qualification Failures

### "validation failed"

The recipe YAML has a syntax or schema error. Run:
```bash
forjar validate -f recipes/NN-name.yaml
```

### "plan failed"

The dependency graph cannot resolve. Check `depends_on` references point to
existing resource names.

### "first apply failed"

The convergence scripts failed on the target machine. Check:
- Is the target machine reachable?
- Does the transport (local/SSH/container) work?
- Are required packages available in the apt repo?

### "idempotency check failed"

The second apply produced changes. Common causes:
- File content with timestamps (`{{now}}`)
- apt-get running on every apply (check script bug)
- Service restarts triggered by hash changes

### "score below threshold"

The static-only score command exited non-zero. Check dimension scores:
```bash
cargo run --bin cookbook-runner -- score -f recipes/NN-name.yaml
```

Common causes for low dimension scores:
- **SAF**: mode 0777 on files, curl|bash patterns, missing version pins
- **OBS**: no tripwire policy, missing notify hooks, no outputs section
- **DOC**: comment ratio below 15%, missing header metadata
- **RES**: no failure policy, no dependency DAG, no lifecycle hooks
- **CMP**: no params, no templates, no includes, no tags

## Build Issues

### "cargo test fails"

```bash
cargo test --workspace 2>&1 | grep "FAILED"
```

### "coverage below 95%"

```bash
cargo llvm-cov --workspace --lib --summary-only
```

Look for files with low coverage and add tests.

### "clippy errors"

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

### "pmat comply warnings"

```bash
pmat comply check
```

Most warnings are advisory (CB-500 series). Errors need fixing.
