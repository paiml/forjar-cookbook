# Infrastructure Recipes

## #1 Developer Workstation

Installs build tools, cargo utilities, git config, and shell profile.

**Resources**: dev-packages (apt), cargo-tools (cargo), gitconfig (file), profile-d (file)

**Tier**: 2+3 | **Idempotency**: Strong

## #2 Web Application Server

Nginx with custom config, firewall rules, and service management.

## #9 Secure Baseline

SSH hardening, fail2ban, UFW firewall defaults, unattended-upgrades.

**Resources**: security-pkgs (apt), sshd-config (file), fail2ban-config (file), firewall-ssh (network)

**Tier**: 2+3 | **Idempotency**: Strong
