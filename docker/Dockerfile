FROM node:22.22.0-bookworm-slim AS admin-builder
WORKDIR /source
RUN npm install -g pnpm@10.28.2
COPY package.json pnpm-workspace.yaml pnpm-lock.yaml ./
COPY apps/admin-web/package.json apps/admin-web/package.json
RUN pnpm install --frozen-lockfile
COPY apps/admin-web apps/admin-web
RUN pnpm --dir apps/admin-web build

FROM rust:1.98.1-bookworm AS rust-builder
RUN apt-get update && apt-get install -y --no-install-recommends cmake pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /source
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates crates
COPY apps/proxy-server apps/proxy-server
COPY migrations migrations
RUN cargo build --locked --release -p proxy-server

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 curl && rm -rf /var/lib/apt/lists/* && useradd --uid 10001 --create-home proxy && mkdir /data /app && chown proxy:proxy /data /app
WORKDIR /app
COPY --from=rust-builder /source/target/release/proxy-server /usr/local/bin/proxy-server
COPY --from=admin-builder /source/apps/admin-web/dist /app/admin
COPY config/config.docker.toml /app/config.toml
ENV PROXY_CONFIG=/app/config.toml
USER proxy
EXPOSE 8080 9000
VOLUME /data
STOPSIGNAL SIGTERM
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s CMD curl --fail --silent http://127.0.0.1:9000/health/ready > /dev/null || exit 1
ENTRYPOINT ["proxy-server"]
