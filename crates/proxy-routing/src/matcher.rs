use super::*;
impl RoutingSnapshot {
    fn pick<'a>(&'a self, indices: &[usize], method: Option<&str>) -> Option<&'a CompiledMapping> {
        indices
            .iter()
            .map(|i| &self.mappings[*i])
            .filter(|m| {
                method.is_none()
                    || m.mapping.source_matcher.methods.is_empty()
                    || m.mapping
                        .source_matcher
                        .methods
                        .iter()
                        .any(|v| Some(v.as_str()) == method)
            })
            .max_by_key(|m| m.mapping.priority)
    }
    pub fn lookup(&self, host: &str, path: &str, method: &str) -> Option<RouteMatch<'_>> {
        self.lookup_inner(host, path, Some(method))
    }
    pub fn path_exists(&self, host: &str, path: &str) -> bool {
        self.lookup_inner(host, path, None).is_some()
    }
    fn lookup_inner(&self, host: &str, path: &str, method: Option<&str>) -> Option<RouteMatch<'_>> {
        let host = normalize_host(host).ok()?;
        let routes = self.hosts.get(&host)?;
        if let Some(mapping) = routes.exact.get(path).and_then(|v| self.pick(v, method)) {
            return Some(RouteMatch {
                mapping,
                parameters: HashMap::new(),
            });
        }
        if let Ok(found) = routes.templates.at(path) {
            let group = &routes.groups[*found.value];
            let indices = found
                .params
                .get("mapping_id")
                .and_then(|key| group.keys.get(key))
                .unwrap_or(&group.routes);
            if let Some(mapping) = self.pick(indices, method) {
                return Some(RouteMatch {
                    mapping,
                    parameters: found
                        .params
                        .iter()
                        .map(|(k, v)| (k.into(), v.into()))
                        .collect(),
                });
            }
        }
        let mut node = &routes.prefix;
        let mut best = None;
        for byte in path.bytes() {
            node = match node.children.get(&byte) {
                Some(n) => n,
                None => break,
            };
            if let Some(mapping) = self.pick(&node.routes, method) {
                best = Some(mapping);
            }
        }
        best.map(|mapping| RouteMatch {
            mapping,
            parameters: HashMap::new(),
        })
    }
}
