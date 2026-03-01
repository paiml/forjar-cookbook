# Runner Setup

## Intel Runner (Primary)

The self-hosted Intel runner is the primary qualification target.

| Component | Spec |
|-----------|------|
| OS | Ubuntu 22.04 (Jammy), kernel 6.8.0-101-generic |
| CPU | 32-core Intel Xeon W |
| RAM | 283 GB |
| Storage | 3.6 TB NVMe RAID-0 |
| GPU | 2x AMD Radeon Pro W5700X (Navi 10) |
| Docker | 29.2.1 |

## Connecting

```bash
ssh intel
cd ~/src/forjar-cookbook
make qualify-all
```
