# Observability

Forjar exports W3C-compatible traces to any OTLP collector for infrastructure observability.

## Jaeger Integration

```bash
# Start Jaeger all-in-one
docker run -d --name jaeger \
  -p 4318:4318 -p 16686:16686 \
  jaegertracing/all-in-one:latest

# Apply with telemetry export
forjar apply -f forjar.yaml --telemetry-endpoint http://localhost:4318

# View traces at http://localhost:16686
```

## Grafana Tempo

```bash
forjar apply -f forjar.yaml \
  --telemetry-endpoint http://tempo:4318
```

## CI Health Check

Use traces + health queries in CI for deployment gates:

```bash
# Apply with trace export
forjar apply -f forjar.yaml \
  --telemetry-endpoint $OTEL_ENDPOINT

# Check stack health
forjar state-query --health --json | jq '.health_pct'

# Check for failures
forjar state-query --failures --since 1h
```

## Local Traces

Without an OTLP endpoint, traces are still persisted locally:

```bash
# View local traces
cat state/intel/trace.jsonl | jq -c '{name, action, duration_us, exit_code}'
```
