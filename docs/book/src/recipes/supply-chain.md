# Supply Chain & Security Recipes

Recipes demonstrating forjar's supply chain security toolkit: SLSA provenance,
Merkle lineage, convergence proofs, fault injection, security scanning,
privilege analysis, and BOM generation.

## #79 Provenance Attestation

Supply chain assurance pipeline combining three layers of verification:
SLSA provenance attestation, Merkle DAG lineage, and convergence proofs.

**Tier**: 1+2 | **Idempotency**: Strong

Key commands exercised:
- `forjar prove` — 5 convergence proof obligations
- `forjar provenance` — SLSA attestation with BLAKE3 hashes
- `forjar lineage` — Merkle DAG content-addressed tree
- `forjar privilege-analysis` — least-privilege audit

## #80 Fault Injection Gate

Pre-deploy resilience gate that simulates infrastructure failures before they
happen. Tests network timeouts, disk-full conditions, permission errors,
dependency cascades, and idempotency across all resources.

**Tier**: 1+2 | **Idempotency**: Strong

Key commands exercised:
- `forjar fault-inject` — 5 fault categories per resource
- `forjar invariants` — structural invariant verification
- `forjar cost-estimate` — deployment time estimation

## #81 Security Hardening

Security analysis pipeline running static security scan, privilege analysis,
SBOM generation, and CBOM (Cryptographic Bill of Materials) for compliance
auditing.

**Tier**: 1+2 | **Idempotency**: Strong

Key commands exercised:
- `forjar security-scan` — static IaC security scanner
- `forjar privilege-analysis` — minimum privilege audit
- `forjar sbom` — Software Bill of Materials
- `forjar cbom` — Cryptographic Bill of Materials

## #82 MLOps Pipeline

Full ML lifecycle management: model card generation, data freshness
monitoring, data validation, checkpoint management, dataset lineage
tracking, reproducibility proofs, and data sovereignty compliance.

**Tier**: 1+2 | **Idempotency**: Strong

Key commands exercised:
- `forjar model-card` — ML model documentation
- `forjar data-freshness` — stale artifact detection
- `forjar data-validate` — declarative data checks
- `forjar checkpoint` — training checkpoint management
- `forjar dataset-lineage` — dataset versioning
- `forjar repro-proof` — reproducibility certificate
- `forjar sovereignty` — data sovereignty compliance

## #83 Config Merge & Extract

Configuration composition operations: merge multiple configs into one,
extract resource subsets by tag, analyze cross-machine dependencies,
and export for air-gapped deployment.

**Tier**: 1+2 | **Idempotency**: Strong

Key commands exercised:
- `forjar config-merge` — merge two configs
- `forjar extract` — extract by tag/group/glob
- `forjar cross-deps` — cross-machine dependency analysis
- `forjar iso-export` — air-gapped deployment export

## #84 Saga Multi-Stack

Saga-pattern multi-stack deployment with ordered apply, state generations
for rollback, preservation checks, and stack dependency graph visualization.

**Tier**: 1+2 | **Idempotency**: Strong

Key commands exercised:
- `forjar stack-graph` — stack dependency visualization
- `forjar preservation` — no-destructive-change verification
- `forjar generation list` — state generation audit
- `forjar registry-list` — available recipe registry

## #85 Brownfield Import

Adopt existing unmanaged infrastructure into forjar management. State
reconstruction from event logs, service catalog listing, agent registry,
and recipe signing for trust establishment.

**Tier**: 1+2 | **Idempotency**: Strong

Key commands exercised:
- `forjar import-brownfield` — discover and import existing state
- `forjar catalog-list` — available service catalog
- `forjar agent-registry` — registered forjar agents
- `forjar sign` — recipe signing for trust chain
- `forjar state-reconstruct` — rebuild state from events
