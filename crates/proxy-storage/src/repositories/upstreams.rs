impl crate::Store {
    pub async fn upstreams(&self) -> anyhow::Result<Vec<proxy_domain::Upstream>> {
        let rows: Vec<String> = sqlx::query_scalar("SELECT data FROM upstreams ORDER BY id")
            .fetch_all(&self.pool)
            .await?;
        rows.iter()
            .map(|data| serde_json::from_str(data).map_err(Into::into))
            .collect()
    }
}
