# Networking & VPN Recipes

Recipes for network infrastructure: VPN tunnels, edge clusters,
and observability pipelines.

## 88: WireGuard VPN Mesh

Hub-and-spoke WireGuard VPN with peer config, key generation,
firewall rules, and health monitoring.

```yaml
resources:
  wg-conf-hub:
    type: file
    path: /etc/wireguard/wg0.conf
    mode: '0600'
    content: |
      [Interface]
      Address = {{params.hub_vpn_addr}}/24
      ListenPort = {{params.vpn_port}}
      PostUp = wg set %i private-key /etc/wireguard/private.key

      [Peer]
      PublicKey = {{params.spoke_public_key}}
      AllowedIPs = {{params.spoke_vpn_addr}}/32
```

Key features:
- Multi-machine: hub + spoke topology
- Key generation scripts (idempotent — skip if keys exist)
- UDP firewall rules for WireGuard port
- Health check cron (peer count, tunnel status)

```bash
forjar check -f recipes/88-wireguard-vpn.yaml
forjar apply -f recipes/88-wireguard-vpn.yaml --tag vpn
```

## 93: K3s Lightweight Kubernetes

K3s edge cluster: server node with agent join, kubeconfig,
Helm, and Traefik ingress.

```yaml
machines:
  server:
    hostname: k3s-server
    addr: 10.0.1.10
  agent:
    hostname: k3s-agent
    addr: 10.0.1.11
```

Downloads K3s installer to disk first (no `curl|bash` pipe) for
SAF compliance. Server and agent nodes configured in parallel.

## 95: OpenTelemetry Collector

OTLP receiver pipeline with Prometheus exporter, batch processor,
and memory limiter. Firewall rules for gRPC (4317), HTTP (4318),
and Prometheus scrape (8889) endpoints.
