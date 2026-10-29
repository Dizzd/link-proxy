use crate::service::*;
pub struct ProxyContext {
    pub(crate) snapshot: Option<Arc<RoutingSnapshot>>,
    pub(crate) action: Option<Action>,
    pub(crate) parameters: HashMap<String, String>,
    pub(crate) host: String,
    pub(crate) bytes_read: usize,
    pub(crate) started: Instant,
}
