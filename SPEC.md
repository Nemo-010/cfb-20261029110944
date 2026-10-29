# SPEC

v0.2.0 — 2026-10-29. All external facts API- or primary-source-verified
2026-10-29. Normative language: MUST / SHOULD / MAY.

## 1. Requirements

R1 Portability: app code written for this architecture MUST run
    unchanged on Cloudflare Workers and on self-hosted backends.
R2 One agent: every payload host runs exactly one agent binary that
    owns tunnel, broker, monitoring, and payload supervision.
R3 Control plane on Cloudflare: SPA served by a Worker behind Access;
    all management flows through it.
R4 Payload hosts: no public inbound ports.
R5 No heavy per-host stacks: no Kubernetes, no shared-DB agents, no
    multi-container dashboards on payload hosts.

## 2. Portability contract (R1)

The contract is layered; each layer names what code may assume.

**L0 — runtime**: all application code targets the Workers programming
model (fetch/queue/cron handlers, Durable Objects, bindings declared in
`wrangler.json`). Deploy targets:

| Target | Runtime | Same `wrangler.json` |
|---|---|---|
| Cloudflare | Workers | yes |
| Own metal | celld (→ merged workerd) | yes |
| Other self-hosted runtimes | out of scope v1 | — |

App code MUST restrict itself to the binding surface both targets
implement: DOs (SQLite storage, alarms, hibernating WebSockets), KV,
D1, Queues, Workflows, cron, static assets, R2-style object storage.
CF-only products (Workers AI, Analytics Engine, Hyperdrive, Durable
Objects access to CF-only beta APIs, Browser Rendering) MUST NOT be
imported without a capability check; a shared `compat.ts` feature
probe is the single gate. Enforcement: lint rule rejecting
CF-only imports outside `compat.ts`.

**L1 — storage**: object storage access is S3/SigV4 over standard
endpoints. Targets: R2 (S3 endpoint), SeaweedFS, Garage, any S3 store.
R2-native bindings are permitted only for CF-side UI state. Relational
state lives in D1 (both targets implement SQLite/SQL semantics) or a
self-hosted Postgres behind the same interface.

**L2 — control plane split**: the SPA + API Worker is thin and
CF-resident; every payload-facing operation goes through an interface
(`Origin` trait / abstract endpoints) with two implementations:
CF-managed tunnel routes and agent broker routes. Adding a backend =
implementing the interface, not editing the SPA.

**L3 — swap procedure**: moving a service from CF to self-hosted =
(1) point `wrangler.json` deploy target at the celld fleet, (2) point
storage binding at the S3 endpoint, (3) redeploy. No source edits;
`compat.ts` probes decide feature availability at runtime.

## 3. Topology

```
 browser ──▶ CF edge: Access ─▶ SPA (Worker static assets)
                                  │ same-origin /api/*
                                  ▼
                             API Worker  ──CF API──▶ tunnels, DNS, Access apps
                                  │
             ┌────────────────────┼───────────────────┐
             ▼ outbound only      ▼                   ▼
        [cfb agent]           [cfb agent]         [cfb agent]
        host A                host B              host C
        ├ tunnel              ├ tunnel            ├ tunnel
        ├ broker WS           ├ broker WS         ├ broker WS
        ├ monitor             ├ monitor           ├ monitor
        └ supervised payloads: celld nodes, Woodpecker agent,
          sandbox daemon, object store
```

Peer/bulk traffic: Headscale overlay between agents. The agent
supervises the overlay client; it does not implement WireGuard.

## 4. The agent (one binary per host)

Rust, static single binary. Roles, in priority order:

| Module | Function | Design source |
|---|---|---|
| `transport` | tunnel: (a) bundled cloudflared, managed (spawn/config/token rotation) — default; (b) native CF edge client — optional, adopt when a Rust implementation of the edge transport is edge-tested (cfrs `tunnel` is NOT: no serving loop, untested against the real edge, 2026-10-29). Fallback relay path: SSH remote-forward splice on 443 to a unix-socket origin (cfrs `serve`, working) | cfrs |
| `broker` | one persistent, outbound, versioned multiplexed connection to the API Worker; carries control commands, metrics, logs, sessions. Version negotiation on connect | mirrord `protocol-api` / operator-websocket |
| `enroll` | one-time enrollment offer (≤10 min TTL, single use) → agent generates its own run token before claiming → token persisted, auto-reconnect, never re-enrolls; host keypair pinned at enroll | ServerBee handshake + omnyssh Ed25519 bootstrap (with `sshd_config` backup/rollback) |
| `monitor` | self-reported host metrics (CPU/mem/disk/net/load/temp), 3 s push over broker; alert rules evaluated server-side | ServerBee / Glances data model |
| `doctor` | capability probe at enroll + on demand: KVM present, ports, egress class, disk, versions; returned as the host's capability table | cfrs `doctor` |
| `gates` | per-command capability gates with host-local temporary grants; least-privilege remote execution | ServerBee |
| `supervise` | child-process supervision of payload daemons (celld node, Woodpecker agent, sandbox daemon, store) — one binary owns them; payloads stay stock upstream binaries, agent only supervises/configures | — |
| `ssh` | optional operator terminal over broker; Ed25519 identity managed at enroll | omnyssh |

