# Domain Model

`proxy-domain` contains Serde/chrono/UUID types with no Axum, SQLx, Pingora or Redis dependencies.

Domain has immutable ID, hostname, enabled state and creation/update times. Mapping has immutable ID, globally unique mapping_key, metadata, enabled state, priority, SourceMatcher, ordered rules and fallback Action. SourceMatcher contains hosts, path_type (exact/prefix/template), path_value and optional methods. This flat JSON path representation preserves compatibility with the inspected source.

MappingRule is contained in one mapping and persisted with a foreign-key mapping_id. Rules have stable IDs, name, priority, enabled state, condition_schema_version=1, Condition and Action. SQLite owns the persisted parent relationship; no independent evaluator exists.

Condition is a tagged enum: all/any/not/time_window/date_range/device_type/query_parameter_equals. Action is redirect or reverse_proxy. Redirects contain status, target_url, path_handling and query_policy. Reverse proxy actions reference an upstream ID and optional rewrite with preserve_host.

Upstream contains immutable ID, unique name, scheme, hostname, explicit port, mandatory HTTPS verification and positive connect/read timeouts. ConfigRevision records version, lifecycle status, checksum and creation time; its serialized immutable configuration is stored in SQLite data. Audit events record operation, principal, optional resource and revision in additive schema columns.

## Validation
Mapping keys are 1..128 ASCII alphanumeric/underscore/hyphen. Source paths start with /, are at most 4096 bytes and reject query/fragment, backslash, controls and dot segments. Hostnames normalize case, trailing dot and authority port. Redirects require HTTP(S), no credentials and safe headers. Unknown upstream IDs, duplicate IDs/keys, conflicting equal-priority routes and equal-priority enabled rules are rejected. Each mapping permits at most 64 rules; each condition has at most 128 nodes and depth 12. Schema version 1 is the supported representation.
