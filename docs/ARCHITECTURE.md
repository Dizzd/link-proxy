# Architecture

## Components and dependencies
Domain → Routing. Domain → Storage. Domain/Storage/Observability → Cache. Domain/Cache → Import. These feed Application. Admin API and Pingora depend on Application. The binary owns lifecycle and listeners. Root Cargo.toml is a virtual workspace. Routing tests belong to proxy-routing, control/import/cache integration tests to proxy-application, API tests to proxy-admin-api and live HTTP tests to proxy-e2e.

## Data plane
Pingora reads one Arc from ArcSwap for each request. That Arc remains in request context through streaming so a publication cannot mix routing and upstream definitions between revisions. Host/exact indexes use HashMap, templates use matchit plus a mapping-key index, prefixes use a byte trie. Conditions are validated/compiled during publication. SQL and Redis are never called during request routing. Upstream DNS is resolved at connection time and a concrete allowlisted SocketAddr is given to Pingora.

## Control plane
Authenticated Axum handlers mutate validated drafts in the selected SQLite/PostgreSQL database. Application services serialize mutations within the process. Candidate snapshots compile in spawn_blocking. Database transaction commit precedes ArcSwap store. Active revision acknowledgement is a separate recoverable operation. Redis cache writes occur in background and never decide which configuration is authoritative.

## Request flow
Host/path/method → exact/template/prefix matcher → first enabled matching rule by descending priority → fallback action → validated redirect or streamed Pingora upstream. Unknown paths return 404; known paths without a matching method return 405. Simulator calls the same matching and action evaluator without upstream access or traffic metrics.

## Deployment boundary
One active control writer per selected SQLite/PostgreSQL database. SQLx Any uses separate embedded migrations; control writes lock the database singleton and compare revision/generation after compiling. Import scheduling remains process-local. Independent databases do not become replicated by sharing Redis. The database is the recoverable control log; routing uses only ArcSwap memory.
