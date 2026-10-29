# Deployment

## Local Linux / WSL
Install Rust stable, build-essential, CMake, pkg-config and libssl-dev. Install Node 22.12+ and pnpm 10.28.2 for frontend builds. Set ADMIN_USERNAME and ADMIN_PASSWORD (at least 12 bytes) for initial bootstrap, build `pnpm --dir apps/admin-web build`, then `cargo run --locked -p proxy-server`. Default `config.toml` uses proxy 0.0.0.0:8080 and private admin 127.0.0.1:9000; SQLite directory is created as needed. `PROXY_CONFIG` selects another TOML. `RUST_LOG` controls structured logging.

Built UI files are served by the binary from admin_static_dir; public assets contain no credentials. Browser API requests are same-origin. Vite development port 5173 forwards /api, /health and /metrics to the private listener. Windows can build the UI; Pingora verification should run on Linux/WSL.

## Published container releases

Use .github/workflows/release.yml by publishing a GitHub Release with a Docker-compatible tag such as v1.0.0. It runs reusable Rust/frontend/deployment checks, builds linux/amd64, runs an isolated full-stack smoke test (PostgreSQL, proxy, Caddy, Redis), and only then pushes the tested image to ghcr.io/OWNER/REPO:TAG. OWNER/REPO is lowercased automatically. The bundle pins the manifest digest returned by docker push, not a mutable tag. There is no moving latest image tag. Both normal and prerelease GitHub Releases build their own versioned images.

The workflow uses contents:write and packages:write only on its publishing job, authenticated by GITHUB_TOKEN. No external registry password or application/database secret is required in GitHub. Existing packages must grant this repository Actions write access; organization package policies can restrict publication. First-published GHCR packages may be private: make the package public for anonymous pulls or authenticate the server with read:packages access. Private Release downloads also require authorized GitHub access. The workflow never changes package visibility or deploys onto a server automatically.

Assets include dynamic-url-proxy-deploy.tar.gz, docker-compose.yml, .env.example, Caddyfile, config.toml, README.md, release.json and SHA256SUMS. The tar archive contains only the runtime files and no source/.env/secrets. Download the archive plus SHA256SUMS to verify downloaded/extracted files with sha256sum -c SHA256SUMS. A successful workflow uploads these files to the existing release. Reruns replace assets with the same filenames; existing installed image digests remain immutable.

On a fresh server (Docker Compose >= 2.20.0, linux/amd64):

```bash
mkdir -p ~/link-proxy
cd ~/link-proxy
curl -fLO https://github.com/Dizzd/link-proxy/releases/latest/download/dynamic-url-proxy-deploy.tar.gz
tar -xzf dynamic-url-proxy-deploy.tar.gz
cp .env.example .env
chmod 600 .env
nano .env
docker compose config --quiet
docker compose pull
docker compose up -d --wait --wait-timeout 180
docker compose ps
```

Replace the example URL for a fork or download a specific release instead of latest. latest excludes prereleases. Fill ADMIN_PASSWORD (at least 12 bytes), POSTGRES_PASSWORD (prefer openssl rand -hex 32) and PUBLIC_HOST before starting. DATABASE_URL interpolates the PostgreSQL values; URL-encode non-hex passwords explicitly. Environment variables already exported in your shell take precedence over .env; unset conflicting deployment variables before starting.

A single Compose file declares every service:

| Service | Selection and persistence | Exposure |
|---|---|---|
| proxy | Always enabled; proxy-data for SQLite, mounted TOML files | 127.0.0.1:8080 and 127.0.0.1:9000 by default |
| postgres | postgres profile; postgres-data volume | Internal network only |
| caddy | caddy profile; caddy-data and caddy-config volumes | TCP 80/443 and optional UDP 443 |
| redis | redis profile; redis-data volume | Internal network only |

The example enables postgres,caddy. Optional dependencies use Compose required:false so disabling PostgreSQL for SQLite/external DB works; enabled PostgreSQL is awaited until healthy. Caddy waits for proxy health. Redis synchronization retries asynchronously and does not gate proxy health.

PUBLIC_HOST=:80 supports initial HTTP/IP access. Set a real DNS hostname and open 80/443 for Caddy automatic HTTPS. Multiple DNS hostnames may be comma-separated. The bundled Caddyfile preserves Host and forwards only routing traffic to proxy:8080. Admin remains private: run ssh -N -L 19000:127.0.0.1:9000 USER@SERVER_IP locally and open http://127.0.0.1:19000. Container localhost refers to the container itself; Caddy uses the Compose service name proxy, and the application uses postgres for the bundled database.

