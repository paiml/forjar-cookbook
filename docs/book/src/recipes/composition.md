# Composition Stack Recipes

Multi-recipe compositions that build complete environments by combining
infrastructure, application, and operational recipes. These recipes
validate forjar's ability to orchestrate complex, multi-layered
deployments.

## #53 Development Server

Full development server combining developer workstation (#1) with
monitoring (#4) and security baseline (#9). A single recipe that
provisions a ready-to-use development machine.

**Tier**: 2+3 | **Idempotency**: Strong

## #54 Production Web Stack

Complete web application deployment: web server (#2), database (#3),
cache (#5), and monitoring (#4). Tests multi-machine orchestration
with cross-resource dependencies.

**Tier**: 2+3 | **Idempotency**: Strong

## #55 CI/CD Platform

Full continuous integration platform: CI runner (#6), package repo (#27),
and monitoring (#4). Validates the build-test-deploy pipeline from
infrastructure perspective.

**Tier**: 2+3 | **Idempotency**: Strong

## #56 Secure Production

Production hardening stack: security baseline (#9), log management (#43),
automated patching (#47), and time sync (#44). Defense-in-depth
configuration for production servers.

**Tier**: 2+3 | **Idempotency**: Strong

## #57 Data Platform

Database and analytics stack: PostgreSQL (#3), Redis (#5), and
monitoring (#4). Optimized for data-intensive workloads with
proper backup and cache configuration.

**Tier**: 2+3 | **Idempotency**: Strong

## #58 GPU Compute Cluster

GPU-accelerated compute environment: NVIDIA GPU (#8) or ROCm GPU (#7),
development tools (#1), and monitoring (#4). For ML training and
inference workloads.

**Tier**: 3 | **Idempotency**: Strong | **Requires**: GPU hardware

## #59 Edge Node

Minimal edge deployment: security baseline (#9), time sync (#44),
hostname (#48), and cron (#40). Designed for remote nodes with
limited connectivity and resources.

**Tier**: 2+3 | **Idempotency**: Strong

## #60 Build Farm

Distributed build infrastructure: CI runners (#6), Rust toolchain (#16),
cross-compilation (#19), and package distribution (#29). For large-scale
parallel builds.

**Tier**: 2+3 | **Idempotency**: Strong

## #61 Monitoring Hub

Centralized monitoring and observability: monitoring stack (#4),
log management (#43), and NFS server (#10) for log aggregation.
Central point for fleet-wide visibility.

**Tier**: 2+3 | **Idempotency**: Strong

## #62 Fleet Baseline

Standard fleet baseline applied to every machine: security baseline (#9),
hostname (#48), time sync (#44), automated patching (#47), and
resource limits (#46). The minimum configuration for all servers.

**Tier**: 2+3 | **Idempotency**: Strong
