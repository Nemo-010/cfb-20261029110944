# cfb — CF-hosted control plane, single-agent payloads, portable code

Spec + catalogs for self-hosting Cloudflare R2 / Workers / Durable
Objects / Sandboxes / CI with the dashboard on Cloudflare. Two hard
requirements:

1. **Portability**: application code runs unchanged on Cloudflare
   Workers or on self-hosted backends. Swapping = editing
   `wrangler.json` targets, not editing code.
2. **One agent per server**: a single lightweight binary (cfrs-shaped)
   owns tunnel, broker, monitoring and supervision. No fleet of
   separate daemons.

All GitHub numbers verified via the API 2026-10-29. License: 0BSD.

| File | Content |
|---|---|
| [SPEC.md](SPEC.md) | Architecture: portability contract, agent spec, control plane, security |
| [catalog/](catalog/) | Verified tool catalogs (8 files) |
| [CRITIQUE.md](CRITIQUE.md) | Rejected alternatives, known gaps |

## Stack

| CF service | Self-hosted | License |
|---|---|---|
| Workers + DOs + KV + D1 + Queues + Workflows | [celld](https://github.com/denoland/celld) (merging into [workerd](https://github.com/cloudflare/workerd)) | Apache-2.0 |
| R2 | [SeaweedFS](https://github.com/seaweedfs/seaweedfs) / [Garage](https://github.com/deuxfleurs-org/garage) | Apache-2.0 / AGPL-3.0 |
| Tunnel | [cloudflared](https://github.com/cloudflare/cloudflared); native edge client per [cfrs](https://github.com/talaria0101/cfrs) when mature | Apache-2.0 / 0BSD |
| Overlay | [Headscale](https://github.com/juanfont/headscale) | BSD-3-Clause |
| Sandboxes | [microsandbox](https://github.com/superradcompany/microsandbox) / [gVisor](https://github.com/google/gvisor) | Apache-2.0 |
| CI | [Woodpecker](https://github.com/woodpecker-ci/woodpecker) (+ [Dagger](https://github.com/dagger/dagger)) | Apache-2.0 |
| Agent (per host) | cfb agent — new, Rust; designs from cfrs, ServerBee, mirrord | 0BSD |