Normative constraints:

- Metrics flow agent → broker → control-plane storage. Agents MUST
  NOT write to a shared database directly (CoreObs anti-pattern).
- The agent MUST NOT implement CF-only edge semantics itself beyond
  the pluggable `transport`; tunneling correctness stays with a
  tested implementation.
- The agent exposes zero listening ports by default; the unix-socket
  origin pattern is the default local interface.
- Config: TOML file + `CFB_`-prefixed env overrides with `__` nesting
  (ServerBee convention).

## 5. Control plane on Cloudflare

- SPA: static assets on a Worker, same-origin `/api/*`.
- API Worker: terminates broker connections, proxies tunnel routes,
  holds machine registry + tokens-hashed in D1, calls the CF API for
  tunnel/DNS/Access provisioning.
- Access: one application (distinct AUD) per hostname/role; service
  tokens are static credentials — 90-day rotation, per-AUD scoping.
- Tier-0 CF API token: dedicated, per site, tunnel/DNS permissions
  split, audit-log webhook on mutations. Blast radius: Workers
  compromise = control-plane takeover of that site; data plane not
  keyable from it.

## 6. Payload components (unchanged from v0.1, verified)

| Role | Component | Note |
|---|---|---|
| Workers/DOs/KV/D1/Queues/Workflows | celld ≥2 nodes/site | bucket-lease coordination; long-term state in bucket, LTX-replicated SQLite; bucket must be replicated (RPO 15 min, RTO <1 h) |
| R2 | SeaweedFS (scale, object-lock capable) / Garage (homelab, unmodified upstream, no untrusted-writer buckets) | S3/SigV4 per §2 L1 |
| Sandboxes | microsandbox (KVM hosts) / gVisor (no KVM) / E2B Embed (single-node) | untrusted code ⇒ KVM-grade only |
| CI | Woodpecker + Dagger; Gitea Actions if forge wanted | agents poll outbound |

## 7. Onboarding

1. SPA "add host" → API Worker creates tunnel, mints one-time
   enrollment URL (≤15 min TTL) carrying a bootstrap capability + the
   host's generated public key — NOT the tunnel token.
2. Operator approves the host fingerprint in the SPA.
3. Installer (checksum-pinned, served by the Worker) exchanges
   capability + key for tunnel token, overlay join, agent config.
4. Agent `doctor` reports capabilities → registry DO → SPA card green.

Kill switch: delete tunnel + overlay node + service token — three CF
API calls.

## 8. Failure modes

| Failure | Effect | Mitigation |
|---|---|---|
| CF outage | dashboard dark; public serving stops (tunnels edge-terminated); overlay peers keep working | operator path via Headscale ACL; third-party DERP relay noted (encrypted, not self-hosted) |
| API Worker/CF token compromise | tier-0 control-plane takeover (site) | §5 scoping + audit alerts |
| celld bucket down | fleet-wide lease stall | replicated bucket, same-overlay placement (fencing via conditional-write leases) |
| Agent compromise | that host only | per-host scoped tokens, AUD scoping, kill switch |
| Sandbox escape | worst case | KVM-grade isolation mandatory for untrusted code |

## 9. Rollout

0. coexistence: celld runs one non-critical Worker; no migrations
1. CI: Woodpecker behind agent tunnel
2. storage: new buckets self-hosted; R2 remains for legacy objects
3. Workers/DOs: app deploys move to celld; CF Workers stay SPA-only
4. sandboxes: microsandbox cells
Each phase behind a SPA flag; strict after 100% soak.

## 10. Known limits

- Native CF edge transport in Rust: not production-ready (cfrs);
  bundled cloudflared until then.
- celld: v0.4.x, PRs-by-email + CLA; merge into workerd announced
  2026-10-09, timeline unpublished.
- Proxied body cap: 100/200/500 MB by plan (community-verified;
  official page 404s 2026-10-29 — re-check); bulk data on overlay.
- E2B multi-node gating mechanism unverified; Embed treated as
  single-node only.
