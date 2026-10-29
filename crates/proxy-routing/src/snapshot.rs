use super::*;
pub struct RoutingSnapshot {
    pub revision: i64,
    pub configuration: Configuration,
    pub mappings: Vec<CompiledMapping>,
    pub upstreams: HashMap<String, Upstream>,
    pub(crate) hosts: HashMap<String, HostRoutes>,
}
pub struct RouteMatch<'a> {
    pub mapping: &'a CompiledMapping,
    pub parameters: HashMap<String, String>,
}
