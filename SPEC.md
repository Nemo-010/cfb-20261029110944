# SPEC — "cfb": a Cloudflare-hosted control plane for self-hosted payloads

Version: 0.1.0 (2026-10-29). All external facts cited inline; GitHub
numbers verified against the GitHub REST API on 2026-10-29.

## 1. Goal

Replace Cloudflare's expensive services — R2, Workers, Durable Objects,
Sandboxes/Containers/Computer, and heavy CI — with software you control,
while keeping exactly one thing on Cloudflare: **the dashboard and its
API** (a SPA served by a Cloudflare Worker, gated by Cloudflare
Access). You provision, manage and maintain payload machines remotely
from that SPA. Payload machines have no inbound ports; they dial out via
`cloudflared` and peer over an encrypted overlay.

Non-goals (v1): replacing Cloudflare's CDN/cache/Access itself;
GPU/browser-farm workloads (explicitly out of scope even for celld);
multi-tenant "Workers for Platforms". Ports statement, precise: **no
public inbound ports** on payload hosts; overlay-internal reachability
(operator SSH via Headscale) exists by design and is covered in §7.

## 2. Topology

```
                      ┌─────────────────────────────┐
   human ──browser──▶ │  Cloudflare edge            │
                      │  ┌───────────────────────┐  │
                      │  │ Access (SSO, 2FA)     │  │
                      │  └──────────┬────────────┘  │
                      │  ┌──────────▼────────────┐  │
                      │  │ SPA (static assets)   │  │  ← Workers static assets
                      │  └──────────┬────────────┘  │
                      │  ┌──────────▼────────────┐  │
                      │  │ API Worker            │  │  ← the only "brain" on CF
                      │  └───┬───────────────┬───┘  │
                      └──────┼───────────────┼──────┘
                             │ fetch/SSE     │ CF API (tunnel tokens, DNS, Access)
                             ▼               ▼
                    cloudflared tunnels   Cloudflare dashboard API
                             │
              ┌──────────────┼───────────────┐
              ▼              ▼               ▼
        payload host A  payload host B   payload host C
        celld + overlay + storage + sandbox/CI daemons (no inbound ports)
```

Principle: **Cloudflare holds identity, routing and state-for-the-UI;
your machines hold data and compute.** The API Worker is stateless or
stores only UI-state (saved views, tokens-alias table) in Workers KV/D1,
so a Cloudflare outage degrades you to "cannot see the dashboard" but
payloads keep running; a payload outage shows up as an offline machine
card in the SPA.

**Daemon count per site (the honest ops cost):** cloudflared,
Headscale, the host agent, ≥2 celld nodes, the object store, the
Woodpecker server + agents, and the sandbox daemon — 8–10 long-running
services before any application logic. That is the bill you accept in
exchange for Cloudflare's invoice; if a site cannot run that, the site
is not a cfb site.

## 3. Components

### 3.1 Control plane on Cloudflare (the part you keep)

- **SPA**: static assets served by a Worker (`assets` binding). Any
  framework; the key constraint is that all backend calls go through
  same-origin `/api/*` to the API Worker.
- **API Worker**: one Worker that (a) proxies authenticated calls to
  payload-host endpoints, (b) calls Cloudflare's own API to create/destroy
  tunnels, DNS routes and Access applications per machine, and (c) holds
  per-machine metadata in D1/KV.
- **Cloudflare Access** with service tokens: every machine-to-control-plane
  and control-plane-to-machine request carries `CF-Access-Client-Id` /
  `CF-Access-Client-Secret`; the Access JWT (`Cf-Access-Jwt-Assertion`)
  is validated inside the API Worker. Human sessions use your IdP via
  Access. **Access service tokens are static long-lived credentials**
  that mint short-lived assertions: the security property is the
  rotation policy, not the assertion TTL. Normative: one Access
  application (distinct AUD) per hostname/role; 90-day service-token
  rotation enforced as a SPA-managed checklist; a stolen host token
  reaches only the hostnames whose AUD it holds.
