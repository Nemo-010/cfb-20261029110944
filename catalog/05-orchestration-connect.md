# Catalog — orchestration, tunnels, Access

All GitHub numbers verified via the GitHub REST API on 2026-10-29
unless marked ~. Verdicts assume the SPEC constraint: dashboard stays
on Cloudflare; machines dial out; nothing self-hosted is load-bearing
for the UI path.

## Transport & zero-trust

| Tool | Repo | License | Stars | Last push | Verdict |
|---|---|---|---|---|---|
| cloudflared | cloudflare/cloudflared | Apache-2.0 | 16,336 | 2026-10-29 | **adopt** (edge path; API-provisioned) |
| Headscale | juanfont/headscale | BSD-3-Clause | 45,164 | 2026-10-29 | **adopt** (overlay) |
| Authelia | authelia/authelia | Apache-2.0 | 29,437 | 2026-10-29 | **adopt** (origin-side identity) |
| Pangolin | fosrl/pangolin | AGPL-3.0 + commercial dual | 23,263 | 2026-10-29 | trial (Tunnel+Access escape hatch) |
| NetBird | netbirdio/netbird | BSD-3 client / AGPLv3 control dirs | 30,316 | 2026-10-29 | trial |
| Pomerium | pomerium/pomerium | Apache-2.0 | 5,081 | 2026-10-29 | trial (conflicts with in-path cloudflared) |
| frp | fatedier/frp | Apache-2.0 | 110,280 | 2026-10-23 | trial (raw transport fallback) |
| rathole | rathole-org/rathole | Apache-2.0 | 14,382 | 2026-08-23 | avoid (frp covers it; slower cadence) |

## License verifications (fetched raw, 2026-10-29)

- **NetBird**: "This BSD-3-Clause license applies to all parts of the
  repository except for the directories management/, signal/, relay/
  and combined/. Those directories are licensed under the AGPLv3."
- **Pangolin**: "Portions of this software are licensed as follows"
  (Fossorial, Inc. 2025; AGPL-3.0 + Fossorial Commercial License per
  the file body) — dual-licensed by the vendor; future open-core
  squeeze risk.
- **Headscale**: BSD-3-Clause; self-hosted Tailscale control server;
  official clients connect to it.

## Why keep cloudflared (and when not to)

The dashboard must be hosted on Cloudflare and manage everything from
there — so the edge path (public hostnames, Access policies, DNS) is
already Cloudflare's. cloudflared completes that loop with zero inbound
ports and is provisioned by the CF API itself (tunnel creation from the
API Worker). Replace it only if Cloudflare-as-transport becomes the
risk: then Pangolin (tunnel + identity + dashboard, self-hosted) or
frp + Authelia. The spec keeps that swap behind one abstraction
(SPEC §7, last row).

## Orchestration layer

The SPA does not need a full scheduler; the control plane is:

1. **API Worker (CF)** — identity, machine registry, tunnel/DNS
   provisioning via CF API.
2. **celld fleet (your metal)** — the actual orchestration substrate for
   Workers/DO-shaped work (placement via bucket leases).
3. **Per-host daemons** — Woodpecker agents (CI), sandbox daemons
   (microsandbox/msb), storage (SeaweedFS/Garage), Headscale.

Kubernetes/Nomad were deliberately rejected for v1: K3s (Apache-2.0) is
the documented fallback if container orchestration beyond celld's cells
is genuinely needed, but Nomad is BUSL-1.1 since the HashiCorp license
change and adds a control plane the SPA would then have to front.
Machine provisioning (hypervisor-level) is left to existing tooling on
site (Proxmox/Incus); the spec's onboarding flow (SPEC §5) consumes
whatever a site has.

## Sources

GitHub REST API (2026-10-29); LICENSE files fetched raw; Tailscale
opensource page (https://tailscale.com/opensource).
