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

## Debugging with `forjar logs`

Every `forjar apply` captures structured run logs under `state/<machine>/runs/<run_id>/`.
Use `forjar logs` to inspect what happened.

### View recent runs

```bash
forjar logs --machine intel           # latest run on intel
forjar logs --all-machines            # latest run across all machines
forjar logs --run <run_id>            # specific run by ID
```

### Filter to failures

```bash
forjar logs --failures                # only show failed resources
forjar logs --resource nginx-pkg      # single resource detail
forjar logs --script                  # include generated script source
```

### JSON output for tooling

```bash
forjar logs --json | jq '.[] | select(.failed)'
```

### Garbage collection

Old run logs accumulate over time. Clean them up:

```bash
forjar logs --gc --dry-run            # preview what would be deleted
forjar logs --gc                      # delete old runs (keeps last 10)
forjar logs --gc --keep-failed        # keep failed runs, delete old successes
```

### Run directory structure

```
state/intel/runs/run-20260307-143022-a1b2/
  meta.yaml                           # run metadata + per-resource status
  nginx-pkg.apply.log                 # structured log (stdout, stderr, exit code)
  nginx-pkg.script                    # raw generated script
  cargo-tools.apply.log
  cargo-tools.script
```

## Lint Rules

Forjar's lint engine detects common anti-patterns in recipe YAML.

### Semicolon-chained commands (FJ-3000)

```yaml
# BAD: semicolons mask exit codes
command: "cd /app ; make build ; make install"

# GOOD: && fails fast
command: "cd /app && make build && make install"

# GOOD: multiline block
command: |
  cd /app
  make build
  make install
```

### nohup without LD_LIBRARY_PATH (FJ-3030)

```yaml
# WARNS: binary may fail to find shared libraries
command: "nohup /opt/llama/llama-server --port 8080 &"

# FIXED: set LD_LIBRARY_PATH before nohup
command: "LD_LIBRARY_PATH=/opt/llama nohup /opt/llama/llama-server --port 8080 &"
```

### nohup + sleep + health check race (FJ-3040)

```yaml
# BAD: fixed sleep is fragile
command: "nohup /opt/server & sleep 15; curl http://localhost:8080/health"

# GOOD: use task_mode: service with health_check
task_mode: service
command: "/opt/server --port 8080"
health_check:
  command: "curl -sf http://localhost:8080/health"
  retries: 10
  timeout: "5s"
```

### Running the linter

```bash
forjar lint -f recipe.yaml              # human-readable output
forjar lint -f recipe.yaml --json       # JSON output for CI
forjar lint -f recipe.yaml --strict     # additional strict rules
forjar lint -f recipe.yaml --fix        # auto-fix (resource key sorting)
```

## Performance Benchmarks

### Running benchmarks

```bash
forjar bench --iterations 1000              # colorized table
forjar bench --iterations 1000 --json       # JSON with p50/p95 stats
forjar bench --compare                      # diff against baseline
```

### Updating the baseline

```bash
make bench-update                           # writes benchmarks/RESULTS.md
```

### Investigating regressions

Use `--compare` to see percentage deltas against baseline:

```bash
forjar bench --iterations 1000 --compare
# Positive delta = faster, negative = regression
```

JSON output with `--compare` includes `baseline_avg_us` and `delta_pct` fields
for CI integration.

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