- **Tier-0 secret, designed not shrugged:** the API Worker's Cloudflare
  API token can create tunnels and edit DNS account-wide. Normative:
  (1) one dedicated CF API token per site (separate accounts per site
  when possible), (2) permission split — tunnel-edit and DNS-edit are
  separate tokens held as separate Worker secrets, (3) audit-log webhook
  on tunnel/token/DNS mutations, (4) stated blast radius: a Workers
  compromise equals control-plane takeover of the sites whose tokens it
  holds; the data plane is not keyable from it. This row also appears
  in §7's failure table.

This mirrors the auth pattern `gofixpoint/amika` implements in its open
`@amika/service-auth` package (signed service tokens + per-rig gateway
authorization) — note: verified only as a **pattern donor**; the
package is version 0.0.1 in-repo (no independent npm publishing or
license field checked), and amika's surrounding control plane is
closed source. Do not ship the package itself; ship the pattern. See
`catalog/07-amika.md`.

### 3.2 Transport: cloudflared tunnels (adopted, not replaced)

- One named tunnel per payload host (or per site with multiple
  hostnames), created **by the API Worker** via the Cloudflare API
  (`POST /accounts/{id}/cfd/tunnel`), token handed to the host through a
  one-time enrollment link — the machine-onboarding flow in §5.
- Ingress rules map hostnames like `host-a-api.example.com` →
  `localhost:<port>` for the host's control daemon, `s3.example.com` →
  the object store, `celld.example.com` → the celld public listener.
- Proxied request body limit (verified from Cloudflare's published
  limits): 100 MB on Free/Pro plans, 200 MB Business, 500 MB Enterprise;
  not raisable below Enterprise
  (community.cloudflare.com/t/maximum-upload-size-is-limit/418490 — the
  current official `fundamentals/reference/upload-size` page 404s;
  re-check at build time). Consequence, normative: **bulk data paths
  (object store writes, CI artifacts, images) use the overlay, not the
  tunnel**; the tunnel carries only control traffic and small payloads.
  The overlay path terminates the object store's S3 listener on the
  Headscale network; SigV4 credentials for it are scoped per-site,
  minted by the host agent, and never transit Cloudflare.

Why keep the tunnel when Pangolin/frp exist: the requirement is that
the manager itself lives on Cloudflare and stays manageable from there.
cloudflared is Apache-2.0 (16,336★, pushed 2026-10-29), the edge side is
managed by the same Cloudflare account the dashboard already uses, and
Access policies bind to the tunnel hostnames natively. Pangolin
(23,263★, AGPL-3.0 + commercial dual-license) is the escape hatch if
Cloudflare-as-transport becomes the risk; see `catalog/05-orchestration.md`.

### 3.3 Payload runtime: celld → workerd (the Workers/DO answer)

`denoland/celld` (Apache-2.0, 6,169★, pushed 2026-10-21) runs Workers,
**distributed Durable Objects** (SQLite storage, alarms, hibernating
WebSockets), KV, Queues, D1, R2 bindings, Workflows, cron triggers and
static assets from an existing `wrangler.json`. Nodes coordinate purely
through a shared S3-compatible bucket: conditional bucket writes are the
leases, each cell is a single-threaded actor with its own SQLite DB.
No control plane, no consensus service. Verified from the README.

Consequences for this spec:

- The CI orchestration problem, the "machine" state, even the SPA's own
  backend can be expressed as Workers that run **on your metal** via
  celld, while the thin SPA-facing Worker on Cloudflare just fronts them.
- celld requires its peer listener on a trusted private network —
  exactly the overlay in §3.4 — and its R2 bindings can point at a
  self-hosted store via `--endpoint`.
- Latency and durability facts to design around (verified from the
  README): a single-node celld fleet waits on a bucket round-trip per
  write (~90 ms region-local); ≥2 nodes write peer-to-peer over
  replicated-log HTTP first. **Long-term state — deployments, cell
  state, ownership records — lives in the bucket**, and each committed
  SQLite write is captured in LTX replication format, so host loss
  costs unreplicated recent writes, not the cell. Consequences,
  normative: (a) run **two or more** celld nodes per site from day one;
  (b) the bucket is a fleet-wide dependency — lease acquisition stalls
  fleet-wide if it is down, so it must be replicated (two-site Garage
  or replicated SeaweedFS) with a stated RPO (15 min) and RTO (manual
  failover < 1 h); (c) the conditional-write lease is the fencing
  mechanism — a node separated from the bucket cannot renew leases, so
  cells fail over to nodes that can; a bucket *partition* (reachable by
  some nodes, not others) is the dangerous case and is mitigated by
  keeping the bucket on the same overlay as the nodes, not split across
  networks.
