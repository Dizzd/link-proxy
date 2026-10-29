# Management API Contract

Base `/api/v1`; sign in with username/password to obtain an HttpOnly session cookie. Cookie-authenticated mutations require X-CSRF-Token from the login/session response and a same-origin Origin when present. /auth/login is public; /auth/session requires a session. Other management APIs require a valid session or an explicitly enabled optional API token. JSON bodies use Content-Type application/json. Entity created_at/updated_at values are RFC3339 UTC; revision/audit text timestamps use the selected database's UTC format. IDs are opaque strings. Draft edits require explicit publication except import commit.

## Authentication and database status

| Method | Path | Request and response |
|---|---|---|
| POST | /auth/login | `{username,password}`; 200 `{user:{id,username},csrf_token,expires_at}` and Set-Cookie; generic 401 for invalid credentials |
| GET | /auth/session | Same response; restores CSRF after a page reload; 401 for expired/revoked session |
| POST | /auth/logout | CSRF required; 204 clears cookie and revokes the server session |
| POST | /auth/password | `{current_password,new_password}`; CSRF required; 204 revokes all user sessions and clears cookie; 403 for incorrect current password |
| GET | /system/database/status | `{backend,connected,database_revision,active_revision}`; no credentials/connection URL |

Usernames normalize to lowercase ASCII (3..64 characters, letters/digits/underscore/dot/hyphen/@). New passwords contain 12..1024 bytes and are not blank. Session tokens expire after auth.session_ttl_seconds (default 12 hours); up to 10 sessions per user. Login is limited to 20 attempts/minute per process by default and 2 concurrent Argon2 workers. Authentication responses disable caching. Sessions need no password on reload. Legacy Authorization: Bearer tokens require auth.api_token_enabled=true and at least 24 bytes in ADMIN_API_TOKEN; the UI never uses them. There is one bootstrap administrator and no user-management/RBAC UI.

Health GET /health/live and /health/ready are public minimal probes. /health and /metrics remain authenticated.


## Mappings and rules
| Method | Path | Request and response |
|---|---|---|
| GET | /mappings | filters search/domain/action/enabled, offset/limit; `{items,total,offset}` |
| POST | /mappings | Mapping; 201 with stored draft Mapping |
| GET | /mappings/{id} | Mapping or 404 |
| PATCH | /mappings/{id} | Partial top-level Mapping fields; nested fields replace the complete nested value |
| DELETE | /mappings/{id} | 204, removes draft only |
| GET | /mappings/{id}/rules | `{items: MappingRule[]}` |
| POST | /mappings/{id}/rules | complete MappingRule; `{ok:true}` |
| PATCH | /mappings/{id}/rules/{rule_id} | partial top-level rule; `{ok:true}` |
| DELETE | /mappings/{id}/rules/{rule_id} | 204 |
| POST | /mappings/validate | Mapping; compiles it in current draft context without persistence; `{valid:true}` |
| POST | /mappings/simulate | Simulation; selected action/parameters/revision, no upstream traffic |

Mapping schema:
```json
{
 "id":"example-001", "mapping_key":"QR001", "name":"Example", "description":"",
 "enabled":true,"priority":0,
 "source_matcher":{"hosts":["abc.example.com"],"path_type":"exact","path_value":"/QR001","methods":["GET"]},
 "rules":[{"id":"mobile-rule","name":"Mobile","priority":10,"enabled":true,"condition_schema_version":1,
   "condition":{"type":"device_type","in":["mobile"]},
   "action":{"type":"redirect","status":307,"target_url":"https://mobile.example.com/","path_handling":"replace","query_policy":"drop"}}],
 "default_action":{"type":"redirect","status":302,"target_url":"https://default.example.com/","path_handling":"replace","query_policy":"drop"}
}
```

IDs and created_at are immutable in PATCH. updated_at is set by mapping/rule edits. Exact, prefix and template path types are supported. Mapping ID/key and rule IDs must be unique. A reverse proxy action is `{"type":"reverse_proxy","upstream_id":"backend","rewrite":"/api/{id}","preserve_host":false}`. Rule conditions use the versioned AST in CONDITIONAL_ROUTING.md.

