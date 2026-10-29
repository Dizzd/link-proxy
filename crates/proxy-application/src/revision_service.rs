use crate::ControlPlane;
impl ControlPlane {
    pub async fn restore_revision(
        &self,
        version: i64,
        expected_revision: i64,
    ) -> anyhow::Result<i64> {
        self.publish(expected_revision, Some(version)).await
    }
}
