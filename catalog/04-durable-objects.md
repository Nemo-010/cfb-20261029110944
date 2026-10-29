# Catalog — Durable Objects alternatives

All GitHub numbers verified via the GitHub REST API on 2026-10-29.
DO semantics to match: keyed singleton actor, single-threaded execution,
per-actor SQLite, alarms, hibernating WebSockets, strong consistency.

## Primary answer

**celld** (see catalog/03) — it is a DO reimplementation, not an
alternative. Everything below is for when you want DO-like primitives
without the Workers programming model, or as fallback.

## Tools

| Tool | Repo | License | Stars | Last push | Verdict |
|---|---|---|---|---|---|
| celld | denoland/celld | Apache-2.0 | 6,169 | 2026-10-21 | **adopt** (Workers/DO model) |
| Rivet | rivet-dev/rivet | Apache-2.0 | 6,487 | 2026-10-29 | **trial** (DO-shaped, no Workers runtime) |
| Temporal | temporalio/temporal | MIT | 24,057 | 2026-10-29 | **adopt** (workflow half only) |
| Restate | restatedev/restate | BSL-1.1 | 4,621 | 2026-10-29 | trial (self-host OK; not OSS) |
| Golem | golemcloud/golem | BSL-1.1 | 1,523 | 2026-10-29 | trial (WASM component model; heavier) |
| Dapr | dapr/dapr | Apache-2.0 | 26,169 | 2026-10-29 | trial (broader runtime; looser actor semantics) |
| Orleans | dotnet/orleans | MIT | 10,918 | 2026-10-29 | adopt if .NET |
| Durable Actors | TerseAI/durable-actors | MIT | 262 | 2026-10-29 | trial (young, literal API clone) |
| workerd standalone | cloudflare/workerd | Apache-2.0 | 9,093 | 2026-10-29 | avoid as prod DO substrate (single-instance DOs) |
| miniflare | cloudflare/miniflare | MIT | 3,926 | 2025-03-13 | avoid (archived; dev-only) |
| PartyKit | partykit/partykit | MIT | 5,742 | 2026-10-09 | avoid here (self-hosts onto Cloudflare DOs) |
| akka-core | akka/akka-core | BSL-1.1 | 13,283 | 2026-10-16 | avoid (license; JVM-centric) |

## Notes

- **Rivet**: virtual actors with per-actor SQLite, ~4.4 ms wake,
  realtime broadcast, cron, actor-to-actor calls; positions itself as
  "an open-source alternative to Cloudflare Durable Objects"; self-host
  = one Rust control-plane binary + S3-compatible bucket; bare-metal/K8s
  guides (https://www.rivet.dev/docs/deploy/self-host/control-plane/).
  Actors run on Node/Bun/Rust — no wrangler compatibility.
- **Restate**: Virtual Objects ≈ keyed stateful handlers; single binary;
  BSL-1.1 permits production self-host (change license = Apache-2.0 four
  years after each release). Lacks WebSocket-hibernation and Workers
  runtime compatibility.
- **Temporal**: the durable-workflow half of DO workloads (CF Workflows
  map ~1:1); no keyed singleton actor; heavy ops (service + DB +
  Elasticsearch).
- **Golem**: durable WASM "agents" keyed by ID; Kubernetes recommended
  for self-host — bigger shift than DOs.

## Decision rule for the spec

1. Workers/DO app code exists or is wanted → celld.
2. Actor workload without the Workers model, Apache license required →
   Rivet.
3. Long-lived multi-step workflows → Temporal (accept the ops weight).
4. Anything BSL → only if the feature gap is decisive and the license
   is acceptable for your use (it is, for internal self-host).

## Sources

GitHub REST API (2026-10-29); LICENSE files fetched raw where license
fields showed NOASSERTION (Restate, Golem, NetBird, Pangolin); docs
links above.