To enable Redis, set COMPOSE_PROFILES=postgres,caddy,redis and CACHE_MODE=memory_redis. SQLite uses COMPOSE_PROFILES=caddy and DATABASE_URL=sqlite:///data/proxy.db. External ingress users remove caddy and target the loopback port; external PostgreSQL users remove postgres and supply their external URL. An empty profile list starts only proxy. Never run two control-plane replicas against the same DB.

## Source-build containers and updates

The root docker-compose.yml declares the same full stack, with build instead of an image reference. Copy the root .env.example, edit it and run docker compose up --build -d. The old PostgreSQL/Redis overlay files have been consolidated and removed; no -f chains are needed. Source Compose mounts config/config.docker.toml and config/caddy/Caddyfile. Release Compose uses flat paths from the downloaded bundle.

The image builds Node/Rust in separate stages and runs as UID 10001, with a public minimal health probe. Pingora has a two-second grace plus ten-second final shutdown timeout; Compose permits 20 seconds. The release workflow smoke-tests the actual image before publishing, but it must run successfully on GitHub before a particular release is considered verified. Docker Engine is unavailable on the local implementation host; Compose parsing can be verified independently without a daemon.

For upgrades, retain the original COMPOSE_PROJECT_NAME, .env, TOML files and Caddyfile. Extract the new release into a separate directory and review differences; update PROXY_IMAGE in the existing .env to the new digest, then run docker compose pull and docker compose up -d --wait --wait-timeout 180. Changing project name creates different named volumes, so existing deployments must keep their original name. Back up DB/certificate volumes first. Do not use down -v. Binary rollback depends on database migration compatibility; configuration rollback uses the API/Revisions UI.

## Configuration and recovery
Keep upstream_cidr_allowlist narrowly scoped; the default denies proxy connections. Enable Redis only through mode=memory_redis and redis.enabled=true, with credentials in the named environment variable. Cache failure never blocks active traffic. Restart recompiles the last committed database revision. Restore revisions through the authenticated API; never rewrite migration history or delete revisions to roll back.

Optional Caddyfile terminates public TLS for the proxy only. Replace hostname/DNS and keep admin private. Forwarded metadata is rebuilt from the direct connection; the application does not yet have a trusted-ingress TLS scheme list. Back up SQLite consistently and monitor disk, revision lag and cache errors.

## PostgreSQL

Set DATABASE_URL=postgres://USER:URL_ENCODED_PASSWORD@HOST:5432/DATABASE, or database_url in TOML. The application applies embedded PostgreSQL migrations at startup. Use a dedicated database/user and TLS verification for remote servers. Default SQLite remains available with no PostgreSQL dependency. Changing the URL does not migrate SQLite data, users or revision history. Preserve the original SQLite file; export/import reviewed mappings if moving deployments.

For Compose, enable the postgres profile and fill POSTGRES_USER, POSTGRES_DB, POSTGRES_PASSWORD and DATABASE_URL in .env (URL host postgres). The PostgreSQL service has no published host port and persists postgres-data. Redis is selected through the redis profile. Use localhost for a PostgreSQL server running directly on the host, postgres for the Compose service. Apply independent PostgreSQL backup/restore procedures.

## Login and password recovery

Open the private admin UI and sign in with the bootstrap username/password. Existing users persist and environment changes do not replace passwords. Change password in Settings; this signs out every session. For recovery, stop the control plane, set ADMIN_USERNAME to an existing user and ADMIN_PASSWORD to the replacement, and execute proxy-server --reset-admin-password. Restart normally. After bootstrap the password environment variable can be removed (Compose currently requires it for interpolation). No default password is shipped.

Keep one active control-plane process per database because import job ownership is local. Session cookies are HttpOnly and SameSite=Strict; set auth.secure_cookies=true behind HTTPS. API tokens are optional and disabled by default. Public /health/ready supports container health checks without credentials; /metrics and management APIs remain protected.

## Single-file cache mode and older images

Current source and new image builds use only config.toml. Set CACHE_MODE=memory (default) or CACHE_MODE=memory_redis in .env. The latter automatically enables Redis in settings; prefix, URL environment name and timeouts are read from the same file. Add redis to COMPOSE_PROFILES when using the bundled Redis service. With no environment override, the TOML mode/enabled flags are preserved. Empty/unknown cache modes fail startup.

For an older installation, preserve the effective TOML contents as config.toml, replace the old file-selection setting with CACHE_MODE and use the new Compose/image together. Keep allowlists/authentication settings, database URL, project name and volumes unchanged. The published v0.1.0 image and artifacts are not rewritten: that binary ignores CACHE_MODE, so follow its bundled instructions or configure cache.mode/cache.redis.enabled directly until upgrading.
