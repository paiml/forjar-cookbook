# Rust Build Recipes

Declarative Rust compilation pipelines using forjar. These recipes
manage the full build lifecycle from toolchain installation through
compiled binary deployment, including cross-compilation and static
linking.

## #16 Release Build

Standard Rust release build with optimized compilation flags (LTO,
codegen-units=1, strip). Installs the Rust toolchain via the cargo
provider and builds a project in release mode.

**Resources**: rust-toolchain (package/cargo), build-script (file),
artifact-dir (file)

**Tier**: 2+3 | **Idempotency**: Strong | **Grade**: A

## #17 Static MUSL Binary

Statically-linked binary using the `x86_64-unknown-linux-musl` target.
Installs musl-tools, adds the musl target, and cross-compiles for
maximum portability (no glibc dependency).

**Resources**: musl-tools (package), musl-target (package/cargo),
build-script (file)

**Tier**: 2+3 | **Idempotency**: Strong | **Grade**: A

## #19 Cross-Compilation

Cross-compile Rust binaries for different target architectures
(aarch64, armv7). Installs cross-compilation toolchains and
configures cargo for cross-target builds.

**Resources**: cross-toolchain (package), target-config (file),
build-script (file)

**Tier**: 2+3 | **Idempotency**: Strong | **Grade**: A

## #20 Sovereign Stack

Full sovereign infrastructure stack built from source. Compiles all
components of a self-hosted platform (API server, worker, CLI) into
a single deployable artifact set.

**Resources**: source-checkout (file), build-deps (package),
multi-binary-build (file), deploy-dir (file)

**Tier**: 2+3 | **Idempotency**: Strong | **Grade**: A

## #21 APR Model Binary

Compiled APR (Adaptive Pattern Recognition) model binary. Builds
a machine learning inference binary with optimized SIMD support
and deploys it with configuration files.

**Resources**: model-deps (package), build-script (file),
model-config (file), deploy-binary (file)

**Tier**: 2+3 | **Idempotency**: Strong | **Grade**: A
