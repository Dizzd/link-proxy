CREATE INDEX idx_mapping_source ON mappings ((data::jsonb #>> '{source_matcher,path_type}'),(data::jsonb #>> '{source_matcher,path_value}'));
CREATE INDEX idx_domains_enabled ON domains(enabled,hostname);
CREATE INDEX idx_audit_revision ON audit_events(revision,id);
