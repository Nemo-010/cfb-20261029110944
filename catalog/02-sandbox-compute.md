# Catalog — sandboxes, containers, computer-use (Sandboxes/Containers/Computer replacements)

All GitHub numbers verified via the GitHub REST API on 2026-10-29.

## Tools

| Tool | Repo | License | Stars | Last push | Verdict |
|---|---|---|---|---|---|
| Firecracker | firecracker-microvm/firecracker | Apache-2.0 | 37,694 | 2026-10-29 | **adopt** (substrate) |
| microsandbox | superradcompany/microsandbox (was microsandbox/microsandbox) | Apache-2.0 | 8,858 | 2026-10-29 | **trial → adopt when stable** |
| gVisor | google/gvisor | Apache-2.0 | 19,818 | 2026-10-29 | **adopt** (no-KVM hosts) |
| E2B runtime + SDK | e2b-dev/runtime, e2b-dev/E2B | Apache-2.0 | 1,781 / 14,606 | 2026-10-29 | trial (Embed = single node) |
| smolvm | smol-machines/smolvm | Apache-2.0 | 6,868 | 2026-10-29 | trial (runtime primitive) |
| Kata Containers | kata-containers/kata-containers | Apache-2.0 | 9,044 | 2026-10-29 | trial (k8s shops) |
| Cloud Hypervisor | cloud-hypervisor/cloud-hypervisor | Apache-2.0/BSD-3 (API: null) | 6,440 | 2026-10-29 | trial |
| cloudflare/sandbox-sdk | cloudflare/sandbox-sdk | Apache-2.0 (in-tree LICENSE) | 1,156 | 2026-10-29 | parity target, not the runtime |
| Bhatti | sahil-shubham/bhatti | Apache-2.0 | 270 | 2026-10-27 | trial (watch) |
| DifySandbox | langgenius/dify-sandbox | Apache-2.0 | 1,280 | 2026-10-29 | trial (Dify only) |
| Daytona | daytonaio/daytona | **none** | 71,443 | 2026-07-24 | **avoid — archived, closed-source successor** |

## Notes

- **Firecracker** is the production microVM substrate (it underlies E2B,
  Fly, Vercel sandbox) but is a VMM only — bring your own API daemon.
- **microsandbox** is the closest single-binary "self-hosted E2B":
  libkrun microVMs, OCI images, `msb server` HTTP API, MCP server,
  snapshot/restore, documented GitHub-Actions-runner mode
  (https://docs.microsandbox.dev/how-to/ci-cd/github-actions-runner).
  Third parties flag it experimental; repo moved orgs
  (microsandbox/microsandbox redirects to superradcompany) — provenance
  watch. Requires KVM/WHP/Apple Silicon.
- **E2B runtime**: full backend (API, orchestrator, edge router,
  envd, template builder) with a single-node **Embed** deployment
  (docker compose/Terraform); multi-node production is Enterprise — a
  hard boundary for this spec. Desktop SDK covers computer-use.
- **gVisor** for hosts without `/dev/kvm` (VMs, boards): container UX,
  VM-grade syscall interception. Run Docker/containerd with
  `--runtime=runsc`.
- **smolvm**: branchable/portable libkrun microVMs; notably mounts
  S3/R2/MinIO buckets into VMs (`-v s3://bucket:/data`) and is the
  engine amika's `amika-hostd` uses on your own machine.
- **cloudflare/sandbox-sdk**: its bridge is Workers-only by design
  (https://developers.cloudflare.com/sandbox/sdk/bridge/) — treat its
  API surface (per-session sandbox, exec, streams, git, code
  interpreter) as the parity target for the self-hosted sandbox API.
- **Daytona**: went closed-source June 2026, repo frozen at v0.190, no
  security patches (https://www.daytona.io/dotfiles/updates/daytona-is-going-closed-source);
  GitHub shows `archived: true`, no license. Community fork
  nightona-co/nightona is unproven.

## Mapping to the spec

Per-host sandbox daemon behind cloudflared; SPA ↔ celld container/sandbox
cell ↔ sandbox runtime. Untrusted code → KVM-grade (microsandbox /
Firecracker / libkrun), never plain containers. No-KVM host → gVisor.

## Sources

GitHub REST API (2026-10-29); links above; Northflank's self-hostable
E2B-alternatives survey
(https://northflank.com/blog/self-hostable-alternatives-to-e2b-for-ai-agents).
