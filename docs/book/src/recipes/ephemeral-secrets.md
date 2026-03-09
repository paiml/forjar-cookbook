# Ephemeral Secrets

Forjar's ephemeral value pipeline (FJ-3302) resolves secrets at apply time,
uses them for template substitution, then discards the plaintext. Only a
BLAKE3 hash is persisted in state for drift detection. This chapter shows
common patterns for ephemeral secret management.

## Database Password Rotation (Hash-and-Discard)

Rotate a database password without storing the plaintext in state. Forjar
resolves the secret, writes the config file, then stores only the BLAKE3
hash. On the next apply, if the provider returns a different value, the
hash won't match and forjar reports drift.

```yaml
version: "1.0"
name: db-password-rotation
machines:
  db:
    hostname: db-01
    addr: 10.0.2.10

secrets:
  provider: env
  ephemeral: true   # hash-and-discard mode

resources:
  pg-auth:
    type: file
    machine: db
    path: /etc/postgres/pg_auth.conf
    owner: postgres
    mode: "0600"
    content: |
      host all app_user 10.0.0.0/8 scram-sha-256
    tags: [database, auth]

  rotation-script:
    type: file
    machine: db
    path: /usr/local/bin/rotate-db-password.sh
    owner: root
    mode: "0700"
    content: |
      #!/bin/bash
      set -euo pipefail
      NEW_PASS="{{ secrets.db_password }}"
      psql -c "ALTER USER app_user PASSWORD '$NEW_PASS'"
      echo "Password rotated at $(date -Iseconds)" >> /var/log/forjar-rotation.log
    depends_on: [pg-auth]
    tags: [database, rotation]

policy:
  tripwire: true
  notify:
    on_drift: "echo 'DB password hash changed — rotation detected'"
```

```bash
# Generate a new password and apply
export FORJAR_SECRET_DB_PASSWORD=$(openssl rand -base64 32)
forjar apply -f db-rotation.yaml

# State contains only the BLAKE3 hash, not the password
forjar state-query --resource rotation-script --json | jq '.ephemeral_hashes'
```

## Multi-Provider Secret Chain (Fallback Resolution)

Forjar's `ProviderChain` tries providers in order until one resolves. This
pattern configures env -> file -> exec fallback so secrets work across
development, staging, and production without changing the recipe.

```yaml
version: "1.0"
name: multi-provider-chain
machines:
  app:
    hostname: app-01
    addr: 10.0.1.10

secrets:
  chain:
    - provider: env       # 1st: check environment (CI/CD)
    - provider: file      # 2nd: check /run/secrets (containers)
      path: /run/secrets
    - provider: exec      # 3rd: call vault CLI (production)
      command: vault kv get -field=value secret/myapp

resources:
  app-env:
    type: file
    machine: app
    path: /etc/app/.env
    owner: app
    mode: "0600"
    content: |
      DATABASE_URL=postgres://app:{{ secrets.db_password }}@db:5432/myapp
      REDIS_URL=redis://:{{ secrets.redis_password }}@cache:6379/0
      API_KEY={{ secrets.api_key }}
    tags: [app, config]

  connection-test:
    type: file
    machine: app
    path: /usr/local/bin/test-connections.sh
    owner: root
    mode: "0755"
    content: |
      #!/bin/bash
      set -euo pipefail
      source /etc/app/.env
      pg_isready -d "$DATABASE_URL" && echo "DB: ok"
      redis-cli -u "$REDIS_URL" ping && echo "Redis: ok"
    depends_on: [app-env]
    tags: [app, healthcheck]
```

```bash
# Development: secrets from environment
export FORJAR_SECRET_DB_PASSWORD=dev-pass
export FORJAR_SECRET_REDIS_PASSWORD=dev-redis
export FORJAR_SECRET_API_KEY=dev-key
forjar apply -f chain.yaml

# Container: secrets from mounted files
echo -n "prod-pass" > /run/secrets/db_password
forjar apply -f chain.yaml

# Production: secrets from Vault (exec provider)
forjar apply -f chain.yaml  # vault CLI called automatically
```

## Ephemeral Drift Detection

When `ephemeral: true` is set, forjar stores BLAKE3 hashes in state
instead of plaintext values. On subsequent applies, it re-resolves the
secret and compares hashes. If the hash changes, forjar reports drift
without ever exposing the old or new value in logs.

