# Catalog — gofixpoint/amika: piece-reuse map

Repo: https://github.com/gofixpoint/amika — Apache-2.0, **295 stars**,
pushed 2026-10-28, not archived (GitHub API, 2026-10-29). "Multiplayer
cloud workstations for coding agents and humans": provisions VMs
("Rigs") on any cloud or your own computer, injects repos + agent
credentials, remote-controls coding agents from chat/web/CLI/SSH.

## What is open vs closed (verified from README + docs/self-hosting.md)

| Component | Status | Reusable for cfb? |
|---|---|---|
| `amikad` (in-VM daemon: rig setup, networking, gateway/relay comms) | **open** | yes — study as the host-agent design |
| `amika-hostd` (runs Rigs on your own machine as **smolvm** microVMs) | **open** | yes — reference for per-host sandbox provisioning |
| `sandbox-image`, TS SDKs | **open** | yes — SDK ergonomics reference |
| `@amika/service-auth` (signed service tokens, browser login passes, per-rig gateway auth) | **open** | **yes — directly parallels our Access-service-token model; worth porting** |
| `amika-control-plane` (schedules Rigs, rig registry) | **closed** | no — build this as API Worker + celld DOs |
| `amika-ui`, `amika-relay` | **closed** | no — the SPA replaces the UI; relay only if you accept a binary blob |
| `amika-gateway` + `sandbox-provisioner` (would enable full self-host, single-player, no multi-user IAM) | "open source soon" | watch — re-evaluate when shipped |

## Hard facts (verified)

- The self-hosting doc states there is **no supported local CLI path
  today**: every `amika` command talks to Amika's hosted control plane;
  the `--local` Docker mode was removed.
- Auth: device authorization flow + org-scoped API keys;
  `AMIKA_API_URL` is overridable (docs/auth.md).
- Runtime on your own machine = smolvm (https://github.com/smol-machines/smolvm,
  Apache-2.0, 6,868★) — a project worth adopting on its own merits for
  the sandbox lane.

## Verdict for cfb

**Trial, as a donor not a foundation.** Amika's architecture (BYO
compute + agent mesh + service-token auth) validates this spec's shape,
and `@amika/service-auth` + `amikad`/`amika-hostd` are worth porting or
borrowing. But the brain (control plane, UI, relay) is closed, the
fully-self-host gateway is unshipped, and the company (Fixpoint, Inc.,
YC-backed) could follow Daytona's closed-source path. Nothing load-bearing
in cfb depends on amika. Do not file it under "adopt" despite the fit.

## Related context

- E2B runtime Embed (single-node OSS, multi-node paid) — catalog/02.
- Daytona closed-source June 2026 — the lane's cautionary tale.
- microsandbox — the fully-open alternative for the same lane.

## Sources

- https://github.com/gofixpoint/amika (README: open/closed split)
- https://github.com/gofixpoint/amika/blob/main/docs/self-hosting.md
- https://github.com/gofixpoint/amika/blob/main/docs/auth.md
- https://raw.githubusercontent.com/gofixpoint/amika/main/js/service-auth/AGENTS.md
- GitHub REST API, 2026-10-29