Simulation request:
```json
{"host":"abc.example.com","path":"/QR001","method":"GET","user_agent":"iPhone Mobile","query":"campaign=a","now":"2026-10-29T01:00:00Z","draft":false}
```
Defaults: method GET, query/user_agent empty, clock now UTC, active snapshot. Response contains revision, mapping_id, mapping_key, captured parameters and selected action. draft=true compiles the draft for this simulation without activating it. It never increments production statistics.

## Domains and upstreams
| Method | Path | Schema/result |
|---|---|---|
| GET | /domains | `{items:Domain[]}` |
| POST | /domains | `{id?,hostname,enabled?}` → Domain |
| PATCH | /domains/{id} | top-level partial Domain → `{ok:true}` |
| DELETE | /domains/{id} | 204 |
| GET | /upstreams | `{items:Upstream[]}` |
| POST | /upstreams | complete Upstream → Upstream |
| PATCH | /upstreams/{id} | partial Upstream → `{ok:true}` |
| DELETE | /upstreams/{id} | 204; validation rejects remaining mapping references |

Upstream: `{id?,name,scheme:"http"|"https",hostname,port,tls_verify:true,connect_timeout_ms:3000,read_timeout_ms:30000}`. Domains have optional generated id, enabled=true and UTC created_at/updated_at. Lists of domains/upstreams return the complete draft resource set in this MVP; mappings/imports/errors/audits are paginated.

## Imports and exports
| Method | Path | Behavior |
|---|---|---|
| POST | /imports | multipart `file`, `format=csv|xlsx|json`, `conflict_policy=reject|skip|upsert`; 202 `{id,status:"pending"}` |
| GET | /imports | paginated `{items,total}` |
| GET | /imports/{id} | ImportJob including status/counts/base revision/generation/result/committed_revision |
| GET | /imports/{id}/preview | offset/limit; `{job,items:StagedRecord[]}` |
| GET | /imports/{id}/errors | offset/limit; `{total,items}`; format=csv downloads report |
| POST | /imports/{id}/commit | body below plus Idempotency-Key; one revision, idempotent retry |
| POST | /imports/{id}/cancel | cancellable pending/validating/ready; `{status:"cancelled"}` |
| GET | /imports/templates | format=csv|xlsx|json; attachment download |
| GET | /exports/mappings | format=csv|xlsx|json; optional mapping filters; draft export attachment |

Commit body:
```json
{"import_job_id":"job-id","expected_revision":3,"conflict_policy":"reject","confirmation":true}
```
`Idempotency-Key` is required (1..128 bytes). ID must match URL. Policy and base revision must match the validated job; draft_generation must still match. Result `{inserted,updated,skipped,revision}`. A retry after completion returns the same result only when key and original payload agree. Preview staging representation is Serde's external enum form, e.g. `{"Mapping":{...}}`. Error rows contain file_name, location, field, code, description, suggestion. No active data changes occur at upload/preview.

## Configuration and operations
| Method | Path | Request/result |
|---|---|---|
| GET | /config/active | `{revision,draft_generation,configuration}` |
| GET | /config/revisions | `{items:[{version,status,checksum,created_at}]}`, latest 200 |
| POST | /config/publish | `{expected_revision:n}` → `{revision:new}` |
| POST | /config/revisions/{version}/restore | `{expected_revision:n}` → new revision |
| GET | /system/cache/status | mode, connected, active_revision, database_revision, last_sync, error |
| POST | /system/cache/reconcile | reconcile from SQLite; return cache status |
| GET | /audit-events | offset/limit; `{items}` |

Outside `/api/v1`: authenticated `GET /health`, `/health/live`, `/health/ready` return status/revision; `GET /metrics` exposes Prometheus text. Readiness confirms a loaded snapshot and a bounded TCP connection to the configured Pingora listener. It returns 503 when that listener is unavailable; liveness does not depend on SQLite or Redis.

## Errors and bounds
Typical structured application error: `{"error":{"code":"409","message":"revision conflict"}}`. 401 for missing/invalid session or optional API token; 404 for missing resources; 409 for stale revision/conflict/idempotency mismatch; 413 for oversized JSON/body; 422 for invalid configuration; 429 for worker/rate capacity; sanitized 500 for database failures. Axum extraction errors are converted to the same JSON error envelope. Lists default to limit=50, clamped 1..500; offset is zero-based. Rate-limit response supplies Retry-After=60. Malformed JSON retains Axum's status and a structured message; application validation never publishes a partially valid configuration.
