# WASM Plugins

WASM resource plugins extend forjar with compiled WebAssembly modules
for type-safe, sandboxed resource management.

## Scaffolding a Plugin

```bash
# Create plugin scaffold
forjar plugin init my-plugin

# Creates:
# plugins/my-plugin/
#   plugin.yaml       # Manifest with BLAKE3 hash
#   plugin.wasm        # Stub WASM module
```

### Manifest Format

```yaml
name: my-plugin
version: "0.1.0"
description: "My custom resource plugin"
abi_version: 1
wasm: plugin.wasm
blake3: "abc123..."   # BLAKE3 hash of WASM binary
schema:
  required:
    - name
    - version
  properties:
    name:
      type: string
    version:
      type: string
    replicas:
      type: integer
```

## Using WASM Plugins

```yaml
resources:
  my-deployment:
    type: "plugin:my-plugin"
    name: web-app
    version: "1.0.0"
    replicas: 3
```

## Plugin Lifecycle

```bash
# List installed plugins
forjar plugin list

# Verify BLAKE3 integrity
forjar plugin verify plugins/my-plugin/plugin.yaml

# Initialize new plugin project
forjar plugin init my-new-plugin --output ./plugins/my-new-plugin
```

## Plugin Dispatch

The plugin system uses BLAKE3-verified manifests to ensure supply chain
integrity. Each plugin operation:

1. **Resolve**: Find manifest in plugin directory
2. **Verify**: Check BLAKE3 hash of WASM binary matches manifest
3. **Schema validate**: Check resource properties against schema
4. **Dispatch**: Execute check/apply/destroy via WASM ABI

## Security Model

- WASM sandboxing prevents filesystem/network access
- BLAKE3 integrity verification on every load
- ABI version checking for forward compatibility
- Schema validation catches invalid resource configurations
