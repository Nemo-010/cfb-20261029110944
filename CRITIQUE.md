# Rejected alternatives & known gaps

Verified against GitHub API 2026-10-29. Companion to SPEC.md.

## Rejected

| Alternative | Reason |
|---|---|
| Kubernetes/K3s as payload orchestration | control plane the SPA would have to front; celld bucket-lease placement covers the need |
| Agent-writes-to-shared-DB (CoreObs pattern) | couples agents to central schema; no offline buffering; agent is closed source there — spec mandates agent→broker→storage |
| Multi-daemon per host (layer/intproxy/agent/operator mirrord topology) | distinct trust domains justify it for mirrord's k8s threat model; here one binary with capability gates suffices |
| Metrics-over-SSH-exec (omnyssh model) | fork-per-poll; agent self-reports instead. SSH kept for operator terminal only |
| MinIO / Daytona / miniflare-standalone | archived upstream (API-verified) |
| R2-native bindings for payload storage | CF-only; breaks R1 portability |
| Garage for untrusted-writer buckets | no object lock/versioning; untrusted CI output needs immutability — SeaweedFS/Ceph tier |
| Pangolin as default transport | dashboard must be CF-resident (R3); kept as escape hatch if CF transport becomes the risk (AGPL-3.0 + Fossorial Commercial License) |
| NetBird over Headscale | AGPLv3 control dirs vs BSD-3; feature gap not decisive |

## Known gaps

- cfb agent does not exist yet; §4 is a build spec assembled from
  verified precedents (cfrs, ServerBee, mirrord, omnyssh), not code.
- Native Rust CF edge transport: no tested implementation; cfrs'
  `tunnel` module has no serving loop and is untested against the
  edge (its README states this). Bundled cloudflared until one is.
- celld: no external production case studies; contribution model
  (email patches + CLA) limits outside maintainership.
- No performance benchmarks run; no cost model.
- Self-hosted identity (Authelia/Pangolin) is weaker than CF Access;
  only acceptable as emergency path.
- ServerBee: AGPL-3.0 — design reuse only, no code.
- Star counts drift; all numbers are "as of 2026-10-29".