- On 2026-10-09 Cloudflare announced the Deno team joined to merge celld
  into workerd and make self-hosting first-class (primary source:
  blog.cloudflare.com/deno-joins-cloudflare). Kenton Varda confirms
  workerd's own DO support was single-instance-only, i.e. raw workerd is
  **not** the DO answer today; celld is. Track the merge; plan to
  migrate to the merged runtime.

Fallback candidates (catalog/03, catalog/04): `rivet-dev/rivet`
(Apache-2.0, 6,487★) as a DO-shaped actor substrate;
`restatedev/restate` (BSL-1.1) for durable execution; Temporal (MIT,
24,057★) for the workflow half only.

### 3.4 Overlay (the "VPC" answer)

Headscale (BSD-3-Clause, 45,164★) with stock Tailscale clients as the
default: single control server on one of your payload hosts, ACLs as
code, no inbound ports, and it satisfies celld's "encrypted overlay"
requirement. NetBird (30,316★) is the alternative when you want a
managed-style UI, at the cost of an AGPLv3 control plane
(management/signal/relay directories — verified from its LICENSE file).
Avoid making the overlay load-bearing for UI traffic; the SPA reaches
origins through the tunnel, not the overlay, so a dead overlay does not
kill the dashboard.

### 3.5 Storage (the R2 answer)

Self-hosted S3-compatible store behind the tunnel, sized per site:

- **SeaweedFS** (Apache-2.0, 35,799★): scale tier, S3 + POSIX + tiering.
- **Garage** (AGPL-3.0, 4,784★): homelab tier; run **unmodified
  upstream only** (AGPL triggers on network-offered modified versions).
- Consumer audit before picking the store tier (do not argue from
  "R2 parity"): this stack's S3 consumers are celld's fleet bucket
  (needs only get/put/list + conditional writes), CI artifacts
  (Woodpecker/Dagger — versioning not required), sandbox scratch
  volumes, and backups. Backups of anything the CI/sandbox tier writes
  SHOULD be immutable (object lock / versioning) because untrusted code
  writes there; only SeaweedFS and Ceph RGW among the adopted options
  provide that — so the object-lock tier (if enabled) runs SeaweedFS or
  Ceph, and Garage is restricted to buckets without untrusted writers.
  (For reference: R2's own S3 API also lacks versioning, object lock,
  tagging and ACLs — verified against
  developers.cloudflare.com/r2/api/s3/api/ — so migrating off R2 loses
  nothing; but "no worse than R2" is a floor, not a feature goal.)
- Routing rule, reconciled with §3.3: **R2-native bindings are used
  only for Cloudflare-side UI state (the SPA's KV/D1/R2); every
  payload-side store — including celld's fleet bucket — is consumed via
  explicit S3/SigV4 endpoints.** celld's own R2-style bindings point at
  the fleet bucket via its `--endpoint` flag; that endpoint lives on
  the overlay (see §3.2 routing rule).

### 3.6 Sandboxes / Containers / Computer (the compute answer)

Per-session isolation on payload hosts, ordered by what to run where:

- Hosts with `/dev/kvm`: **microsandbox** (Apache-2.0, 8,858★,
  `msb` HTTP API daemon, OCI images, MCP) as the default sandbox
  runtime; **Firecracker** (37,694★) under E2B's runtime (Apache-2.0,
  1,781★; single-node "Embed" is OSS, multi-node is their paid tier) if
  you want the richest SDK surface including a Desktop/computer-use SDK.
- Hosts without KVM (VMs, ARM boards): **gVisor** (`runsc`) with
  Docker/containerd.
