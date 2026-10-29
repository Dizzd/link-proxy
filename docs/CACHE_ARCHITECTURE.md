# Cache Architecture

The local ArcSwap<RoutingSnapshot> is always present and owns the complete compiled configuration. A request holds its own Arc through completion. It never queries SQLite or Redis. Cache mode memory remains independent of a Redis URL or service.

Optional memory_redis requires `[cache.redis].enabled=true` and the URL in the environment variable named by url_env. Version envelopes carry schema_version, revision, checksum and serialized data. Validation checks revision and SHA-256 against authoritative configuration. Keys use `<prefix>config:revision:<n>`, `<prefix>config:latest` and `<prefix>config:updates`; default prefix is dynamic_proxy:. Version keys expire after seven days.

Redis writes use Lua compare-and-set so an older delayed writer cannot move latest backwards. Pub/Sub publishes only when latest advances; equal-version synchronization does not cause notification loops. The binary subscribes with bounded hint buffering and reconnects in the background. Every notification triggers SQLite reconciliation; notification payloads cannot activate arbitrary snapshots.

Periodic reconciliation compares the committed database revision with the local active revision and recovers missed notifications. Redis degradation retains the last valid local snapshot, logs sanitized degradation and records connection/error metrics. The selected SQLite/PostgreSQL database remains authoritative even when Redis claims a newer revision. A cache checksum mismatch is never activated. A corrupted existing cache entry is reported as degradation; operator repair or expiration is needed before that cache key can be trusted again.

Sharing Redis does not replicate SQLite. Use one control writer and a common authoritative database, or design an actual repository replication strategy before horizontal control-plane deployment.

## Single-file startup configuration

Deployments mount only config.toml. CACHE_MODE=memory or CACHE_MODE=memory_redis overrides the TOML cache mode and automatically sets cache.redis.enabled consistently at startup. Other Redis tuning is preserved, local memory stays mandatory, and invalid/empty/non-UTF-8 values fail startup. An absent variable leaves validated TOML settings unchanged. Enable the Redis Compose service separately with the redis profile; service selection does not silently change application behavior. Configuration changes require a proxy restart. New environment handling requires a new image; published v0.1.0 retains its original behavior.
