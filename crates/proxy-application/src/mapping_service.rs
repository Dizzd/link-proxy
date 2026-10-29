use crate::ControlPlane;
use proxy_domain::Mapping;
impl ControlPlane {
    pub async fn save_mapping(&self, mapping: Mapping) -> anyhow::Result<()> {
        self.update_draft(|config| {
            if let Some(old) = config.mappings.iter_mut().find(|m| m.id == mapping.id) {
                *old = mapping
            } else {
                config.mappings.push(mapping)
            }
            Ok(())
        })
        .await?;
        Ok(())
    }
}
