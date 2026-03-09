# Shell Providers

Shell providers extend forjar with custom bash scripts for check/apply/destroy
operations on resources not covered by built-in types.

## Creating a Shell Provider

### Manifest

Create `providers/nginx/provider.yaml`:

```yaml
name: nginx
version: "1.0.0"
description: "Nginx configuration management"
check: check.sh
apply: apply.sh
destroy: destroy.sh
```

### Scripts

`providers/nginx/check.sh`:
```bash
#!/bin/bash
set -euo pipefail
# Return 0 if nginx is running and config is valid
nginx -t 2>/dev/null && systemctl is-active nginx
```

`providers/nginx/apply.sh`:
```bash
#!/bin/bash
set -euo pipefail
nginx -t
systemctl reload nginx
```

`providers/nginx/destroy.sh`:
```bash
#!/bin/bash
set -euo pipefail
systemctl stop nginx
systemctl disable nginx
```

### Using in Resources

```yaml
resources:
  web-server:
    type: "shell:nginx"
    params:
      config_path: /etc/nginx/nginx.conf
```

## Security

All shell provider scripts are validated before execution:

1. **bashrs validation**: Ensures scripts follow safe bash patterns
2. **Secret leakage scan**: 14 regex patterns detect hardcoded credentials

### Blocked Patterns

```bash
# These will be REJECTED:
echo $PASSWORD               # leaks secret via echo
curl -u admin:pass url       # inline credentials
sshpass -p secret ssh host   # inline password
export TOKEN=abc123          # exports secret to env
```

### Safe Alternative

Use forjar's ephemeral values for secret injection:

```yaml
resources:
  api-config:
    type: "shell:api-setup"
    ephemeral:
      API_TOKEN:
        provider: env
        key: VAULT_API_TOKEN
```

## Listing and Validating

```bash
# List installed shell providers
forjar plugin list --plugin-dir providers/

# Validate all scripts in a provider
# (runs bashrs + secret leak detection)
```