- celld's experimental Containers/Sandbox-SDK cells are the
  integration point: a DO supervises a sandbox, and the SPA talks to
  the DO — that is the "provision and drive a machine from the
  dashboard" loop, in Workers-native terms.
- cloudflare/sandbox-sdk (1,156★) is the API-shape parity target: its
  bridge is Workers-only by design, so adopt its surface, not its
  runtime.

### 3.7 Heavy CI (the motivating workload)

- **Woodpecker CI** (Apache-2.0, 8,067★): server + agents, agents poll
  outbound (no inbound ports; tunnel-friendly), SQLite default,
  ~100 MB/~30 MB RAM idle. The SPA's "CI" tab is a client of the
  Woodpecker API through the tunnel.
- **Gitea + Gitea Actions** (MIT, 58,760★) when you also want the forge
  and near-GitHub-Actions syntax; runner is a hard fork of `act`.
- **Dagger** (Apache-2.0, 16,543★) as the engine layer inside those
  pipelines (container-native, cached, programmable), not as the
  scheduler.
- Long-running containers: Woodpecker for pipeline units + celld's
  experimental container cells (or plain systemd units managed by a
  small host agent) for services that outlive pipelines.

## 4. CF-service → component mapping (normative table)

| CF service | Spec component | Runtime owner | Notes |
|---|---|---|---|
| Workers (SPA + API) | SPA + API Worker | Cloudflare (kept) | Thin by design |
| Workers (app logic) | celld nodes | your metal | wrangler.json unchanged |
| Durable Objects | celld cells | your metal | distributed, SQLite, alarms, WebSockets |
| KV / D1 / Queues / Workflows | celld | your metal | |
| R2 | SeaweedFS / Garage | your metal | S3 SigV4 over tunnel; R2 bindings stay CF-side |
| Tunnel | cloudflared (API-provisioned) | your metal dials out | one tunnel per host |
| VPC | Headscale overlay | your metal | peer + bulk-data path |
| Sandboxes/Computer | microsandbox / E2B Embed / gVisor | your metal | API shape mirrors sandbox-sdk |
| CI | Woodpecker (+ Gitea, Dagger) | your metal | outbound agents |

## 5. Machine onboarding flow (the "provision a machine remotely" loop)

1. Operator clicks "Add machine" in the SPA → API Worker creates a named
   tunnel via the CF API, minted as a one-time enrollment URL
   (short TTL, ≤15 min) that carries **only a bootstrap capability** —
   not the tunnel token itself.
2. The SPA shows an explicit **"approve new host"** step: the enrollment
   URL also carries the host's generated public key; the operator
   approves the fingerprint in the SPA before anything is issued.
3. On the new host, the installer (fetched over the enrollment URL,
   checksum-pinned to a release published by the same Worker) exchanges
   the bootstrap capability + key for: `cloudflared` (tunnel token),
   the overlay join (Headscale pre-auth key), and the host agent — only
   after operator approval. Bootstrapping paradox acknowledged: the
   machine's first credential arrives over the same transport being
   provisioned; the capability is short-TTL, single-use, and
   key-pinned so interception without the private key is useless.
4. Host agent registers capabilities (CPU/RAM/disk/KVM yes-no, roles:
   `ci`, `sandbox`, `storage`, `celld`) into a Durable Object **on the
   site's celld fleet** (selection rule: least-loaded node with the
   `celld` role in the same site, as reported by the agent); the API
   Worker reads it through the tunnel and the SPA card goes green.
5. All later management (redeploy, drain, rotate tokens, wipe) is API
   calls from the Worker through the tunnel; nothing dials in from the
   internet.

Kill-switch property: revoking a machine = delete its tunnel +
overlay node + Access service token. Three CF-API calls, immediate,
no host access needed.

## 6. Security model

- Identity: humans via Access (IdP); machines via Access service tokens
  + per-tunnel tokens; overlay peers via Headscale ACLs.
- Trust boundaries: the API Worker never holds host SSH keys; host
  credentials are scoped tokens stored hashed in D1. Secrets for
  payloads live on the payloads (env/secrets of celld services), not in
  the SPA.
