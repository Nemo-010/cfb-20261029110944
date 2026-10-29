# CRITIQUE — what is wrong with this spec

Written before external review; to be amended after the review swarm.

## 1. The Cloudflare paradox (structural, not fixable — price it honestly)

The spec's premise is "escape Cloudflare's expensive services" while
keeping the dashboard on Cloudflare and all traffic paths through
Cloudflare's edge. Consequences:

- You are not reducing Cloudflare dependence; you are shrinking it to
  its most business-critical part (identity + routing) and spending
  what you save on metal. If Cloudflare raises Tunnel/Access pricing,
  the data plane cannot migrate quickly — only the control plane is
  portable on paper, and the SPA rewrite plus Access replacement is
  real work the spec hand-waves as "Pangolin escape hatch".
- Every tunnel packet transits a proprietary edge the spec does not
  control and cannot self-host. The honest claim is "cheaper", not
  "independent".

## 2. celld is a bet on an unfinished merge

The whole Workers/DO story rests on a v0.4.x project with PRs disabled,
a CLA funneling rights to Deno Land Inc., and an announced merger into
workerd whose timeline is "announcements in the coming months". Risks:

- If the merge stalls, celld's community may fragment (its contribution
  model actively discourages outside maintainership).
- If the merge lands differently, `wrangler.json` compatibility
  guarantees may shift.
- The spec's mitigation — "track the merge" — is thin. Honest
  additional mitigation: keep the app's non-DO logic runtime-agnostic;
  Rivet as the Apache-2.0 fallback is credible but is a rewrite, not a
  migration.

## 3. Latency and topology realities the spec underweights

- celld's bucket-lease writes: ~90 ms single-node. The spec says "run
  ≥2 nodes", which fixes latency but means the **bucket is a silent
  single point of failure for the fleet** unless replicated; the spec
  parks that in a failure-mode row rather than a design.
- Access-token validation inside the API Worker on every hop adds
  subrequest latency and couples SPA responsiveness to Access health.
- Tunnel upload caps are admitted-unverified, yet the storage lane's
  integration story leans on the tunnel; the overlay fallback is
  asserted, not designed (who terminates S3 SigV4 on the overlay? how
  do credentials get there?).

## 4. Security model gaps

- The API Worker holds a Cloudflare API token capable of creating
  tunnels and editing DNS — it is a tier-0 secret living in Workers.
  The spec says credentials are "scoped", but tunnel-creation power is
  inherently account-wide. A Workers compromise = full fleet takeover.
  Mitigations (Cloudflare scoped API tokens, separate account/API
  tokens per site, audit-log alerting) are not designed.
- The onboarding `curl | sh` flow is a supply-chain foot-gun; no
  checksum/signing story exists.
- Sandbox verdicts lean on libkrun/microsandbox ("third parties flag it
  experimental") — the spec acknowledges this but still routes
  untrusted agent code through it in the rollout.

## 5. Ops-burden honesty

- Ceph/SeaweedFS "adopt" rows come with multi-component ops; a one-page
  spec sentence is not a runbook. The realistic homelab answer (Garage,
  two disks, offsite mirror) is underspecified.
- Woodpecker + Gitea + Dagger + celld + Headscale + cloudflared +
  sandbox daemon + storage = ≥7 daemons per site before any app logic.
  The spec never counts the daemons; it should, because that is the
  cost being traded for Cloudflare's bill.

## 6. Catalog integrity notes

- All GitHub numbers were API-verified 2026-10-29; three third-party
  figures (microsandbox ~6.7k, cloudflared ~16.1k, firecracker ~38k)
  were corrected to API values (8,858 / 16,336 / 37,694).
- Two lanes could not run the mandated `gh`/curl tooling and used the
  public REST API / built-in search instead — disclosed in their
  reports; the final numbers were re-verified here.
- Star counts drift daily; treat every number as "as of 2026-10-29".

## 7. What this spec deliberately does not claim

- No performance benchmarks (none were run).
- No cost model (the motivating premise is qualitative).
- No claim that self-hosted Access equals Cloudflare Access on security
  guarantees (it does not; Authelia/Pangolin are weaker identity planes).

— End of first-pass critique.

## 8. Review-swarm amendments (2026-10-29)

Two adversarial reviewers (security/transport, architecture/licensing)
both returned **SOUND WITH FIXES**. All 10 required changes from each
were applied to SPEC.md; the load-bearing ones:

- §3.3/§3.5 contradiction resolved: R2-native bindings = CF-side UI
  state only; all payload-side storage via explicit S3 endpoints.
- Garage "parity" argument replaced by a consumer audit; object-lock
  tier (SeaweedFS/Ceph) required where untrusted code writes; Garage
  restricted to unmodified upstream, non-untrusted-writer buckets.
- Bucket failure row corrected (fleet-wide lease stall), split-brain/
  fencing analysis added; bucket + Headscale SPOFs added to §7.
- Onboarding redesigned: key-pinned, checksum-pinned, operator-approved
  enrollment; `curl | sh` gone.
- Static service-token reality + 90-day rotation + per-hostname AUD
  scoping written into §3.1/§6; "replay window = TTL" claim removed.
- Tier-0 CF API token designed (per-site, permission-split, audited)
  and added to §7 as a first-class failure mode.
- Cloudflare-outage row corrected: public serving dies with tunnels;
  DERP relay dependency named; §1 softened to "no public inbound ports".
- Proxied body-size limit verified (100/200/500 MB by plan) and made
  normative: bulk data goes over the overlay; S3-over-overlay
  credentialing specified in §3.2.
- amika `@amika/service-auth` downgraded to pattern donor (v0.0.1,
  in-repo, license/npm status unverified).
- Daemon-count paragraph added to §3.

Rejected/amended reviewer points: none dropped entirely; the E2B
multi-node gating and upload-cap items were moved to §9 (verifiable,
not yet verified) rather than asserted.
