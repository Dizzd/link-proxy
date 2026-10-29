# Security

## Administration
Default loopback listener. First administrator is bootstrapped from ADMIN_USERNAME/ADMIN_PASSWORD only if no user exists. Passwords are salted Argon2id hashes; new passwords require 12..1024 bytes. Unknown users and incorrect passwords receive the same error and password hashing work. Two blocking hash workers and a separate process-wide login rate (default 20/minute) bound resource use. Stored passwords are never overwritten by restart. The explicit --reset-admin-password command revokes all sessions.

Random session tokens live only in HttpOnly, SameSite=Strict cookies; the database stores SHA-256 token hashes with expiry. Cookies use Secure when auth.secure_cookies=true (required for HTTPS deployment). Cookie-authenticated mutations require a constant-time CSRF token comparison and reject cross-site Origin/Sec-Fetch-Site. The UI stores no credentials in localStorage or build variables. Session restore uses the cookie; logout revokes the server record, and password changes revoke all sessions. At most 10 concurrent sessions per user.

Optional legacy API tokens are disabled by default, with SHA-256 constant-time digest comparison when explicitly enabled. There is no multiuser RBAC UI. General admin requests have a separate fixed-window rate limit. JSON/multipart bodies are bounded. Audit records retain authenticated username, operation, resource ID and revision. Logs omit passwords and raw tokens. Minimal liveness/readiness probes are public; metrics, detailed health and all management operations require authentication. Apply TLS/VPN/SSH tunnels for remote administration.

## Proxy and SSRF
Default upstream network allowlist denies all addresses. DNS resolution is bounded by connect timeout; every connection selects a concrete SocketAddr only after CIDR validation, including IPv4-mapped IPv6 normalization. This prevents DNS rebinding from bypassing the policy. Operators must narrowly allow intended upstream networks; broad 0.0.0.0/0 deliberately permits metadata/private targets and is unsafe. HTTPS certificate and hostname verification are mandatory. Upstreams must have HTTP/HTTPS scheme and positive port/timeouts.

Incoming Forwarded and all X-Forwarded-* headers are removed and replaced with server-observed host/protocol/client IP. The direct proxy listener is HTTP; deploy Caddy for ingress TLS. Pingora streams bodies; content-length and observed body bytes enforce a 16 MiB default maximum. HTTP request URI/query sizes are bounded. No unsafe request retry is added. Connection/read/write timeouts are configured per upstream. A single total wall-clock body deadline and downstream slow-client tuning remain deployment hardening work.

## Redirects
Only fixed validated HTTP(S) destinations without credentials or raw controls. Header parsing rejects CRLF injection. Optional redirect host allowlist. Source/template/query values cannot select arbitrary hosts. Direct same-authority HTTP loops are rejected; multi-hop loops across hosts remain an operator responsibility.

## Imports
64 MiB default file, 50,000 records, 64 KiB record, 256 MiB expanded XLSX and 512 ZIP entries; all configurable limits remain bounded. ZIP entry traversal and actual decompressed lengths are checked before workbook parsing. JSON depth capped at 48. Jobs/workers are bounded; staging is durable database storage rather than public filesystem uploads. Parser errors are handled without executing formulas or code. CSV formula-leading cells are apostrophe-escaped; XLSX writes explicit text cells. No active changes without preview and explicit commit. Reject/skip/upsert are explicit; IDs and keys are checked and existing mappings are never silently overwritten.

## Known operational limits
Use one active control-plane process per database. Rate limiting and import workers are not distributed. There is no multiuser/RBAC administration UI. Container images and TLS integration require actual deployment verification before production. Keep SQLite volume permissions private and back it up with its WAL-consistent backup mechanism.

Upload buffering has a separate semaphore, acquired before multipart file bytes are read. Its capacity equals the configured import worker count. At most that many uploads and that many parser jobs can hold file buffers concurrently; excess requests receive 429.
