# Dynamic URL Proxy release deployment

This bundle runs the prebuilt linux/amd64 image with Docker Compose >= 2.20.0.
It needs no Rust, Node, source checkout or image build on the server.

## First start

Extract dynamic-url-proxy-deploy.tar.gz into a permanent deployment directory.
Copy .env.example to .env, chmod 600 .env, and edit .env:

```bash
cp .env.example .env
chmod 600 .env
nano .env
docker compose config --quiet
docker compose pull
docker compose up -d --wait --wait-timeout 180
docker compose ps
```

Set ADMIN_PASSWORD (at least 12 bytes), POSTGRES_PASSWORD (generate with openssl rand -hex 32) and PUBLIC_HOST. The example DATABASE_URL interpolates the PostgreSQL user/password/database. For non-hex passwords, URL-encode the password in DATABASE_URL explicitly. Never commit .env.

One docker-compose.yml defines every service. COMPOSE_PROFILES=postgres,caddy starts PostgreSQL, the proxy and Caddy. Add redis and select PROXY_CONFIG_FILE=config.redis.toml to enable the optional Redis cache. PostgreSQL and Redis expose no host ports. Direct proxy/admin ports bind to host loopback by default.

PUBLIC_HOST=:80 enables HTTP/IP access. For public HTTPS, set PUBLIC_HOST to your DNS hostname, point its A/AAAA records to the server and open TCP 80/443 (UDP 443 is optional for HTTP/3). Caddy obtains/renews certificates and preserves Host for routing. Multiple hostnames can be comma-separated in PUBLIC_HOST; each must have valid DNS. Caddy exposes only the routing listener, not the administration UI. If another ingress already occupies 80/443, remove caddy from COMPOSE_PROFILES and point that ingress at 127.0.0.1:8080.

## Administrator access and mappings

On your workstation:

```bash
ssh -N -L 19000:127.0.0.1:9000 USER@SERVER_IP
```

Open http://127.0.0.1:19000 and sign in with ADMIN_USERNAME/ADMIN_PASSWORD. Startup initializes only an empty user table. Change the password in Settings; later changes to .env do not overwrite stored credentials. Create draft mappings with the actual source hostname and publish them from Revisions. Unknown routes return 404 until configured.

Before reverse proxy actions, edit upstream_cidr_allowlist in config.toml (or config.redis.toml when Redis is enabled); it denies every upstream by default. These files are mounted read-only. Restart proxy after configuration changes. Redirect mappings need no upstream network allowlist.

## Alternatives

- SQLite + Caddy: set COMPOSE_PROFILES=caddy and DATABASE_URL=sqlite:///data/proxy.db. SQLite persists in proxy-data. This does not transfer existing PostgreSQL/SQLite data between backends.
- PostgreSQL + Caddy + Redis: set COMPOSE_PROFILES=postgres,caddy,redis and PROXY_CONFIG_FILE=config.redis.toml. Redis is advisory; routing always uses local memory.
- External PostgreSQL: remove postgres from COMPOSE_PROFILES and set DATABASE_URL to the external database URL (use TLS verification outside trusted local networking).
- No Caddy: remove caddy from COMPOSE_PROFILES. PROXY_BIND_IP stays 127.0.0.1; adjust only if intentionally exposing the direct listener.

## Upgrades and backups

The .env.example pins PROXY_IMAGE to an immutable GHCR digest. Keep .env, TOML configuration, Caddyfile and the original COMPOSE_PROJECT_NAME during upgrades. Back up the database and certificate volumes first. Extract the new bundle into a separate directory, compare configuration/schema changes, then update only PROXY_IMAGE in the existing .env and reviewed deployment files:

```bash
docker compose pull
docker compose up -d --wait --wait-timeout 180
docker compose logs --tail=100 proxy postgres caddy
```

Do not overwrite .env with the example or change the project name. Existing installations must retain their original project name so named volumes are reused. Never use down -v on a deployment whose data must be retained. Image rollback is safe only when the previous binary supports the migrated database schema; configuration rollback is a separate Revisions operation.

Example PostgreSQL backup (store it in a private directory):

```bash
umask 077
docker compose exec -T postgres sh -c 'pg_dump -U "$POSTGRES_USER" -d "$POSTGRES_DB" -Fc' > database.dump
```

GHCR packages are private by default on first publication. For anonymous pulls, the repository owner must make the package public. Otherwise run docker login ghcr.io using a token authorized for read:packages before docker compose pull. Private GitHub Release downloads also require GitHub authentication. The release image currently targets linux/amd64 only.
