# Package Distribution Recipes

Linux package build and distribution pipelines. These recipes cover
the full lifecycle from source to signed, published packages deployed
across a fleet.

## #25 Third-Party APT Repository

Add and manage third-party APT repositories with GPG key verification.
Tests the package provider's repository management capabilities with
signed repos and key pinning.

**Resources**: repo-key (file), repo-source (file),
apt-update (package)

**Tier**: 2+3 | **Idempotency**: Strong

## #26 .deb Package Build

Build Debian packages from source using dpkg-deb. Creates the package
directory structure, control file, install scripts, and builds the
.deb artifact.

**Resources**: build-deps (package), package-dir (file),
control-file (file), build-script (file)

**Tier**: 2 | **Idempotency**: Strong

## #27 Private APT Repository

Set up a private APT repository with reprepro for internal package
distribution. Manages repository structure, GPG signing keys, and
distribution configuration.

**Resources**: reprepro-pkg (package), repo-dir (file),
distributions-config (file), gpg-key (file)

**Tier**: 2+3 | **Idempotency**: Strong

## #28 RPM Package

Build RPM packages using rpmbuild. Creates the spec file, source
directory structure, and builds the RPM for Red Hat-family distributions.

**Resources**: rpm-build-deps (package), spec-file (file),
source-dir (file), build-script (file)

**Tier**: 2 | **Idempotency**: Strong

## #29 Distribution Pipeline

End-to-end package distribution pipeline: build, sign, publish to
repository, and deploy across a fleet. Combines package building
with repository management and fleet-wide deployment.

**Resources**: build-stage (file), sign-stage (file),
publish-stage (file), fleet-deploy (file)

**Tier**: 2+3 | **Idempotency**: Strong
