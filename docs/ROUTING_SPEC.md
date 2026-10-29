# Routing Specification

1. Normalize Host from HTTP authority to lowercase hostname, excluding port and trailing dot; reject malformed labels.
2. Match exact path, then template, then prefix. Exact lookup is indexed by host/path. Template `{mapping_id}` values select a mapping_key via a HashMap rather than scanning concrete mappings. Other captures are typed string routing parameters, never arbitrary destinations.
3. Within a path group, select a method-compatible mapping with highest priority. Empty methods means any method. Method names normalize uppercase when compiling.
4. Prefix lookup uses longest matching byte prefix, then priority within that prefix. `/api` therefore also matches `/apix`; configure `/api/` when a segment boundary is required.
5. Evaluate enabled conditional rules by descending priority and choose the first true condition. Equal enabled rule priorities are rejected. Use default_action if none match.
6. Return 404 when no path/host mapping exists; return 405 when a path exists but no mapping accepts the method.

Enabled domain=false suppresses mappings for that hostname. A separate domain registration is optional; mappings may directly name hosts. Disabled mappings are validated but not indexed. Equal-priority overlapping methods for the same source identity are rejected. Equivalent template patterns with conflicting parameter names are rejected by matchit. Different-priority mappings remain deterministic.

## Actions
Redirect supports 301/302/303/307/308 and a fixed validated HTTP(S) target. Replace uses the configured target path. Append adds the incoming path to the target base. Query drop does not copy the source query (configured target query remains), preserve replaces the target query with source query, merge appends source query to configured query. Obvious direct HTTP authority/path loops are rejected at execution. Dynamic captures cannot change target host or scheme.

Reverse proxy references a compiled upstream. Rewrite replaces `{parameter}` tokens in a configured path; unresolved tokens fail. Original query is retained. preserve_host=false uses upstream authority with non-default port; true forwards the incoming Host. Pingora handles streaming and connection/header protocol details. No automatic retry is configured.
