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

Load the built image directly into the local Docker/Podman daemon:

```bash
forjar build -f forjar.yaml app --load
```

Creates an OCI tarball from the layout directory and pipes it to
`docker load` or `podman load`. Requires `docker` or `podman` in PATH.

## FAR Archive Export

Wrap the OCI layout in a FAR (Forjar Archive) for offline distribution:

```bash
forjar build -f forjar.yaml app --far
```

Produces `state/images/<resource>.far` with:
- zstd-compressed file chunks
- BLAKE3 Merkle tree hash for streaming verification
- Full manifest with file inventory, provenance, and architecture metadata

FAR archives are self-contained and can be transferred to air-gapped environments.

## Sandbox Build (Ephemeral Container)

Build inside an ephemeral Docker/Podman container. The container starts
from the base image, resource scripts run inside it, and filesystem
changes are extracted into an OCI layout:

```bash
forjar build -f forjar.yaml app --sandbox
```

Pipeline:
1. `docker run -d` starts an ephemeral container from the base image
2. `docker exec` applies each resource script
3. `docker diff` extracts added files
4. `docker cp` copies files to a staging directory
5. Overlay scan + image assembler produce the OCI layout
6. Container is force-removed on exit

This produces host-identical builds without modifying the host filesystem.
Requires `docker` or `podman` in PATH.

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

## Automatic Layer Splitting (E13)

Forjar separates config files from binaries into distinct OCI layers.
Config files (`.yaml`, `.toml`, `.json`, `.conf`, `.cfg`, `.ini`,
`.env`, `.properties`) go to a top layer; binaries go to a lower layer.
Only the changed layer needs uploading on push.

## Parallel Layer Building (E18)

Multi-layer images build concurrently using `std::thread::scope`.
Each layer's tar creation and gzip compression runs in parallel.
Single-layer images skip thread overhead.

## Chunked Registry Push (E14)

Blobs under 64 MB use monolithic PUT. Blobs >= 64 MB use OCI chunked
upload protocol: 16 MB PATCH chunks with `Content-Range` headers,
following `Location` between chunks, finalized with PUT + digest.

## Build Caching (E16)

Forjar caches image builds based on BLAKE3 input hashing. On rebuild, if
all layer inputs (file paths, content, permissions) are unchanged, the
build is skipped:

```
$ forjar build -f forjar.yaml app
Building app (myapp:1.0.0) — CACHED
  Layer inputs unchanged (hash: a1b2c3d4...), skipping rebuild
```

The cache is stored in `state/images/<resource>/build-cache.hash`.
Delete this file to force a rebuild.

## Build Metrics (E17)

Every build writes `build-metrics.json` to the output directory with:
- Image tag, layer count, total size
- Per-layer metrics (file count, compressed/uncompressed sizes)
- Build duration, timestamp, forjar version, target architecture

```bash
cat state/images/app/build-metrics.json
```

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

## Image Drift Detection (E15)

After building and deploying an image, forjar can detect drift by
comparing the running container's image digest to the expected
manifest digest from the build:

```bash
forjar drift -f forjar.yaml
```

For each converged image resource, forjar runs:
```
docker inspect <container> --format '{{.Image}}'
```

Drift scenarios:
- **Digest mismatch** — someone pushed a different image
- **Container not running** — expected container has stopped
- **Transport error** — machine unreachable

To skip drift checks for specific images, use lifecycle rules:
```yaml
resources:
  dev-image:
    type: image
    lifecycle:
      ignore_drift: ["*"]
```
