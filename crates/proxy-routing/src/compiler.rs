use super::*;
impl RoutingSnapshot {
    pub fn compile(
        revision: i64,
        mut config: Configuration,
        redirect_allowlist: &[String],
    ) -> Result<Self> {
        ensure!(
            config.schema_version <= 1,
            "unsupported configuration schema"
        );
        config.schema_version = 1;
        let mut upstreams = HashMap::new();
        let mut names = HashSet::new();
        for u in &config.upstreams {
            ensure!(
                !u.id.is_empty() && names.insert(u.name.clone()),
                "duplicate upstream name"
            );
            normalize_host(&u.hostname)?;
            ensure!(
                !u.hostname.contains(':') || u.hostname.starts_with('['),
                "upstream hostname must not include port"
            );
            ensure!(
                matches!(u.scheme.as_str(), "http" | "https")
                    && u.port > 0
                    && u.connect_timeout_ms > 0
                    && u.read_timeout_ms > 0,
                "invalid upstream settings"
            );
            ensure!(
                u.scheme != "https" || u.tls_verify,
                "TLS verification cannot be disabled"
            );
            ensure!(
                upstreams.insert(u.id.clone(), u.clone()).is_none(),
                "duplicate upstream ID"
            );
        }
        let mut domains = HashMap::new();
        for d in &config.domains {
            ensure!(
                domains
                    .insert(normalize_host(&d.hostname)?, d.enabled)
                    .is_none(),
                "duplicate domain"
            );
        }
        let mut hosts: HashMap<String, HostRoutes> = HashMap::new();
        let mut mappings = Vec::new();
        let mut keys = HashSet::new();
        let mut ids = HashSet::new();
        // Ambiguity validation uses indexed groups, not pairwise comparison of all mappings.
        let mut conflicts: HashMap<(String, String, String, i32, String), Vec<String>> =
            HashMap::new();
        for m in &mut config.mappings {
            ensure!(
                !m.id.is_empty() && ids.insert(m.id.clone()),
                "duplicate or empty mapping ID"
            );
            ensure!(
                !m.mapping_key.is_empty()
                    && m.mapping_key.len() <= 128
                    && m.mapping_key
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b)),
                "invalid mapping key"
            );
            ensure!(
                keys.insert(m.mapping_key.clone()),
                "duplicate mapping key: {}",
                m.mapping_key
            );
            validate_path(&m.source_matcher.path_value)?;
            ensure!(
                m.source_matcher.path_value.matches('/').count() <= 64,
                "path exceeds 64 segments"
            );
            ensure!(!m.source_matcher.hosts.is_empty(), "source hosts required");
            for method in &mut m.source_matcher.methods {
                *method = method.to_ascii_uppercase();
                http::Method::from_bytes(method.as_bytes())?;
            }
            m.source_matcher.methods.sort();
            m.source_matcher.methods.dedup();
            validate_action(&m.default_action, &upstreams, redirect_allowlist)?;
            ensure!(m.rules.len() <= 64, "mapping exceeds 64 rules");
            let mut rule_priorities = HashSet::new();
            let mut rule_ids = HashSet::new();
            let mut rules = Vec::new();
            for r in &m.rules {
                ensure!(rule_ids.insert(&r.id), "duplicate rule ID");
                ensure!(
                    r.condition_schema_version == 1,
                    "unsupported condition schema version"
                );
                let condition = CompiledCondition::compile(&r.condition)?;
                validate_action(&r.action, &upstreams, redirect_allowlist)?;
                if r.enabled {
                    ensure!(
                        rule_priorities.insert(r.priority),
                        "enabled rules must have distinct priorities"
                    );
                    rules.push((r.priority, condition, r.action.clone()));
                }
            }
            rules.sort_by_key(|(p, _, _)| std::cmp::Reverse(*p));
            let compiled = CompiledMapping {
                mapping: m.clone(),
                rules: rules.into_iter().map(|(_, c, a)| (c, a)).collect(),
            };
            if !m.enabled {
                continue;
            }
            let idx = mappings.len();
            for host in &m.source_matcher.hosts {
                let host = normalize_host(host)?;
                if domains.get(&host) == Some(&false) {
                    continue;
                }
                let routes = hosts.entry(host.clone()).or_default();
                let source = &m.source_matcher;
                let keyed = source.path_type == PathType::Template
                    && source.path_value.split('/').any(|s| s == "{mapping_id}");
                let conflict_key = (
                    host,
                    format!("{:?}", source.path_type),
                    source.path_value.clone(),
                    m.priority,
                    if keyed {
                        m.mapping_key.clone()
                    } else {
                        String::new()
                    },
                );
                let methods = conflicts.entry(conflict_key).or_default();
                let incoming = if source.methods.is_empty() {
                    vec!["*".to_string()]
                } else {
                    source.methods.clone()
                };
                ensure!(
                    !methods
                        .iter()
                        .any(|p| p == "*" || incoming.iter().any(|n| n == "*" || n == p)),
                    "ambiguous mappings with equal priority"
                );
                methods.extend(incoming);
                match source.path_type {
                    PathType::Exact => routes
                        .exact
                        .entry(source.path_value.clone())
                        .or_default()
                        .push(idx),
                    PathType::Prefix => {
                        let mut node = &mut routes.prefix;
                        for byte in source.path_value.bytes() {
                            node = node.children.entry(byte).or_default();
                        }
                        node.routes.push(idx);
                    }
                    PathType::Template => {
                        let group_id =
                            if let Some(id) = routes.template_patterns.get(&source.path_value) {
                                *id
                            } else {
                                let id = routes.groups.len();
                                routes.templates.insert(&source.path_value, id)?;
                                routes.groups.push(RouteGroup::default());
                                routes
                                    .template_patterns
                                    .insert(source.path_value.clone(), id);
                                id
                            };
                        let group = &mut routes.groups[group_id];
                        if keyed {
                            group
                                .keys
                                .entry(m.mapping_key.clone())
                                .or_default()
                                .push(idx);
                        } else {
                            group.routes.push(idx);
                        }
                    }
                }
            }
            mappings.push(compiled);
        }
        Ok(Self {
            revision,
            configuration: config,
            mappings,
            upstreams,
            hosts,
        })
    }
}
