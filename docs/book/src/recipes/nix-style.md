# Nix-Style Recipes

Declarative, reproducible environments using forjar's pepita kernel isolation
instead of the Nix store.

Recipes #11-15 use pepita transport (cgroups v2, overlayfs, netns) to create
isolated development environments, pinned toolchains, and hermetic build
sandboxes.
