# Secret Providers

Forjar resolves `{{ secrets.* }}` templates through 4 pluggable providers. This section shows common patterns for each.

## Environment Variables (Default)

The simplest approach — no external tools required:

```yaml
# forjar.yaml
secrets:
  provider: env

resources:
  app-env:
    type: file
    machine: web
    path: /etc/app/.env
    content: |
      DB_PASSWORD={{ secrets.db_password }}
      API_KEY={{ secrets.api_key }}
```

```bash
export FORJAR_SECRET_DB_PASSWORD=s3cret
export FORJAR_SECRET_API_KEY=sk-live-abc123
forjar apply -f forjar.yaml
```

**Best for**: CI/CD pipelines, development environments, simple deployments.

## File-Based Secrets

Mount secrets as files (works with Docker secrets, Kubernetes, systemd `LoadCredential=`):

```yaml
secrets:
  provider: file
  path: /run/secrets

resources:
  db-config:
    type: file
    machine: db
    path: /etc/postgres/pg_hba.conf
    content: |
      host all all 0.0.0.0/0 md5
      # password: {{ secrets.pg_password }}
```

Create the secret files:

```bash
echo -n "s3cret" > /run/secrets/pg_password
chmod 600 /run/secrets/pg_password
```

**Best for**: Container orchestration, systemd services, air-gapped environments.

## SOPS Encryption

Encrypt secrets at rest with Mozilla SOPS (supports AWS KMS, GCP KMS, Azure Key Vault, age, PGP):

```yaml
secrets:
  provider: sops
  file: secrets.enc.yaml
```

```bash
# Create and encrypt secrets file
cat > secrets.yaml << 'EOF'
db_password: s3cret
api_key: sk-live-abc123
EOF

sops -e secrets.yaml > secrets.enc.yaml
rm secrets.yaml  # plaintext removed
```

**Best for**: GitOps workflows, team collaboration (encrypted secrets in version control).

## 1Password

Resolve secrets from 1Password vaults via the `op` CLI:

```yaml
secrets:
  provider: op
  path: production    # vault name (default: forjar)
```

```bash
# Sign in first
eval $(op signin)

# Apply — secrets resolved from 1Password
forjar apply -f forjar.yaml
```

**Best for**: Teams already using 1Password, enterprise environments with centralized secret management.

## Combining with Age Encryption

You can use secret providers AND inline age encryption together. Secret providers handle `{{ secrets.* }}` templates, while age handles `ENC[age,...]` markers:

```yaml
secrets:
  provider: env

resources:
  mixed-config:
    type: file
    machine: web
    path: /etc/app/config
    content: |
      dynamic_secret={{ secrets.api_key }}
      static_secret=ENC[age,YWdlLWVuY3J5cHRpb24...]
```

## Secret Scanning

Always validate that secrets aren't hardcoded:

```bash
forjar validate --check-secrets -f forjar.yaml
```
