# Product Requirements

## Business problem
Destination URLs and backend services change while printed or distributed source URLs must remain stable. Operators need host/path mappings, conditional routing and bulk maintenance without restarting traffic handling.

## Scope
Unified Mapping owns source matching, ordered conditional rules and a fallback ReverseProxy or Redirect action. Hosts may share upstreams. Conditions support local time, dates, device heuristics, exact query parameters and nested All/Any/Not groups. The selected SQLite/PostgreSQL database is authoritative; Redis is advisory. No QR-specific entity exists.

## User stories
- As an operator, I create a mapping in a draft, simulate representative requests and publish one immutable revision.
- As an operator, I import thousands of mappings, inspect row errors and explicitly commit a validated batch.
- As an operator, I restore a prior configuration as a new revision and retain audit history.
- As a service owner, I continue serving the last valid local snapshot when Redis is unavailable.

## Acceptance evidence
A criterion is complete only after execution. Required checks are listed in IMPLEMENTATION_PLAN.md and TEST_PLAN.md; actual command logs are stored under verification/. HTTP traffic must exercise Pingora, not just the simulator. Docker configuration is not considered validated until a real engine builds and runs it.

## Limits
Single active control writer per database. No automatic upstream retries. Private administration with a persisted username/password administrator and revocable sessions. Device recognition is a routing heuristic, never authorization. Benchmark results describe the measurement host only.
