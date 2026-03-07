# Task Pipeline Recipes

Recipes demonstrating forjar's task pipeline, service, and dispatch modes
for DataOps, MLOps, and CI/CD workflows. These use `type: task` with
`task_mode: pipeline|service|dispatch` for multi-stage execution with
quality gates, health checks, and GPU scheduling.

## Pipeline Mode

Pipeline tasks execute ordered stages with quality gates between them.
Each stage can specify inputs/outputs for cache invalidation — if inputs
haven't changed, the stage is skipped.

### Data Quality Pipeline (alimentar)

```yaml
resources:
  quality-pipeline:
    type: task
    task_mode: pipeline
    stages:
      - name: ingest
        command: "alimentar ingest --source raw/ --output staging/"
        inputs: ["raw/**/*.csv"]
        outputs: ["staging/ingested.parquet"]
        gate: true
      - name: validate
        command: "alimentar validate --input staging/ingested.parquet"
        inputs: ["staging/ingested.parquet"]
        outputs: ["staging/validation-report.json"]
        gate: true
      - name: score
        command: "alimentar score --input staging/validation-report.json"
        outputs: ["output/quality-score.json"]
    quality_gate:
      parse: json
      field: grade
      threshold: ["A", "B"]
      on_fail: block
```

The `gate: true` flag means the pipeline stops if that stage fails.
The `quality_gate` at the end parses JSON output to enforce grade thresholds.

### ML Training Pipeline (entrenar)

```yaml
resources:
  training-pipeline:
    type: task
    task_mode: pipeline
    gpu_device: 0
    stages:
      - name: prepare
        command: "entrenar prepare --dataset data/"
        inputs: ["data/**/*.jsonl"]
        outputs: ["prepared/train.bin"]
        gate: true
      - name: train
        command: "entrenar train --config train.yaml --data prepared/"
        inputs: ["prepared/train.bin", "train.yaml"]
        outputs: ["checkpoints/final/model.safetensors"]
        gate: true
      - name: evaluate
        command: "entrenar eval --model checkpoints/final/"
        outputs: ["eval/metrics.json"]
    quality_gate:
      parse: json
      field: accuracy
      threshold: ["0.85"]
      on_fail: warn
```

The `gpu_device: 0` field injects `CUDA_VISIBLE_DEVICES=0` into all stages.

## Service Mode

Service tasks run long-lived processes with health checks and restart policies.

### Agent Service (batuta)

```yaml
resources:
  agent-service:
    type: task
    task_mode: service
    command: "batuta agent serve --config agent.yaml --port 8080"
    health_check:
      command: "curl -sf http://localhost:8080/health"
      interval: "30s"
      timeout: "5s"
      retries: 3
    restart_policy:
      max_restarts: 5
      backoff_base_secs: 2
      backoff_cap_secs: 60
```

Restart uses exponential backoff: 2s, 4s, 8s, 16s, 32s, capped at 60s.

## GPU Scheduling

Multiple GPU tasks can run in parallel on different devices using
round-robin or explicit device assignment:

```
Task: train-a → CUDA_VISIBLE_DEVICES=0
Task: train-b → CUDA_VISIBLE_DEVICES=1
Task: train-c → CUDA_VISIBLE_DEVICES=0  (round-robin wraps)
```

For multi-GPU models, assign multiple devices:

```yaml
resources:
  large-model:
    type: task
    gpu_device: "0,1,2,3"
    command: "torchrun --nproc_per_node=4 train.py"
```

## Scatter/Gather (FJ-2704)

Distributed tasks can move artifacts between machines using `scatter:`
(copy local files to remote paths before execution) and `gather:` (collect
remote results back to local paths after execution).

```yaml
resources:
  train-model:
    type: task
    machine: gpu-worker
    scatter:
      - "/data/dataset.csv:/remote/input/dataset.csv"
      - "/config/train.yaml:/remote/config/train.yaml"
    command: "python /remote/train.py --data /remote/input/dataset.csv"
    gather:
      - "/remote/output/model.bin:/local/models/latest.bin"
```

The execution order is:
1. **Scatter** — copy local artifacts to remote paths (`cp -r`)
2. **Command** — run the task on the remote machine
3. **Gather** — copy remote results back to local paths

Each mapping is a `local:remote` pair separated by `:`. Directories are
created automatically via `mkdir -p`. Invalid mappings (no `:`) are silently
skipped.

For federated learning across multiple machines:

```yaml
resources:
  train-node-1:
    type: task
    machine: gpu-0
    scatter:
      - "/shared/global-model.bin:/tmp/model.bin"
    command: "python train.py --model /tmp/model.bin --output /tmp/gradients.bin"
    gather:
      - "/tmp/gradients.bin:/shared/gradients/node-1.bin"

  train-node-2:
    type: task
    machine: gpu-1
    scatter:
      - "/shared/global-model.bin:/tmp/model.bin"
    command: "python train.py --model /tmp/model.bin --output /tmp/gradients.bin"
    gather:
      - "/tmp/gradients.bin:/shared/gradients/node-2.bin"
```

## Barrier Tasks

Cross-machine synchronization uses barrier tasks that wait for all
specified machines to complete before proceeding:

```
barrier/sync-training: waiting for gpu-0, gpu-1, gpu-2 (33%)
barrier/sync-training: SATISFIED
```

This enables federated workflows where multiple machines must reach
a checkpoint before the next phase begins.

## Reference Recipes

Full reference recipes for each consumer domain are available in
the forjar repository at `examples/consumer-*.yaml`:

| Recipe | Mode | Domain |
|--------|------|--------|
| `consumer-alimentar-quality.yaml` | pipeline | Data quality |
| `consumer-entrenar-training.yaml` | pipeline + GPU | ML training |
| `consumer-apr-model-build.yaml` | pipeline | Model compilation |
| `consumer-batuta-agent.yaml` | service + dispatch | Agent lifecycle |
| `consumer-forjar-self-build.yaml` | pipeline | CI/CD self-build |
