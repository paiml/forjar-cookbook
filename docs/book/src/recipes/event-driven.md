# Event-Driven Automation Recipes

Recipes using forjar's event-driven engine (FJ-3100) for reactive
infrastructure convergence.

## Config Drift Auto-Repair

Automatically re-apply config when a managed file is modified:

```yaml
version: "1.0"
name: nginx-auto-repair
machines:
  web:
    hostname: web-01
    addr: 10.0.1.10
resources:
  nginx-config:
    type: file
    machine: web
    path: /etc/nginx/nginx.conf
    source: configs/nginx.conf
    tags: [config, nginx]
rulebooks:
  - name: nginx-repair
    events:
      - type: file_changed
        match:
          path: /etc/nginx/nginx.conf
    actions:
      - apply:
          file: forjar.yaml
          tags: [config]
    cooldown_secs: 60
```

## Process Crash Recovery

Restart a service when the process exits unexpectedly:

```yaml
rulebooks:
  - name: app-recovery
    events:
      - type: process_exit
        match:
          process: myapp
          exit_code: "137"
    actions:
      - script: "systemctl restart myapp"
      - notify:
          channel: "https://hooks.slack.com/services/xxx"
          message: "myapp crashed on {{machine}} (OOM kill)"
    cooldown_secs: 300
    max_retries: 3
```

## Scheduled Cleanup

Run cleanup on a cron schedule:

```yaml
rulebooks:
  - name: log-rotation
    events:
      - type: cron_fired
    actions:
      - script: "find /var/log/myapp -name '*.log' -mtime +7 -delete"
    cooldown_secs: 3600
```

## Webhook-Triggered Deploy

Deploy on GitHub push webhook:

```yaml
rulebooks:
  - name: auto-deploy
    events:
      - type: webhook_received
        match:
          ref: refs/heads/main
    actions:
      - script: "cd /opt/app && git pull && forjar apply -f deploy.yaml"
      - notify:
          channel: "https://hooks.slack.com/services/xxx"
          message: "Auto-deploy triggered by push to main"
```

## Multi-Rulebook Config

```yaml
rulebooks:
  - name: config-repair
    events:
      - type: file_changed
        match:
          path: /etc/nginx/nginx.conf
    actions:
      - apply:
          file: forjar.yaml
          tags: [config]

  - name: daily-audit
    events:
      - type: cron_fired
    actions:
      - script: "forjar drift --tripwire"

  - name: deploy-notify
    events:
      - type: manual
    actions:
      - notify:
          channel: "https://hooks.slack.com/services/xxx"
          message: "Manual deployment completed"
```

## Example

```bash
cargo run --example event_rulebook
```
