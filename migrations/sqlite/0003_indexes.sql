CREATE INDEX IF NOT EXISTS idx_mapping_source ON mappings(json_extract(data,'$.source_matcher.path_type'),json_extract(data,'$.source_matcher.path_value'));
CREATE INDEX IF NOT EXISTS idx_domains_enabled ON domains(enabled,hostname);
CREATE INDEX IF NOT EXISTS idx_audit_revision ON audit_events(revision,id);
