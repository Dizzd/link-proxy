use crate::ControlPlane;
use proxy_domain::Upstream;
impl ControlPlane {
    pub async fn save_upstream(&self, upstream: Upstream) -> anyhow::Result<()> {
        self.update_draft(|config| {
            if let Some(old) = config.upstreams.iter_mut().find(|u| u.id == upstream.id) {
                *old = upstream
            } else {
                config.upstreams.push(upstream)
            }
            Ok(())
        })
        .await?;
        Ok(())
    }
}
