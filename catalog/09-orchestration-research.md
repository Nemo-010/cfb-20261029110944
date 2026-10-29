# Catalog — orchestration research (Rivet-class, adopted vs rejected)

Verified via GitHub API 2026-10-29. Sources: rivet.dev docs, Golem
learn docs, Dapr/Temporal/Hatchet/Inngest/Nomad/DBOS docs, s6/systemd
freedesktop docs, Komodo docs, Docker docs.

## Rivet (rivet-dev/rivet — Rust, Apache-2.0, pushed 2026-10-29)

- Placement: allocation from a process-local Envoy directory cache of
  outbound-connected nodes (1 s fresh / 5 s max stale); node pools
  with metadata hints; prefers highest node version; placement gated
  on `destination_stability_ms` (new node must hold discovery + pass a
  disk probe beyond lease+renewal+takeover grace, ~35 s).
- Routing: actor keys (`getOrCreate(["room","general"])`); hashed
  index-bucket volumes per (namespace, actor name), default 8 buckets;
  immutable key reservations; gateway holds requests until a start
  notice (2 s) with polling fallback.
- Liveness: outbound Envoy WebSocket per node; `envoy_lost_threshold`
  40 s advertised to the node at connect, must exceed the 30 s
  reconnect backoff; reconnect shedding with retry-after hints;
  gateway retry budget capped at 10%.
- Fencing: actor generations on commands; key epochs; leases/
  heartbeats/manifests as conditional PUTs on the object bucket.
- Lifecycle: `sleepTimeout` 30 s idle → sleep; `sleepGracePeriod` 15 s
  single budget; `keepAwake(p)` vs `waitUntil(p)`; WebSocket
  hibernation across sleep/migration (beta); persisted alarms re-armed
  on wake.
- Upgrades: monotonic node versions ("versions only go up");
  `drain_on_version_upgrade`; eviction delay 5 s / period 60 s /
  rate 1 actor/s; SIGTERM → onSleep → reschedule; ≥60 s SIGTERM→SIGKILL.
- Substrate: self-host control plane = N identical instances + one S3
  bucket, no DB, no message bus — validates cfb's bucket-only core.

## Golem (golemcloud/golem — BSL-1.1)

1024 fixed shards, agent ID → shard; routing table published by shard
manager; executor leases ~1 min, renewed ~1/3 TTL; renewals carry
per-shard epochs, assignment pushes carry revisions; oplog writes
atomically check the epoch — a lease-lost executor cannot append.
Virtual-actor hibernation = memory eviction + oplog replay on next
event; recovery and migration share the replay path.

## Nomad (hashicorp/nomad — BUSL)

`disconnect` block: missed heartbeat → alloc `unknown` → `lost` after
`heartbeat_grace`; `lost_after` lets a partitioned node keep running
while a replacement starts; on reconnect a `reconcile` policy
(`keep_original`/`keep_replacement`/`best_score`/`longest_running`)
resolves the split. Evaluate→plan→apply with leader plan queue
(rejected here: the lease already serializes).

## Dapr (dapr/dapr — Apache-2.0)

Placement service computes a hash-partition table per actor type; every
sidecar caches it and routes directly; table pushed as instances
change. Reminders (persisted, reactivate idle actors) vs timers
(in-memory, warm only). Turn-based per-actor lock, reentrancy opt-in.

## Temporal (temporalio/temporal — MIT) / Hatchet (hatchet-dev/hatchet — MIT)

Sticky execution: per-worker sticky queue + in-memory workflow cache;
not started within `StickyScheduleToStartTimeout` (5 s default) →
fall back to the normal queue; failure invalidates the cache. Hatchet
adds sticky child-task assignment reusing parent's worker state.
Temporal Worker Versioning: Pinned vs Auto-Upgrade workflows, current/
ramping versions with drain states. Pull model: workers poll.

## Inngest (inngest/inngest — NOASSERTION)

Two-tier queue (per-function queues + latency-ordered tier above),
weighted random shuffle of peeked work; idempotent enqueue against
at-least-once streams. Tier mechanics rejected (multi-tenant problem);
idempotent enqueue adopted.

## s6 / systemd (ISC / GPL-2.0; sd_notify protocol MIT-0)

Readiness = one newline on a reserved fd (`s6-notifywhenup`,
`READY=1`); watchdog keepalive with manager-enforced deadline
(`WatchdogSec`, `WATCHDOG=1`); FD holding across restarts (s6-fdholder,
`FDSTORE=1`); `./finish` post-exit hooks.

## Docker Engine (moby/moby — Apache-2.0)

Restart backoff doubling from 100 ms capped at 1 min, reset after 10 s
healthy; `HEALTHCHECK` start-period grace before failures count.
Routing mesh/VIP gossip rejected (registry + peer HTTP suffices).

## Komodo (moghtech/komodo — GPL-3.0, Rust)

Stateless per-host agent over bidirectional outbound WebSocket;
one-time onboarding key bootstraps a pinned Noise keypair with auto
rotation. Confirms the broker/enroll shape; Git-ops reconcilers noted
as a desired-state alternative (not adopted v1).

## Verdict

Bucket-lease core: kept — correct at 2–10 hosts; Rivet independently
converged on bucket-only coordination. Ten mechanisms adopted as
normative rules in SPEC §6.1; consensus groups, plan queues, fair
queues, routing meshes, and per-actor storage engines rejected with
reasons in SPEC §6.1.
