# Catalog — storage (R2 replacements)

All GitHub numbers verified via the GitHub REST API on 2026-10-29.
Verdicts: adopt / trial / avoid.

## Parity bar (what "R2-compatible" actually requires)

R2's own S3 API implements SigV4, multipart, presigned URLs, SSE-C and
conditional writes, but **does not implement** bucket versioning, object
lock, S3 notifications, tagging or ACLs (verified:
https://developers.cloudflare.com/r2/api/s3/api/). A self-hosted store
therefore does not need full AWS conformance to be "R2-equivalent".

## Tools

| Tool | Repo | License | Stars | Last push | Verdict |
|---|---|---|---|---|---|
| SeaweedFS | seaweedfs/seaweedfs | Apache-2.0 | 35,799 | 2026-10-29 | **adopt** (scale) |
| Garage | deuxfleurs-org/garage | AGPL-3.0 | 4,784 | 2026-10-24 | **adopt** (homelab) |
| VersityGW | versity/versitygw | Apache-2.0 | 3,079 | 2026-10-29 | trial |
| Ceph RGW | ceph/ceph | LGPL-ish (NOASSERTION) | 17,178 | 2026-10-29 | adopt (colo tier only) |
| RustFS | rustfs/rustfs | Apache-2.0 | 35,524 | 2026-10-29 | trial (young, governance unproven) |
| Scality CloudServer | scality/cloudserver | Apache-2.0 | 1,955 | 2026-10-29 | trial |
| s3Proxy | gaul/s3proxy | Apache-2.0 | 2,415 | 2026-10-02 | trial (glue only) |
| JuiceFS | juicedata/juicefs | Apache-2.0 | 14,586 | 2026-10-29 | trial (POSIX bridge) |
| OpenStack Swift | openstack/swift | Apache-2.0 | 2,804 | 2026-10-28 | **avoid** (S3 surface too weak: no presigned) |
| MinIO | minio/minio | AGPL-3.0 | 61,300 | 2026-04-24 | **avoid — repo archived** |

## Notes

- **MinIO is archived** (GitHub `archived: true`; banner Apr 25, 2026).
  Historical default; do not build on it.
- Garage S3 surface: SigV4 ✅, path + vhost style ✅, presigned ✅,
  SSE-C ✅, website ✅; versioning/lock/policies/tagging ❌
  (https://garagehq.deuxfleurs.fr/documentation/reference-manual/s3-compatibility/)
  — i.e., R2-shaped. GitHub repo is a mirror of git.deuxfleurs.fr.
- SeaweedFS: S3 gateway + filer + FUSE + EC + tiering; multi-component
  ops heavier than Garage but lighter than Ceph.
- Ceph RGW is the only option here with versioning + object lock +
  notifications + policies implemented — pick it when you need
  R2-plus-AWS semantics and can afford the ops.
- Integration: expose path-style S3 over cloudflared with Access
  service tokens; big-object paths over the Headscale overlay (tunnel
  upload caps: proxied body limits 100/200/500 MB by plan — bulk data
  on the overlay per SPEC §3.2). Native R2 bindings cannot
  target these stores; use an S3 client (aws4fetch) from the CF side.
- Local dev: Miniflare (now inside cloudflare/workers-sdk, the
  standalone repo is archived) emulates R2 for the SPA's dev loop.

## Sources

- https://developers.cloudflare.com/r2/api/s3/api/
- https://garagehq.deuxfleurs.fr/documentation/reference-manual/s3-compatibility/
- GitHub REST API responses, 2026-10-29
