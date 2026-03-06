# Container Build Recipes

Recipes for building OCI container images with forjar's daemonless
image assembler. No Docker daemon required — forjar produces standard
OCI layouts directly from resource definitions.

## Basic Image Build

Build an image with a config file and entrypoint:

```yaml
version: "1.0"
name: app-image
machines:
  builder:
    hostname: build-01
    addr: 127.0.0.1
resources:
  app:
    type: image
    machine: builder
    name: myapp
    version: "1.0.0"
    image: "ubuntu:22.04"      # base image
    command: "/usr/local/bin/app"
    path: /etc/app/config.yaml
```

```bash
forjar build -f forjar.yaml app
```

Output:

```
Building app (myapp:1.0.0)
  Layer 1/1: 1 files, 42 -> 38 bytes

  Image: myapp:1.0.0 (1 layers, 38 bytes)
  Layout: state/images/app
  Built in 0.0s
```

## With Registry Push

Push to an OCI registry after build:

```yaml
resources:
  api-image:
    type: image
    machine: builder
    name: registry.io/team/api
    version: "2.1.0"
    image: "alpine:3.19"
    command: "/app/api-server"
    path: /app/config.toml
```

```bash
forjar build -f forjar.yaml api-image --push
```

The `--push` flag uses OCI Distribution v1.1 protocol:
1. HEAD check for existing blobs
2. POST upload sessions
3. PUT blob uploads
4. PUT manifest with tag

## With Docker Load

Load the built image directly into the local Docker daemon:

```bash
forjar build -f forjar.yaml app --load
```

Requires `docker` or `podman` in PATH.

## FAR Archive Export

Wrap the OCI layout in a FAR (Forjar Archive) for offline distribution:

```bash
forjar build -f forjar.yaml app --far
```

FAR archives use zstd compression with BLAKE3 Merkle verification.

## Base Image Layer Extraction

When a resource has `image:` set, forjar looks for a local OCI layout
at `state/images/<base_ref>/`. If found, base image layers are
extracted and incorporated:

```
state/images/ubuntu_22.04/
  oci-layout
  index.json
  blobs/sha256/...
```

Pre-pull base images with `skopeo`:

```bash
skopeo copy docker://ubuntu:22.04 oci:state/images/ubuntu_22.04
```

## Multi-Layer Images

The layer strategy supports multiple layer types:

```yaml
resources:
  training-image:
    type: image
    machine: builder
    name: myregistry.io/training
    version: "1.0.0-cuda"
    image: "nvidia/cuda:12.4.1-runtime-ubuntu22.04"
    command: "/app/train.sh"
    path: /app/config.yaml
```

Layer types:
- **Files**: Individual files placed at specified paths
- **Packages**: Package lists stored as marker files
- **Build**: Command output captured as layer
- **Derivation**: Nix-style store path export

## OCI Layout Structure

Every `forjar build` produces a standard OCI layout:

```
state/images/<resource>/
  oci-layout                              # {"imageLayoutVersion":"1.0.0"}
  index.json                              # Points to manifest
  manifest.json                           # Docker compat (for docker load)
  blobs/sha256/
    <manifest_digest>                     # OCI manifest
    <config_digest>                       # Image config
    <layer_0_digest> ... <layer_N_digest> # Compressed layers
```

Dual digest strategy:
- **BLAKE3** — forjar store addressing, drift detection, caching
- **SHA-256** (uncompressed) — OCI DiffID in image config
- **SHA-256** (compressed) — OCI layer digest in manifest
