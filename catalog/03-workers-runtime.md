# Catalog — Workers runtime on your own metal

All GitHub numbers verified via the GitHub REST API on 2026-10-29.

## The headline

**workerd is open source (Apache-2.0, 9,093★, pushed 2026-10-29) and is
the same code Cloudflare runs in production** — but on its own it is a
runtime, not a platform, and its Durable Objects support is
single-instance only ("good enough for local testing" — Kenton Varda,
https://blog.cloudflare.com/deno-joins-cloudflare/).

On 2026-10-09 Cloudflare announced the Deno team joined Cloudflare to
**merge celld into workerd** and make workerd self-hosting a
first-class supported way to build and run apps. Until the merge ships,
**celld is the distributed-DO answer** and raw workerd is a fallback
for non-DO services.

## Tools

| Tool | Repo | License | Stars | Last push | Verdict |
|---|---|---|---|---|---|
| workerd | cloudflare/workerd | Apache-2.0 | 9,093 | 2026-10-29 | adopt (merged future) / fallback runtime |
| **celld** | denoland/celld | Apache-2.0 | 6,169 | 2026-10-21 | **adopt — the payload runtime** |
| Miniflare | cloudflare/miniflare | MIT | 3,926 | 2025-03-13 | dev only — **standalone repo archived**; live version inside cloudflare/workers-sdk |
| open-compute | elliothux/open-compute | Apache-2.0 | 2,433 | 2026-10-28 | trial (3 months old, single maintainer, claims unverified) |
| denoflare | skymethod/denoflare | MIT | 858 | 2026-10-25 | trial (dev tooling) |
| Spin | spinframework/spin (ex fermyon/spin) | Apache-2.0 | 6,560 | 2026-10-23 | trial (Wasm FaaS, not Workers-compatible) |
| wasmCloud | wasmCloud/wasmCloud | Apache-2.0 | 2,471 | 2026-10-28 | trial (heavy; only for actor-model scope) |
| OpenWorkers | openworkers/openworkers-infra | null (claimed MIT, unverified) | 39 | 2026-10-14 | trial (pattern reference) |
| llrt | awslabs/llrt | Apache-2.0 | 8,831 | 2026-10-29 | trial (component only) |
| self-workerd | giuseppelt/self-workerd | MIT | 141 | 2025-06-23 | avoid (dormant PoC) |
| OpenFaaS | openfaas/faas | NOASSERTION (split across repos) | 26,254 | 2026-10-23 | avoid for this lane (container FaaS ≠ Workers model) |

## celld, precisely (verified from README, 2026-10-29)

- Runs Workers, **Durable Objects** (SQLite storage, alarms, hibernating
  WebSockets), KV, Queues, D1, R2 bindings, Workflows, cron triggers,
  static assets — deployed from the **existing `wrangler.json`**.
- One Rust binary; V8 embedded; nodes coordinate via a shared
  S3/GCS/Azure bucket: conditional bucket writes as leases, no control
  plane, no consensus service. Fleet = nodes sharing one bucket.
- Write latency: single node waits on bucket (~90 ms region-local);
  ≥2 nodes prove durability peer-to-peer first (~25 ms with a second
  node). **Run ≥2 nodes per site.**
- Peer listener must sit on a trusted private network / encrypted
  overlay (WireGuard/Tailscale named in docs) — matches the spec's
  Headscale plan.
- Experimental: Containers (`@cloudflare/containers`) and the Sandbox
  SDK (`@cloudflare/sandbox`) as DO-supervised cells.
- Explicitly out of scope per its docs: CF network features, GPUs,
  browser farms.
- Contribution model: PRs disabled; patches by email + CLA to
  Deno Land Inc. Bus-factor caveat; watch the workerd merge.

## Exposing workerd/celld through a tunnel (your question)

Yes — this is the intended shape. celld's docs put the public listener
behind a tunnel-capable front and the peer ports behind an overlay; a
cloudflared hostname → `localhost:<port>` for the public listener with
an Access service-token policy in front is exactly compatible. A
"machine that runs workerd and exposes it via the tunnel" is viable
today as: celld node (Workers+DO) or raw workerd (plain HTTP services
only, no distributed DOs), single cloudflared per host.

## Sources

- https://github.com/denoland/celld (README, fetched 2026-10-29)
- https://blog.cloudflare.com/deno-joins-cloudflare/ (read in full)
- https://blog.cloudflare.com/workerd-open-source-workers-runtime/
- GitHub REST API, 2026-10-29
