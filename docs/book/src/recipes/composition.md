# Composition Stack Recipes

Multi-recipe compositions that build complete environments by combining
infrastructure, application, and operational recipes. These recipes
validate forjar's ability to orchestrate complex, multi-layered
deployments without conflict.

## #53 Dev Server

Composability test: Dev Server stack. Layers secure baseline with
developer workstation and toolchain pin. Validates that recipes
compose without conflict on a single machine.

**Tier**: 1+2 | **Idempotency**: Strong | **Grade**: A

## #54 Web Production

Composability test: Web Production stack. Layers security, web server,
database, monitoring, secrets, and TLS. The most complex composition
— validates deep dependency chains across six layers.

**Tier**: 1+2 | **Idempotency**: Strong | **Grade**: A

## #55 GPU Lab

Composability test: GPU Lab stack. Layers dev tools, GPU configuration,
monitoring, and security. Validates GPU resource composability with
base infrastructure layers.

**Tier**: 1+2 | **Idempotency**: Strong | **Grade**: A | **Requires**: GPU hardware

## #56 Build Farm

Composability test: Build Farm stack. Layers build sandbox, static musl,
and cross-compilation. Validates Rust build recipe composition for
distributed build infrastructure.

**Tier**: 1+2 | **Idempotency**: Strong | **Grade**: A

## #57 Package Pipeline

Composability test: Package Pipeline stack. Layers static build, deb
package, APT repo, and fleet provisioning. Full pipeline from
build to package to distribute to deploy.

**Tier**: 1+2 | **Idempotency**: Strong | **Grade**: A

## #58 ML Inference

Composability test: ML Inference stack. Layers security, GPU, APR model
serving, monitoring, and secrets. Validates GPU plus model serving
composition for inference workloads.

**Tier**: 1+2 | **Idempotency**: Weak | **Grade**: A | **Requires**: GPU hardware

## #59 CI Infrastructure

Composability test: CI Infrastructure stack. Layers security, CI runner,
and monitoring. Validates CI runner composition with base security
and observability layers.

**Tier**: 1+2 | **Idempotency**: Strong | **Grade**: A

## #60 Sovereign AI

Composability test: Sovereign AI stack. Layers security, dev tools, GPU,
model serving, monitoring, and secrets. The most comprehensive
composition — validates deep six-layer stacking.

**Tier**: 1+2 | **Idempotency**: Strong | **Grade**: A | **Requires**: GPU hardware

## #61 Fleet Baseline

Composability test: Fleet Baseline stack. Layers security, fleet
provisioning, and monitoring. Validates fleet-wide baseline
composition applied to every machine.

**Tier**: 1+2 | **Idempotency**: Strong | **Grade**: A

## #62 Cross-Distro

Composability test: Cross-Distro Release stack. Layers static build,
deb package, RPM build, and distribution pipeline. Validates
multi-format package release composition.

**Tier**: 1+2 | **Idempotency**: Strong | **Grade**: A
