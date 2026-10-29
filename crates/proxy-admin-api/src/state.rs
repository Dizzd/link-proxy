use crate::router::*;
#[derive(Clone)]
pub(crate) struct ApiState {
    pub(crate) control: Arc<ControlPlane>,
    pub(crate) token_hash: Option<[u8; 32]>,
    pub(crate) login_rate: Arc<Mutex<(Instant, u32)>>,
    pub(crate) auth_workers: Arc<tokio::sync::Semaphore>,
    pub(crate) rate: Arc<Mutex<(Instant, u32)>>,
}
