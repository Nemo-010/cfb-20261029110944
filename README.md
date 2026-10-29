# cfb — control plane on Cloudflare, payloads on your own metal

A spec + tool catalog for self-hosted replacements of Cloudflare's paid
services — R2, Workers, Durable Objects, Sandboxes/Containers/Computer,
and heavy CI — where the dashboard/manager **is** hosted on Cloudflare
itself (a SPA served by a Worker, gated by Cloudflare Access), and your
own machines (home lab / colo) run the actual payloads, reached over
Cloudflare Tunnels with a WireGuard-style overlay for peer traffic.

**The one-sentence verdict:** the architecture is now genuinely viable
because of `denoland/celld` (Apache-2.0) — a self-hosted, distributed
implementation of Workers + Durable Objects + KV + D1 + R2 + Queues +
Workflows that deploys from your existing `wrangler.json` — and on
2026-10-09 Cloudflare announced the Deno team is joining to merge celld
into workerd and make self-hosting a first-class supported mode.

## Contents

| File | What it is |
|---|---|
| [`SPEC.md`](SPEC.md) | The architecture spec: components, flows, security model, per-service mapping, rollout |
| [`catalog/`](catalog/) | Verified tool catalogs per lane (7 files, all GitHub numbers API-checked 2026-10-29) |
| [`CRITIQUE.md`](CRITIQUE.md) | Adversarial critique: what is wrong with this spec and what it honestly cannot do |

## The stack, in one table

| Cloudflare service | Replacement on your metal | License | Verdict |
|---|---|---|---|
| Workers + Durable Objects + KV + D1 + Queues + Workflows | [`denoland/celld`](https://github.com/denoland/celld) (merging into `cloudflare/workerd`) | Apache-2.0 | adopt |
| R2 | [SeaweedFS](https://github.com/seaweedfs/seaweedfs) (scale) or [Garage](https://github.com/deuxfleurs-org/garage) (homelab) | Apache-2.0 / AGPL-3.0 | adopt |
| Sandboxes / Containers / Computer | [microsandbox](https://github.com/superradcompany/microsandbox), or [E2B runtime Embed](https://github.com/e2b-dev/runtime) (single node), on a [Firecracker](https://github.com/firecracker-microvm/firecracker)/[gVisor](https://github.com/google/gvisor) substrate | Apache-2.0 | trial |
| Tunnel | [cloudflared](https://github.com/cloudflare/cloudflared) (stays — that is the point) | Apache-2.0 | adopt |
| (optional) Access on origin side | [Headscale](https://github.com/juanfont/headscale) overlay + [Authelia](https://github.com/authelia/authelia) | BSD-3 / Apache-2.0 | adopt |
| Heavy CI | [Woodpecker CI](https://github.com/woodpecker-ci/woodpecker) (+ [Dagger](https://github.com/dagger/dagger) as engine) | Apache-2.0 | adopt |
| Heavy CI, Actions-compatible | [Gitea + Gitea Actions](https://github.com/go-gitea/gitea) | MIT | trial |
| Durable execution (workflow half of DOs) | [Temporal](https://github.com/temporalio/temporal) | MIT | adopt (only if needed) |

## Landmines found during research (2026-10-29)

These are verified against the GitHub API, not hearsay:

- **MinIO is archived** (`minio/minio`: `archived: true`, pushed 2026-04-24, AGPL-3.0, 61,300★). Do not base anything on it. Successors: SeaweedFS, Garage, RustFS, VersityGW.
- **Daytona is archived** (`daytonaio/daytona`: `archived: true`, pushed 2026-07-24, no license, 71,443★) after going closed-source in June 2026. The highest-starred "E2B alternative" list topping is a trap.
- **cloudflare/miniflare is archived** (pushed 2025-03-13); the live simulator lives inside `cloudflare/workers-sdk`. Dev-only either way.
- **gofixpoint/amika** is architecturally the closest match to "provision machines and remote-control agents" but its control plane, UI and relay are **closed source**; only `amikad`, `amika-hostd` (which runs workloads as [smolvm](https://github.com/smol-machines/smolvm) microVMs), the CLI, TS SDKs and `@amika/service-auth` are open. Its full self-host path (`amika-gateway`) is "coming soon" and single-player. Reuse its pieces, do not build on its control plane.

## Method

Seven parallel research lanes (storage, sandbox/compute, workerd/Workers,
Durable Objects, orchestration/tunnels/Access, CI/control-plane
precedents, amika deep dive), each using a tinyfish-pinned search
gateway; every GitHub number in this repo was then re-verified directly
against the GitHub REST API on 2026-10-29, and load-bearing web claims
against primary sources (Cloudflare blog, product docs, LICENSE files
fetched raw). Anything that could not be confirmed is labeled unverified
in the catalogs.

License: 0BSD.