```yaml
version: "1.0"
name: ephemeral-drift
machines:
  web:
    hostname: web-01
    addr: 10.0.1.20

secrets:
  provider: env
  ephemeral: true

resources:
  tls-passphrase:
    type: file
    machine: web
    path: /etc/nginx/ssl/passphrase
    owner: root
    mode: "0400"
    content: "{{ secrets.tls_passphrase }}"
    tags: [tls, secret]

  api-token:
    type: file
    machine: web
    path: /etc/app/api-token
    owner: app
    mode: "0400"
    content: "{{ secrets.api_token }}"
    tags: [app, secret]

policy:
  tripwire: true
  lock_file: true
  notify:
    on_drift: "echo 'Ephemeral value drift — secret was rotated externally'"
```

```bash
# Apply with initial secrets
export FORJAR_SECRET_TLS_PASSPHRASE=original-passphrase
export FORJAR_SECRET_API_TOKEN=tok-abc123
forjar apply -f ephemeral-drift.yaml

# Check drift (no change)
forjar drift -f ephemeral-drift.yaml
# Output: 0 resources drifted

# Rotate a secret externally
export FORJAR_SECRET_API_TOKEN=tok-xyz789
forjar drift -f ephemeral-drift.yaml
# Output: 1 resource drifted (api-token: ephemeral hash mismatch)

# Verify: plaintext never appears in state
cat state/ephemeral-drift/state.lock.yaml | grep -c "tok-"
# Output: 0
```

## Template Substitution for Connection Strings

Combine ephemeral secrets with `{{ params.* }}` templates to build
connection strings that vary per environment. The secret portion is
resolved and discarded; the structural portion stays in state.

```yaml
version: "1.0"
name: connection-templates
params:
  env: production
  db_host: db.internal
  db_port: "5432"
  db_name: myapp
  redis_host: cache.internal
  redis_port: "6379"

machines:
  app:
    hostname: app-01
    addr: 10.0.1.10

secrets:
  provider: env
  ephemeral: true

resources:
  datasource-config:
    type: file
    machine: app
    path: /etc/app/datasources.yaml
    owner: app
    mode: "0600"
    content: |
      # Environment: {{ params.env }}
      # Managed by forjar — do not edit manually
      datasources:
        primary:
          driver: postgresql
          host: {{ params.db_host }}
          port: {{ params.db_port }}
          database: {{ params.db_name }}
          username: app_{{ params.env }}
          password: {{ secrets.db_password }}
          pool_size: 20
          ssl_mode: require
        cache:
          driver: redis
          url: redis://:{{ secrets.redis_password }}@{{ params.redis_host }}:{{ params.redis_port }}/0
          pool_size: 10
        search:
          driver: elasticsearch
          url: https://{{ secrets.es_user }}:{{ secrets.es_password }}@search.internal:9200
    tags: [app, datasource]

  connection-validator:
    type: file
    machine: app
    path: /usr/local/bin/validate-connections.sh
    owner: root
    mode: "0755"
    content: |
      #!/bin/bash
      set -euo pipefail
      echo "Validating connections for {{ params.env }}..."
      pg_isready -h {{ params.db_host }} -p {{ params.db_port }} -d {{ params.db_name }}
      redis-cli -h {{ params.redis_host }} -p {{ params.redis_port }} ping
      curl -sf https://search.internal:9200/_cluster/health | jq .status
    depends_on: [datasource-config]
    tags: [app, validation]

policy:
  tripwire: true
  notify:
    on_success: "echo 'Connection configs deployed for {{ params.env }}'"
    on_drift: "echo 'Connection config drift in {{ params.env }}'"
```

```bash
# Production deployment
export FORJAR_SECRET_DB_PASSWORD=prod-db-pass
export FORJAR_SECRET_REDIS_PASSWORD=prod-redis-pass
export FORJAR_SECRET_ES_USER=elastic
export FORJAR_SECRET_ES_PASSWORD=prod-es-pass
forjar apply -f connection-templates.yaml

# Staging override via params
forjar apply -f connection-templates.yaml \
  --set env=staging \
  --set db_host=staging-db.internal \
  --set redis_host=staging-cache.internal
```

**Best for**: Multi-environment deployments where connection topology varies
but secret handling must remain consistent and auditable.
