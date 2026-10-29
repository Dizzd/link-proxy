use anyhow::Result;
impl crate::Store {
    pub async fn cleanup(&self, hours: i64) -> Result<()> {
        let cutoff = (chrono::Utc::now() - chrono::Duration::hours(hours))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        sqlx::query("DELETE FROM import_staging_records WHERE job_id IN (SELECT id FROM import_jobs WHERE status IN ('completed','failed','cancelled','ready') AND updated_at < $1)").bind(&cutoff).execute(&self.pool).await?;
        sqlx::query("UPDATE import_jobs SET status='cancelled',result='Staging expired' WHERE status='ready' AND updated_at < $1").bind(&cutoff).execute(&self.pool).await?;
        sqlx::query("DELETE FROM admin_sessions WHERE expires_at <= $1")
            .bind(chrono::Utc::now().timestamp())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
