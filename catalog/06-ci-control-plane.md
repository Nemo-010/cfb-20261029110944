# Catalog — heavy CI + control-plane precedents

All GitHub numbers verified via the GitHub REST API on 2026-10-29
unless marked ~.

## CI engines

| Tool | Repo | License | Stars | Last push | Verdict |
|---|---|---|---|---|---|
| Woodpecker CI | woodpecker-ci/woodpecker | Apache-2.0 | 8,067 | 2026-10-29 | **adopt** |
| Gitea (+ Actions) | go-gitea/gitea | MIT | 58,760 | 2026-10-29 | **trial → adopt if forge wanted** |
| Dagger | dagger/dagger | Apache-2.0 | 16,543 | 2026-10-29 | **adopt** as engine layer, not scheduler |
| OneDev | theonedev/onedev | MIT | 15,327 | 2026-10-29 | trial (own pipeline dialect) |
| Concourse | concourse/concourse | Apache-2.0 | 7,932 | 2026-10-28 | trial (declarative/immutable niche) |
| Laminar | ohwgiles/laminar | GPL-3.0 | 358 | 2026-10-28 | trial (embedded bootstrap only) |
| Harness (ex-Drone) | harness/harness | Apache-2.0 | 38,756 | 2026-10-29 | **avoid** (Drone discontinued as standalone; parity "expected to take some time") |

## Notes

- **Woodpecker**: Drone-lineage, container-executing agents that poll
  outbound (tunnel-friendly: no inbound ports), SQLite default,
  ~100 MB (server) / ~30 MB (agent) idle per its README; main CI at
  Codeberg. YAML dialect differs from GitHub Actions.
- **Gitea Actions**: stable since 1.19, "similar and mostly compatible
  to GitHub Actions"; runner is a hard fork of nektos/act, MIT, primary
  home gitea.com/gitea/runner (GitHub 404s act_runner). Best pick when
  you also want the forge; "mostly" compatible is the honest caveat —
  some marketplace actions break.
- **Dagger**: programmable container pipeline engine (Go/TS/Python
  SDKs), runs on your metal; complements Woodpecker/Gitea as the
  build/cache engine. Dagger Cloud is optional.
- Long-running containers beyond pipeline steps: celld container cells
  (experimental) or plain systemd under the host agent.

## Control-plane precedents (SPA-on-Cloudflare managing remote metal)

- **Headscale** is the strongest precedent that a small control plane +
  standard agents works at scale (45,164★): the SPA pattern here is the
  same idea with the control plane's UI on Cloudflare.
- **gofixpoint/amika** (295★, Apache-2.0, pushed 2026-10-28) is the
  closest product to "provision machines, load agents, remote-control
  from anywhere" — see catalog/07 for the full piece map. Closed control
  plane is the catch.
- **Pangolin** proves the demand for "tunnel manager with a dashboard";
  its dashboard is self-hosted, ours is CF-hosted by requirement.
- Cloudflare's own repos as precedent: cloudflared + Access service
  tokens is exactly the machine-auth pattern this spec formalizes.

## Sources

GitHub REST API (2026-10-29); Woodpecker README; Gitea docs
(https://docs.gitea.com/usage/actions/overview); Harness README
("Where is Drone?").
