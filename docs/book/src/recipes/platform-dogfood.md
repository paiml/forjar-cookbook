# Platform Dogfood Recipes

Recipes that exercise forjar's newest platform features. These
serve as integration tests and documentation for the platform spec.

## 96: OCI Image Build + Registry Push

Dogfoods forjar's container build pipeline (E13-E18):

```yaml
resources:
  oci-image:
    type: image
    machine: builder
    name: "{{params.registry_url}}/{{params.image_name}}"
    version: "{{params.image_version}}"
    image: "{{params.base_image}}"
    command: "{{params.app_command}}"
    path: /app/config.yaml
```

Features exercised:
- **E13**: Automatic layer splitting (config vs binary files)
- **E14**: Chunked registry push (16MB chunks for blobs >= 64MB)
- **E16**: Build caching via BLAKE3 input hash
- **E17**: Build metrics output (`build-metrics.json`)
- **E18**: Parallel layer building via `std::thread::scope`

```bash
forjar build -f recipes/96-oci-image-build.yaml oci-image
forjar build -f recipes/96-oci-image-build.yaml oci-image --push
forjar build -f recipes/96-oci-image-build.yaml oci-image --far
```

## 97: Task Pipeline with Quality Gates

Dogfoods forjar's task pipeline mode (E21/F34):

```yaml
resources:
  quality-pipeline:
    type: task
    task_mode: pipeline
    stages:
      - name: lint
        command: "{{params.lint_command}}"
        gate: true
      - name: test
        command: "{{params.test_command}}"
        gate: true
      - name: build
        command: "cargo build --release"
        gate: false
      - name: deploy
        command: "cp target/release/app {{params.deploy_target}}"
        gate: false
```

Gate stages (`gate: true`) abort the pipeline on failure.
Non-gate stages continue regardless.

## 98: Recipe Signing + Registry

Dogfoods recipe supply chain security (FJ-1432/FJ-1426):

```bash
# Sign a recipe with BLAKE3-HMAC
forjar recipe sign recipes/sample.yaml --signer ci

# Verify signature integrity
forjar recipe sign recipes/sample.yaml --verify

# Publish to versioned registry
forjar registry publish --registry ./registry recipes/sample.yaml --version 1.0.0

# Search by tag
forjar registry search --registry ./registry --tag infrastructure
```

Post-quantum dual signing available via `--pq` flag.
