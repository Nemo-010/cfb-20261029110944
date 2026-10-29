# Catalog — repos researched for the agent design

All numbers verified via GitHub REST API 2026-10-29.

## talaria0101/cfrs — 0★, 0BSD, Rust, pushed 2026-10-09 (20 days old)

Single Rust binary, cloudflared-replacement prototype. Subcommands:
`provision` (anonymous quick-tunnel via `POST /tunnel`, credential-free),
`tunnel` (CF edge HTTP/2 + QUIC transports — **no serving loop,
untested against the real edge**, per its own README), `serve` (working
SSH remote-forward relay on 443 splicing to a unix-socket origin),
`doctor` (host capability probe), `metrics`, `net` (smoltcp userspace
stack + AF_INET→AF_UNIX LD_PRELOAD shim). cloudflared-compatible
ingress config.

**Informs cfb:** agent module split (tunnel/provision/doctor/metrics in
one binary); unix-socket origin as default local interface; SSH-443
fallback relay; capability-probe-as-enrollment; ingress config compat.
**Not:** its edge transport (unproven); its SSH relay as production
path (third-party relay, no host-key pinning). 0★/1-author: design
reference, not a dependency.

## ZingerLittleBee/ServerBee — 339★, AGPL-3.0, Rust, pushed 2026-10-17

Agent/server monitoring: ~4.3 MB agent (sysinfo + tokio-tungstenite)
dials server over WebSocket, 3 s metric push; Axum 0.8 + SQLite WAL
server with React dashboard embedded in one binary. Enrollment:
single-use ≤10-min offer, agent-generated persistent run token before
claiming, auto-reconnect. Capability gates with host-local temporary
grants; web terminal, file manager, Docker stats, auto-update, 180+
OpenAPI endpoints. 3,800+ tests, >91% coverage claimed.

**Informs cfb:** enrollment handshake; run-token model; capability
gates; metric push cadence; TOML + `PREFIX_` env config convention;
single-binary server shape. **Not:** code (AGPL vs 0BSD — ideas only);
its auth/multi-tenant surface (Access covers identity); no tunnel —
plain HTTP/WS, not a cloudflared substitute.

## metalbear-co/mirrord — 5,390★, MIT, Rust, pushed 2026-10-29

Runs local processes as if in a k8s cluster. Components: LD_PRELOAD
`layer`, in-cluster `agent` (CAP_NET_ADMIN/iptables, CAP_SYS_PTRACE),
`intproxy`, optional `operator` (persistent-WebSocket broker replacing
port-forwards), versioned multiplexed `protocol` crates,
`sessions-manager` + `session-monitor-protocol`.

**Informs cfb:** broker design — one long-lived outbound connection
multiplexing sessions with version negotiation (`protocol-api` shape);
session/monitor protocol split for metrics. **Not:** the multi-daemon
topology (justified only by k8s trust domains); traffic interception
caps; k8s coupling. No Workers-portability content.

## ZingerLittleBee/ServerBee · cross-note

With cfrs, ServerBee provides both halves of the agent spec: cfrs the
tunnel/doctor, ServerBee the enroll/monitor/gates.

## Pouzor/homelable — 4,780★, MIT, TypeScript, pushed 2026-10-28

Homelab visualizer: FastAPI backend + Vite frontend + optional MCP
container; agentless polling (nmap, ping/http/tcp/ssh/prometheus
checks, MQTT, Proxmox/Unraid APIs); SQLite; JWT/OIDC + separate
static-key read-only gates; "pending devices" approval queue;
inventory reconciliation by MAC/IP.

**Informs cfb:** pluggable per-node check-method registry; versioned
REST + generated OpenAPI as the broker's HTTP contract; two-key split
(client key vs service key); approval-queue onboarding UX. **Not:**
agentless model (no host-internal state), Python/SQLite/multi-container
stack.

## crocofied/CoreObs — 1,295★, MIT, TypeScript, pushed 2026-02-13 (stale ~8.5 months)

Ex-CoreControl. 3-service compose (Next.js web, Bun+Elysia API,
Postgres 17). Go agent's only config is `DATABASE_URL` — it writes
metrics straight into shared Postgres. **Go agent and web app sources
are not in the repo** (prebuilt images only). Glances used for host
metrics.

**Informs cfb:** Go/Rust static single-binary agent as the right
implementation shape; Glances data model. **Negative reference:**
agent→shared-DB direct writes (no broker, no buffering, migration
coupling) — SPEC §4 forbids this pattern; closed agent source; repo
hygiene (renamed, stale image names).

## timhartmann7/omnyssh — 1,247★, Apache-2.0, Rust, pushed 2026-10-09

SSH client/manager (Tauri GUI + TUI on shared core): dashboard cards,
PTY tabs, SFTP, snippets, ⌘K. Client-only — no daemon, no agent, no
tunnel. One-click Ed25519 setup with `authorized_keys` install and
`sshd_config` backup/verify/rollback; `~/.ssh/config`/ProxyJump import.

**Informs cfb:** the Ed25519 bootstrap + rollback flow for agent
enrollment; operator terminal UX expectations; core/frontend split.
**Not:** metrics-over-SSH-exec; desktop shell (console is the CF SPA).