- Default-deny: tunnel ingress rules enumerate only the daemons that
  must be reachable; the overlay carries celld peer traffic and bulk
  data; the SPA cannot bypass either.
- Worker→origin and origin→Worker calls validate the
  `Cf-Access-Jwt-Assertion` **audience (AUD) per hostname/role**; a
  stolen host service token is replayable until rotated (static
  credential), which is why §3.1 fixes a 90-day rotation policy and
  per-AUD scoping, and why the tunnel-token kill-switch is one API
  call.

## 7. Failure modes (design for these)

| Failure | Effect | Mitigation |
|---|---|---|
| Cloudflare outage | dashboard dark; **tunnels down, so public serving stops** — only overlay peers still work; payloads run but are not reachable from the internet | payloads keep serving overlay peers; degraded control via Headscale ACL'd path (operator needs a Tailscale client and the Headscale control server reachable — it is co-located on payload metal, so it survives CF loss; note stock clients fall back to third-party DERP relays, which are encrypted but not self-hosted transit) |
| API Worker / CF API token compromise | **tier-0: full control-plane takeover of that site's fleet** (tunnels, DNS, Access apps) | per-site tokens, tunnel/DNS permission split, audit-log webhook alerting; data plane not keyable from it |
| celld bucket unavailable | **fleet-wide** lease-acquisition stall (not just single-node); ≥2 nodes do not help | bucket on replicated store (two-site Garage / replicated SeaweedFS), RPO 15 min / RTO <1 h manual failover; same-overlay placement to avoid partition split-brain; conditional-write lease = fencing (partitioned nodes cannot renew, cells fail over) |
| Headscale control server down | no new overlay joins; existing peers keep WireGuard sessions | control server on payload metal, restarted by the host agent; DERP fallback keeps connectivity |
| Tunnel token leak | attacker reaches that host's mapped ports | short-lived tokens, per-host scope, Access policy on every hostname |
| Sandbox escape | worst case in this design | KVM-grade isolation (microsandbox/Firecracker) not containers, on untrusted code paths |
| CF changes Access/Tunnel pricing | the strategic risk | Pangolin (AGPL-3.0 + separate Fossorial Commercial License) + Authelia plan in catalog/05; data plane and control plane are portable, only the UI host is CF |

## 8. Rollout (warn-then-strict, feature-preserving)

1. **Phase 0 — coexistence**: run cfb alongside CF-managed services.
   celld runs your least-critical Worker; SPA reads both worlds. No
   migrations.
2. **Phase 1 — CI first**: Woodpecker behind a tunnel; heavy jobs leave
   paid CI. Lowest blast radius, immediate cost win, validates tunnels
   + Access service tokens.
3. **Phase 2 — storage**: new buckets on self-hosted store; R2 keeps
   serving old objects (lifecycle-copy out, not a cutover).
4. **Phase 3 — Workers/DOs**: move app Workers onto celld nodes behind
   the SPA; CF Workers remain the UI front only.
5. **Phase 4 — sandboxes**: microsandbox cells for agent workloads.
6. Each phase ships behind a per-feature SPA flag; "strict" (CF service
   deprecated) only after the self-hosted path has served 100% of the
   feature's traffic for a defined soak period.

## 9. Open questions (honest list)

- E2B's multi-node gating mechanism (code vs paid product vs ToS
  clause) is unverified; treat Embed as single-node-only until checked
  before Phase 4.
- celld production hardening: v0.4.x-era project, PRs-by-email + CLA
  (Deno Land Inc.); watch the workerd merge timeline.
- Whether amika's promised `amika-gateway` open-sourcing (single-player
  self-host) changes the sandbox story; re-check before Phase 4.
- Cloudflare Workers VPC (private networks from Workers) could replace
  part of §3.4 for Workers→host traffic if/when it fits the threat
  model; not assumed here.
- The 100/200/500 MB proxied body-size limits were verified from a
  Cloudflare community answer because the official
  `fundamentals/reference/upload-size` page 404s (2026-10-29); the
  number's shape is consistent with long-standing limits, but re-verify
  against an official page at build time. Throughput (not the cap)
  still needs measurement on real tunnels.
