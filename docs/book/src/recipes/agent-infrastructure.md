# Agent Infrastructure Recipes

Recipes for deploying and managing AI agents using forjar's standard resource types.
These recipes use `file` for configuration, `service` for process management,
`model` for ML artifacts, and `task` for orchestration steps.

## #73 pforge MCP Server

Deploy a pforge-managed MCP server stack. Configuration is declared in YAML and
managed alongside infrastructure — agent deployment becomes a converging
infrastructure operation.

**Tier**: 1+2 | **Idempotency**: Strong | **Grade**: A

```yaml
version: "1.0"
name: pforge-mcp-server

machines:
  agent:
    hostname: agent-01
    addr: 10.0.0.50

resources:
  # Install Node.js runtime for MCP servers
  node-pkg:
    type: package
    machine: agent
    provider: apt
    packages: [nodejs, npm]

  # Install pforge runtime
  pforge-pkg:
    type: package
    machine: agent
    provider: cargo
    packages: [pforge-runtime]
    depends_on: [node-pkg]

  # Deploy MCP server configuration
  mcp-config:
    type: file
    machine: agent
    path: /etc/pforge/servers.json
    content: |
      {
        "servers": [
          {
            "name": "filesystem",
            "command": "npx",
            "args": ["@anthropic/mcp-server-filesystem", "/data"]
          }
        ]
      }
    mode: "0644"
    owner: pforge
    depends_on: [node-pkg]

  # Start pforge agent service
  pforge-svc:
    type: service
    machine: agent
    name: pforge-agent
    state: running
    enabled: true
    restart_on: [mcp-config]
    depends_on: [pforge-pkg, mcp-config]

  # Health check
  health:
    type: task
    machine: agent
    command: "curl -sf http://localhost:8080/health"
    depends_on: [pforge-svc]
    timeout: 30
```

Key features:
- MCP server config as file resource (drift-detected via BLAKE3)
- pforge service managed with restart-on-config-change
- Health check via task resource
- Full dependency chain: packages -> config -> service -> health

## #74 Agent Deployment (Multi-Machine Fleet)

Deploy AI agents across a fleet of machines with rolling deploys
and configuration management.

**Tier**: 1+2 | **Idempotency**: Strong | **Grade**: A

```yaml
version: "1.0"
name: agent-fleet

machines:
  agent-01:
    hostname: agent-01
    addr: 10.0.1.1
  agent-02:
    hostname: agent-02
    addr: 10.0.1.2
  agent-03:
    hostname: agent-03
    addr: 10.0.1.3

params:
  model_name: claude-sonnet-4-6
  agent_port: "8080"

resources:
  # Deploy agent config to all machines
  agent-config:
    type: file
    machine: [agent-01, agent-02, agent-03]
    path: /etc/pforge/agent.yaml
    content: |
      name: fleet-agent
      model: {{params.model_name}}
      port: {{params.agent_port}}
      mcp_servers:
        - name: filesystem
          command: npx @anthropic/mcp-server-filesystem /data
    mode: "0600"

  # Start agent service on all machines
  agent-svc:
    type: service
    machine: [agent-01, agent-02, agent-03]
    name: pforge-agent
    state: running
    enabled: true
    restart_on: [agent-config]
    depends_on: [agent-config]

policy:
  parallel_machines: true
  serial: 1              # Rolling deploy: one machine at a time
  max_fail_percentage: 33
  notify:
    on_success: "echo 'Fleet deploy complete'"
    on_failure: "echo 'Fleet deploy failed: {{failed}} machines'"
```

Key features:
- Multi-machine targeting via machine list
- Template parameters for config customization
- Rolling deploy policy (serial: 1)
- Failure budget (33% max)
- Notification hooks

## #75 GPU-Accelerated Local Inference Agent

Deploy a GPU-backed inference agent with local model serving.

**Tier**: 2+3 | **Idempotency**: Strong | **Grade**: A

```yaml
version: "1.0"
name: gpu-inference-agent

machines:
  gpu-node:
    hostname: gpu-01
    addr: 10.0.0.50

resources:
  # GPU infrastructure
  gpu:
    type: gpu
    machine: gpu-node
    gpu_backend: nvidia
    driver_version: "550"
    cuda_version: "12.4"
    persistence_mode: true

  # Model artifact
  model:
    type: model
    machine: gpu-node
    name: llama-3.2-1b
    source: /models/llama-3.2-1b-q4_k_m.gguf
    format: gguf
    quantization: q4_k_m
    checksum: "blake3:a1b2c3d4..."
    depends_on: [gpu]

  # Agent config pointing to local model
  config:
    type: file
    machine: gpu-node
    path: /etc/pforge/inference-agent.yaml
    content: |
      name: inference-agent
      model_path: /models/llama-3.2-1b-q4_k_m.gguf
      gpu: true
      port: 8080
    depends_on: [model]

  # Agent service
  service:
    type: service
    machine: gpu-node
    name: inference-agent
    state: running
    enabled: true
    restart_on: [config, model]
    depends_on: [config]

policy:
  tripwire: true       # Detect model/config drift
  convergence_budget: 600
```

Key features:
- GPU driver and toolkit convergence
- Model integrity via BLAKE3 checksum
- Service restarts on model or config change
- Drift detection catches model corruption
