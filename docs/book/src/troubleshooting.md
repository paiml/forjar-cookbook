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
