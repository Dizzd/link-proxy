use super::*;
pub struct SimulationOutcome<'a> {
    pub mapping: &'a Mapping,
    pub action: &'a Action,
    pub parameters: HashMap<String, String>,
    pub revision: i64,
}
pub fn simulate<'a>(
    snapshot: &'a RoutingSnapshot,
    host: &str,
    path: &str,
    method: &str,
    context: &RequestContext<'_>,
) -> std::result::Result<SimulationOutcome<'a>, error::RoutingError> {
    let route = snapshot.lookup(host, path, method).ok_or_else(|| {
        if snapshot.path_exists(host, path) {
            error::RoutingError::MethodNotAllowed
        } else {
            error::RoutingError::NotFound
        }
    })?;
    Ok(SimulationOutcome {
        mapping: &route.mapping.mapping,
        action: route.mapping.action(context),
        parameters: route.parameters,
        revision: snapshot.revision,
    })
}
