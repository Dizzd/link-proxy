# v0.3.1 pre-release verification

Date: 2026-10-30.

- Release: https://github.com/Dizzd/link-proxy/releases/tag/v0.3.1 (published pre-release, not Latest).
- Tagged source: 71bf8c53e41cef3b72168bbe38fc854e6ffeebde.
- Hosted workflow: https://github.com/Dizzd/link-proxy/actions/runs/41032028403 (success).
- All four jobs passed: Rust, frontend, deployment validation and container publication. Rust included real HTTP/recovery/shutdown, PostgreSQL and runtime configuration tests. Publication built the linux/amd64 image, smoke-tested PostgreSQL + proxy + Caddy + Redis, pushed GHCR and uploaded eight release assets.
- Image: ghcr.io/dizzd/link-proxy:v0.3.1.
- Immutable image: ghcr.io/dizzd/link-proxy@sha256:20df0b24f96c5d3b9ed67b394daa83ccad3991b35e1d2021b5923317233fc9bb.

Downloaded all eight published assets through authenticated GitHub access. All seven SHA256SUMS entries matched the actual files. The archive contained exactly six regular files, all with mode 0644, each identical to its standalone asset. GitHub names the standalone environment asset default.env.example; the archive retains .env.example. Both release.json and PROXY_IMAGE pin the image digest recorded by the successful hosted workflow. Version and platform metadata matched v0.3.1 and linux/amd64.

Local functional verification is summarized in [bulk-import-id.md](bulk-import-id.md). Raw hosted logs and downloaded assets remain local and are ignored by Git. No production deployment or production benchmark is claimed; hosted container smoke verifies the isolated release stack.
